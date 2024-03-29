use reaboot_core::api::{ReabootConfig, ReabootEvent};
use reaboot_core::downloader::Downloader;
use reaboot_core::installer::{Installer, InstallerConfig};
use reaboot_core::reaboot_util;
use reaboot_reapack::model::VersionRef;
use tauri::async_runtime::Receiver;

use crate::app_handle::ReabootAppHandle;

pub struct ReabootWorker {
    receiver: Receiver<ReabootWorkerCommand>,
    app_handle: ReabootAppHandle,
}

pub enum ReabootWorkerCommand {
    EmitInitialInstallationEvents(ReabootConfig),
    Install(ReabootConfig),
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
                self.emit_initial_events(&config).await?;
            }
        }
        Ok(())
    }
    async fn emit_initial_events(&self, config: &ReabootConfig) -> anyhow::Result<()> {
        let resolved_config = reaboot_util::resolve_config(config)?;
        let installation_stage = reaboot_util::determine_initial_installation_stage(
            &resolved_config.reaper_resource_dir.clone().into(),
            resolved_config.reaper_target,
        )
        .await?;
        self.app_handle
            .emit_reaboot_event(ReabootEvent::ConfigResolved {
                config: resolved_config,
            });
        self.app_handle
            .emit_reaboot_event(ReabootEvent::InstallationStageChanged {
                stage: installation_stage,
            });
        Ok(())
    }

    async fn process_install(&self, config: ReabootConfig) -> anyhow::Result<()> {
        let downloader = Downloader::new(3);
        let resolved_config = reaboot_util::resolve_config(&config)?;
        let installer_config = InstallerConfig {
            resolved_config,
            package_urls: vec![],
            downloader,
            temp_parent_dir: None,
            keep_temp_dir: false,
            concurrent_downloads: 5,
            dry_run: false,
            listener: self.app_handle.clone(),
            reaper_version: VersionRef::Latest,
            skip_failed_packages: false,
        };
        let installer = Installer::new(installer_config).await?;
        installer.install().await?;
        Ok(())
    }
}
