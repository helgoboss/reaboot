use crate::worker::ReabootWorkerCommand;
use reaboot_lib::api::ReabootConfig;
use std::sync::Mutex;

pub struct ReabootAppState {
    pub config: Mutex<ReabootConfig>,
    pub worker_command_sender: tauri::async_runtime::Sender<ReabootWorkerCommand>,
}
