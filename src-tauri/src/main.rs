// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::worker::keep_processing;
use reaboot_lib::api::WorkerCommand;
use tauri::{Manager, State};

mod worker;

fn main() {
    let (worker_command_sender, worker_command_receiver) = tauri::async_runtime::channel(1);
    let work_state = WorkState {
        command_sender: worker_command_sender,
    };
    tauri::Builder::default()
        .manage(work_state)
        .invoke_handler(tauri::generate_handler![greet, work])
        .setup(move |app| {
            tauri::async_runtime::spawn(keep_processing(worker_command_receiver, app.app_handle()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn work(command: WorkerCommand, state: State<WorkState>) {
    state.command_sender.blocking_send(command).unwrap()
}

struct WorkState {
    command_sender: tauri::async_runtime::Sender<WorkerCommand>,
}
