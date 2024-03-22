use anyhow::{bail, Context};
use reaboot_lib::api::{InstallationStatus, ReabootConfig, Recipe, ResolvedReabootConfig};
use tauri::async_runtime::Receiver;
use tauri::Manager;

use reaboot_lib::downloader::Downloader;
use reaboot_lib::installer::Installer;
use reaboot_lib::reaboot_util;

use crate::app_handle::ReabootAppHandle;

pub struct ReabootWorker {
    receiver: Receiver<ReabootWorkerCommand>,
    app_handle: ReabootAppHandle,
}

pub enum ReabootWorkerCommand {
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
        }
        Ok(())
    }

    async fn process_install(&self, config: ReabootConfig) -> anyhow::Result<()> {
        let downloader = Downloader::new(3);
        let download_temp_dir =
            tempdir::TempDir::new("reaboot-").context("couldn't create temp directory")?;
        // TODO Take the first variant when done with debugging. This clears the temp dir after use.
        // let download_dir = download_temp_dir.path().to_path_buf()
        let download_dir = download_temp_dir.into_path();
        let resolved_config = reaboot_util::resolve_config(&config)?;
        let fixed_recipe: Recipe = serde_json::from_str(include_str!("branding/recipe.json"))
            .context("couldn't parse fixed recipe")?;
        let mut recipes = config.recipes;
        recipes.push(fixed_recipe);
        let installer = Installer::new(
            resolved_config.reaper_resource_dir,
            recipes,
            downloader,
            download_dir,
            5,
            self.app_handle.clone(),
        )
        .await?;
        installer.install().await?;
        Ok(())
    }
}
