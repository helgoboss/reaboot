use crate::api::ReabootEvent;
use reaboot_core::api::{ConfirmationRequest, InstallationStage};
use reaboot_core::installer::{InstallerListener, InstallerTask};
use std::fmt::Display;
use tauri::{AppHandle, Emitter};

#[derive(Clone)]
pub struct ReabootAppHandle(AppHandle);

impl ReabootAppHandle {
    pub fn new(inner: AppHandle) -> Self {
        Self(inner)
    }

    pub fn emit_generic_error(&self, error: impl Display) {
        self.emit_reaboot_event(ReabootEvent::Error {
            display_msg: format!("{error:#}"),
        });
    }

    pub fn emit_reaboot_event(&self, evt: ReabootEvent) {
        self.0.emit("reaboot_event", evt).unwrap();
    }

    pub fn emit_progress(&self, progress: f64) {
        self.0.emit("reaboot_progress", progress).unwrap();
    }
}

impl InstallerListener for ReabootAppHandle {
    fn installation_stage_changed(&self, stage: InstallationStage) {
        self.emit_reaboot_event(ReabootEvent::installation_stage_changed(stage));
    }

    fn installation_stage_progressed(&self, progress: f64) {
        self.emit_progress(progress);
    }

    fn task_started(&self, task_id: u32, task: InstallerTask) {
        self.emit_reaboot_event(ReabootEvent::TaskStarted {
            task_id,
            label: task.label,
        });
    }

    fn task_progressed(&self, task_id: u32, progress: f64) {
        self.emit_reaboot_event(ReabootEvent::TaskProgressed { task_id, progress });
    }

    fn task_finished(&self, task_id: u32) {
        self.emit_reaboot_event(ReabootEvent::TaskFinished { task_id });
    }

    fn warn(&self, message: impl Display) {
        self.emit_reaboot_event(ReabootEvent::Warn {
            display_msg: message.to_string(),
        });
    }

    fn info(&self, message: impl Display) {
        self.emit_reaboot_event(ReabootEvent::Info {
            display_msg: message.to_string(),
        });
    }

    fn debug(&self, message: impl Display) {
        tracing::debug!(%message);
    }

    fn confirm(&self, request: ConfirmationRequest) {
        self.emit_reaboot_event(ReabootEvent::ConfirmationRequested { request });
    }
}
