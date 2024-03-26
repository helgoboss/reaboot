use crate::state::ReabootAppState;
use reaboot_lib::api::{
    InstallationStatus, ReabootConfig, ReabootError, ReabootEvent, ResolvedReabootConfig,
};
use reaboot_lib::downloader::DownloadProgress;
use reaboot_lib::installer::InstallerListener;
use reaboot_lib::task_tracker::Summary;
use reaboot_lib::{reaboot_util, reaper_util};
use std::fmt::Display;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct ReabootAppHandle(pub AppHandle);

impl ReabootAppHandle {
    pub fn emit_initial_events(&self, config: &ReabootConfig) -> anyhow::Result<()> {
        let resolved_config = reaboot_util::resolve_config(config)?;
        let installation_status = reaboot_util::determine_initial_installation_status(
            &resolved_config.reaper_resource_dir,
            resolved_config.reaper_target,
        );
        self.emit_reaboot_event(ReabootEvent::ConfigResolved {
            state: resolved_config,
        });
        self.emit_reaboot_event(ReabootEvent::InstallationStatusChanged {
            status: installation_status,
        });
        Ok(())
    }

    pub fn emit_generic_error(&self, error: impl Display) {
        let error = ReabootError {
            display_msg: format!("{error:#}"),
        };
        self.emit_reaboot_event(ReabootEvent::Error { error })
    }

    pub fn emit_reaboot_event(&self, evt: ReabootEvent) {
        self.0.emit_all("reaboot_event", evt).unwrap();
    }

    pub fn emit_progress(&self, progress: f64) {
        self.0.emit_all("reaboot_progress", progress).unwrap();
    }
}

impl InstallerListener for ReabootAppHandle {
    fn emit_installation_status(&self, status: InstallationStatus) {
        self.emit_reaboot_event(ReabootEvent::InstallationStatusChanged { status });
    }

    fn emit_progress(&self, progress: f64) {
        self.emit_progress(progress);
    }
}
