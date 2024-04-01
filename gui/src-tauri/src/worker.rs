use tauri::async_runtime::Receiver;

use crate::api::ReabootEvent;
use reaboot_core::api::InstallerConfig;
use reaboot_core::installer::{InstallError, Installer};
use reaboot_core::{reaboot_util, PreparationReport, PreparationReportAsMarkdown};

use crate::app_handle::ReabootAppHandle;

pub struct ReabootWorker {
    receiver: Receiver<ReabootWorkerCommand>,
    app_handle: ReabootAppHandle,
}

pub enum ReabootWorkerCommand {
    EmitInitialInstallationEvents(InstallerConfig),
    Install(InstallerConfig),
}

impl ReabootWorker {
    pub fn new(receiver: Receiver<ReabootWorkerCommand>, app_handle: ReabootAppHandle) -> Self {
        Self {
            receiver,
            app_handle,
        }
    }

    pub async fn keep_processing_incoming_commands(&mut self) {
        while let Some(command) = self.receiver.recv().await {
            if let Err(e) = self.process_command(command).await {
                self.app_handle.emit_generic_error(e);
            }
        }
    }

    async fn process_command(&self, command: ReabootWorkerCommand) -> anyhow::Result<()> {
        match command {
            ReabootWorkerCommand::Install(config) => {
                self.process_install(config).await?;
            }
            ReabootWorkerCommand::EmitInitialInstallationEvents(config) => {
                self.emit_initial_events(config).await?;
            }
        }
        Ok(())
    }

    async fn emit_initial_events(&self, config: InstallerConfig) -> anyhow::Result<()> {
        let backend_info = reaboot_util::collect_backend_info();
        let config = reaboot_util::resolve_config(config)?;
        let installation_stage = reaboot_util::determine_initial_installation_stage(
            &config.reaper_resource_dir,
            config.platform,
        )
        .await?;
        self.app_handle
            .emit_reaboot_event(ReabootEvent::BackendInfoChanged { info: backend_info });
        self.app_handle
            .emit_reaboot_event(ReabootEvent::ConfigResolved { config });
        self.app_handle
            .emit_reaboot_event(ReabootEvent::installation_stage_changed(installation_stage));
        Ok(())
    }

    async fn process_install(&self, config: InstallerConfig) -> anyhow::Result<()> {
        let listener = self.app_handle.clone();
        let installer = Installer::new(config, listener).await?;
        let (report, packages_have_been_installed) = match installer.install().await {
            Ok(report) => (Some(report), true),
            Err(e) => match e {
                InstallError::SomePackagesFailed(report) => (Some(report), false),
                InstallError::Other(e) => (None, false),
            },
        };
        if let Some(r) = report {
            let markdown =
                PreparationReportAsMarkdown::new(&r, packages_have_been_installed).to_string();
            self.app_handle
                .emit_reaboot_event(ReabootEvent::InstallationReportReady { markdown });
        }
        Ok(())
    }
}
