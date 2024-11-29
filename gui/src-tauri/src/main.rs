// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use tauri::Manager;
use tauri_plugin_log::LogTarget;
use tempdir::TempDir;
use tracing::log::LevelFilter;

use reaboot_core::api::InstallerConfig;

use crate::app_handle::ReabootAppHandle;
use crate::state::ReabootAppState;
use crate::worker::ReabootWorker;

mod api;
mod app_handle;
mod command_handlers;
mod config_util;
mod state;
mod worker;

fn main() {
    let (worker_command_sender, worker_command_receiver) = tauri::async_runtime::channel(1);
    let (interaction_sender, interaction_receiver) = tokio::sync::broadcast::channel(10);
    let app_state = ReabootAppState {
        worker_command_sender,
        installer_config: Mutex::new(InstallerConfig::default()),
        temp_dir_for_reaper_download: TempDir::new("reaboot-")
            .expect("couldn't create temporary directory for REAPER download"),
        interaction_sender,
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
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
            let mut worker = ReabootWorker::new(
                worker_command_receiver,
                app_handle.clone(),
                interaction_receiver,
            );
            tauri::async_runtime::spawn(async move {
                worker.keep_processing_incoming_commands().await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
