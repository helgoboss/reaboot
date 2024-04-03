// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use crate::app_handle::ReabootAppHandle;
use reaboot_core::api::InstallerConfig;
use tauri::Manager;
use tauri_plugin_log::LogTarget;
use tracing::log::LevelFilter;

use crate::state::ReabootAppState;
use crate::worker::ReabootWorker;

mod api;
mod app_handle;
mod command_handlers;
mod state;
mod worker;

fn main() {
    let (worker_command_sender, worker_command_receiver) = tauri::async_runtime::channel(1);
    let simple_command_state = ReabootAppState {
        worker_command_sender,
        installer_config: Mutex::new(InstallerConfig::default()),
    };
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(LevelFilter::Debug)
                .targets([LogTarget::Stdout])
                .build(),
        )
        .manage(simple_command_state)
        .invoke_handler(tauri::generate_handler![
            command_handlers::reaboot_command,
            command_handlers::reaper_eula,
        ])
        .setup(move |app| {
            let mut worker =
                ReabootWorker::new(worker_command_receiver, ReabootAppHandle(app.app_handle()));
            tauri::async_runtime::spawn(async move {
                worker.keep_processing_incoming_commands().await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
