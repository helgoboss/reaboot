use reaboot_core::api::{InstallationStage, ReabootError, ReabootEvent};
use reaboot_core::installer::{InstallerListener, InstallerTask};
use std::fmt::Display;
use tauri::{AppHandle, Manager};

#[derive(Clone)]
pub struct ReabootAppHandle(pub AppHandle);

impl ReabootAppHandle {
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
    fn installation_stage_changed(&self, stage: InstallationStage) {
        self.emit_reaboot_event(ReabootEvent::InstallationStageChanged { stage });
    }

    fn installation_stage_progressed(&self, progress: f64) {
        self.emit_progress(progress);
    }

    fn task_started(&self, task_id: u32, task: InstallerTask) {
        todo!()
    }

    fn task_progressed(&self, task_id: u32, progress: f64) {
        todo!()
    }

    fn task_finished(&self, task_id: u32) {
        todo!()
    }

    fn warn(&self, message: impl Display) {
        todo!()
    }

    fn info(&self, message: impl Display) {
        todo!()
    }

    fn debug(&self, message: impl Display) {
        todo!()
    }
}
