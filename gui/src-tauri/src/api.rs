use reaboot_core::api::{
    InstallationStage, InstallerConfig, ReabootBackendInfo, ResolvedInstallerConfig,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A simple fire-and-forget command sent from the frontend to the backend.
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum ReabootCommand {
    /// Applies the given installer configuration.
    ConfigureInstallation { config: InstallerConfig },
    /// Starts the installation process.
    StartInstallation,
    /// Cancels the installation process.
    CancelInstallation,
}

/// Event emitted by the backend.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum ReabootEvent {
    Error {
        display_msg: String,
    },
    BackendInfoChanged {
        info: ReabootBackendInfo,
    },
    ConfigResolved {
        config: ResolvedInstallerConfig,
    },
    InstallationStageChanged {
        label: String,
        stage: InstallationStage,
    },
    TaskStarted {
        task_id: u32,
        label: String,
    },
    TaskProgressed {
        task_id: u32,
        progress: f64,
    },
    TaskFinished {
        task_id: u32,
    },
    InstallationReportReady {
        markdown: String,
    },
}

impl ReabootEvent {
    pub fn installation_stage_changed(stage: InstallationStage) -> Self {
        Self::InstallationStageChanged {
            label: stage.to_string(),
            stage,
        }
    }
}
