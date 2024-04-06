// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::{Manager, State};
use tauri_plugin_log::LogTarget;
use tempdir::TempDir;
use tracing::log::LevelFilter;

use reaboot_core::api::InstallerConfig;

use crate::app_handle::ReabootAppHandle;
use crate::config_util::resolve_config_and_send_events;
use crate::recipe_id_extractor::extract_recipe_id_from_current_exe;
use crate::state::ReabootAppState;
use crate::worker::ReabootWorker;

mod api;
mod app_handle;
mod command_handlers;
mod config_util;
#[cfg(target_os = "macos")]
mod hdi_util;
mod recipe_id_extractor;
mod state;
mod worker;

fn main() {
    // let recipe_id = extract_recipe_id_from_current_exe();
    let recipe_id = Some("realearn".to_string());
    let (worker_command_sender, worker_command_receiver) = tauri::async_runtime::channel(1);
    let app_state = ReabootAppState {
        worker_command_sender,
        installer_config: Mutex::new(InstallerConfig::default()),
        temp_dir_for_reaper_download: TempDir::new("reaboot-")
            .expect("couldn't create temporary directory for REAPER download"),
        recipe_id,
        recipe: Mutex::new(None),
    };
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(LevelFilter::Info)
                .targets([LogTarget::Stdout])
                .build(),
        )
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            command_handlers::reaboot_command,
            command_handlers::get_reaper_eula,
        ])
        .setup(move |app| {
            // Setup worker
            let app_handle = ReabootAppHandle::new(app.app_handle());
            let mut worker = ReabootWorker::new(worker_command_receiver, app_handle.clone());
            tauri::async_runtime::spawn(async move {
                worker.keep_processing_incoming_commands().await;
            });
            // Fill recipe in parallel, if necessary
            tauri::async_runtime::spawn(async move {
                let app_state: State<ReabootAppState> = app_handle.get().state();
                if app_state.fill_recipe_if_necessary().await {
                    // Successfully filled recipe. (Re)send events.
                    let _ = resolve_config_and_send_events(
                        app_state.extract_installer_config(),
                        app_state.recipe_id.clone(),
                        app_state.extract_recipe(),
                        app_handle,
                    )
                    .await;
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
