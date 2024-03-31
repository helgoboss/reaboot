use tauri::async_runtime::Receiver;

use reaboot_core::api::{InstallerConfig, ReabootEvent};
use reaboot_core::installer::Installer;
use reaboot_core::reaboot_util;

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
            .emit_reaboot_event(ReabootEvent::InstallationStageChanged {
                stage: installation_stage,
            });
        Ok(())
    }

    async fn process_install(&self, config: InstallerConfig) -> anyhow::Result<()> {
        let listener = self.app_handle.clone();
        let installer = Installer::new(config, listener).await?;
        installer.install().await?;
        Ok(())
    }
}
