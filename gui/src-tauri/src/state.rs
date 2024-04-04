use crate::worker::ReabootWorkerCommand;
use reaboot_core::api::InstallerConfig;
use std::sync::Mutex;
use tempdir::TempDir;

pub struct ReabootAppState {
    pub installer_config: Mutex<InstallerConfig>,
    pub worker_command_sender: tauri::async_runtime::Sender<ReabootWorkerCommand>,
    pub temp_dir_for_reaper_download: TempDir,
}
