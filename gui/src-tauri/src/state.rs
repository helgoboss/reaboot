use crate::worker::ReabootWorkerCommand;
use reaboot_core::api::InstallerConfig;
use std::sync::Mutex;
use tempdir::TempDir;

pub struct ReabootAppState {
    /// Non-enhanced config from client (doesn't contain package URLs from recipe).
    pub installer_config: Mutex<InstallerConfig>,
    pub worker_command_sender: tauri::async_runtime::Sender<ReabootWorkerCommand>,
    pub interaction_sender: tokio::sync::broadcast::Sender<bool>,
    pub temp_dir_for_reaper_download: TempDir,
}

impl ReabootAppState {
    pub fn extract_installer_config(&self) -> InstallerConfig {
        self.installer_config.lock().unwrap().clone()
    }
}
