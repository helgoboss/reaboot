use reaboot_lib::api::{InstallationStatusEvent, RemoteFile, WorkerCommand};
use serde::Serialize;
use std::time::Duration;
use tauri::async_runtime::Receiver;
use tauri::{AppHandle, Manager};

pub async fn keep_processing(mut receiver: Receiver<WorkerCommand>, app_handle: AppHandle) {
    while let Some(command) = receiver.recv().await {
        process(command, &app_handle).await;
    }
}

async fn process(command: WorkerCommand, app_handle: &AppHandle) {
    match command {
        WorkerCommand::Install => {
            process_install(app_handle).await;
        }
    }
}

async fn process_install(app_handle: &AppHandle) {
    app_handle.emit_installation_status(InstallationStatusEvent::Idle);
    app_handle.simulate_progress(1000).await;
    app_handle.emit_installation_status(InstallationStatusEvent::DownloadingReaper {
        file: RemoteFile {
            label: "bla".to_string(),
            url: "foo".to_string(),
        },
    });
    app_handle.simulate_progress(7000).await;
}

trait AppHandleExt {
    fn emit_installation_status(&self, evt: InstallationStatusEvent);
    fn emit_installation_status_progress(&self, evt: f64);
    fn emit_simple<T>(&self, name: &str, evt: T)
    where
        T: Clone + Serialize;
    async fn simulate_progress(&self, millis: u64) {
        for i in (0..millis).step_by(2) {
            self.emit_installation_status_progress(i as f64 / millis as f64);
            sleep(1).await;
        }
    }
}

impl AppHandleExt for AppHandle {
    fn emit_installation_status(&self, evt: InstallationStatusEvent) {
        self.emit_simple("installation-status", evt);
    }

    fn emit_installation_status_progress(&self, evt: f64) {
        self.emit_simple("installation-status-progress", evt);
    }

    fn emit_simple<T>(&self, name: &str, evt: T)
    where
        T: Clone + Serialize,
    {
        self.emit_all(name, evt).unwrap();
    }
}

async fn sleep(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await
}
