use std::path::PathBuf;
use tauri::async_runtime::Receiver;

use reaboot_core::api::InstallerConfig;
use reaboot_core::installer::{InstallError, Installer};
use reaboot_core::recipe::Recipe;
use reaboot_core::{PreparationReportAsMarkdown, PreparationReportMarkdownOptions};

use crate::api::ReabootEvent;
use crate::app_handle::ReabootAppHandle;

pub struct ReabootWorker {
    receiver: Receiver<ReabootWorkerCommand>,
    app_handle: ReabootAppHandle,
}

pub enum ReabootWorkerCommand {
    Install {
        config: InstallerConfig,
        recipe: Option<Recipe>,
        temp_dir_for_reaper_download: PathBuf,
    },
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
            ReabootWorkerCommand::Install {
                config,
                recipe,
                temp_dir_for_reaper_download,
            } => {
                self.process_install(config, recipe, temp_dir_for_reaper_download)
                    .await?;
            }
        }
        Ok(())
    }

    async fn process_install(
        &self,
        config: InstallerConfig,
        recipe: Option<Recipe>,
        temp_dir_for_reaper_download: PathBuf,
    ) -> anyhow::Result<()> {
        let listener = self.app_handle.clone();
        let installer =
            Installer::new(config, recipe, temp_dir_for_reaper_download, listener).await?;
        let (report, actually_installed_things, manual_reaper_install_path) =
            match installer.install().await {
                Ok(outcome) => (
                    Some(outcome.preparation_report),
                    outcome.actually_installed_things,
                    outcome.manual_reaper_install_path,
                ),
                Err(e) => match e {
                    InstallError::SomePackagesFailed(report) => (Some(report), false, None),
                    InstallError::Other(_) => (None, false, None),
                },
            };
        let preparation_report_html = if let Some(r) = report {
            let options = PreparationReportMarkdownOptions {
                actually_installed_things,
                optimize_for_termimad: false,
            };
            let markdown = PreparationReportAsMarkdown::new(&r, options).to_string();
            let html = markdown::to_html_with_options(&markdown, &markdown::Options::gfm())
                .unwrap_or_default();
            Some(html)
        } else {
            None
        };
        let done_event = ReabootEvent::InstallationDone {
            preparation_report_html,
            manual_reaper_install_path,
        };
        self.app_handle.emit_reaboot_event(done_event);
        Ok(())
    }
}
