use anyhow::Context;
use reaboot_lib::api::{InstallationStatusEvent, RemoteFile, WorkerCommand};
use reaboot_lib::downloader::{DownloadStatus, Downloader};
use reaboot_lib::installer::{Installer, InstallerListener};
use serde::Serialize;
use tauri::async_runtime::Receiver;
use tauri::{AppHandle, Manager};

pub async fn keep_processing(mut receiver: Receiver<WorkerCommand>, app_handle: AppHandle) {
    while let Some(command) = receiver.recv().await {
        if let Err(e) = process(command, &app_handle).await {
            // TODO-high Add ability to emit error
        }
    }
}

async fn process(command: WorkerCommand, app_handle: &AppHandle) -> anyhow::Result<()> {
    match command {
        WorkerCommand::Install => {
            process_install(app_handle).await?;
        }
    }
    Ok(())
}

async fn process_install(app_handle: &AppHandle) -> anyhow::Result<()> {
    let downloader = Downloader::new(3);
    let download_dir =
        tempdir::TempDir::new("reaboot-").context("couldn't create temp directory")?;
    let installer = Installer::new(downloader, download_dir.path().to_path_buf());
    let listening_app_handle = ListeningAppHandle(app_handle);
    installer.download_reaper(&listening_app_handle).await?;
    installer.download_reapack(&listening_app_handle).await?;
    Ok(())
}

struct ListeningAppHandle<'a>(&'a AppHandle);

impl<'a> ListeningAppHandle<'a> {
    fn emit_simple<T>(&self, name: &str, evt: T)
    where
        T: Clone + Serialize,
    {
        self.0.emit_all(name, evt).unwrap();
    }
}

impl<'a> InstallerListener for ListeningAppHandle<'a> {
    fn emit_installation_status(&self, event: InstallationStatusEvent) {
        self.emit_simple("installation-status", event);
    }

    fn emit_download_status(&self, event: DownloadStatus) {
        self.emit_simple("installation-status-progress", event.to_simple_progress());
    }
}
