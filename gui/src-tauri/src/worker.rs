use markdown::{CompileOptions, ParseOptions};
use std::path::PathBuf;

use tauri::async_runtime::Receiver;

use reaboot_core::api::InstallerConfig;
use reaboot_core::installer::{InstallError, Installer, InstallerNewArgs};
use reaboot_core::{PreparationReportAsMarkdown, PreparationReportMarkdownOptions};

use crate::api::ReabootEvent;
use crate::app_handle::ReabootAppHandle;

pub struct ReabootWorker {
    command_receiver: Receiver<ReabootWorkerCommand>,
    app_handle: ReabootAppHandle,
    interaction_receiver: tokio::sync::broadcast::Receiver<bool>,
}

pub enum ReabootWorkerCommand {
    Install {
        config: InstallerConfig,
        temp_dir_for_reaper_download: PathBuf,
    },
}

impl ReabootWorker {
    pub fn new(
        command_receiver: Receiver<ReabootWorkerCommand>,
        app_handle: ReabootAppHandle,
        interaction_receiver: tokio::sync::broadcast::Receiver<bool>,
    ) -> Self {
        Self {
            command_receiver,
            app_handle,
            interaction_receiver,
        }
    }

    pub async fn keep_processing_incoming_commands(&mut self) {
        while let Some(command) = self.command_receiver.recv().await {
            if let Err(e) = self.process_command(command).await {
                self.app_handle.emit_generic_error(e);
            }
        }
    }

    async fn process_command(&self, command: ReabootWorkerCommand) -> anyhow::Result<()> {
        match command {
            ReabootWorkerCommand::Install {
                config,
                temp_dir_for_reaper_download,
            } => {
                self.process_install(config, temp_dir_for_reaper_download)
                    .await?;
            }
        }
        Ok(())
    }

    async fn process_install(
        &self,
        config: InstallerConfig,
        temp_dir_for_reaper_download: PathBuf,
    ) -> anyhow::Result<()> {
        let listener = self.app_handle.clone();
        let installer_new_args = InstallerNewArgs {
            config,
            temp_dir_for_reaper_download,
            interactions: self.interaction_receiver.resubscribe(),
            listener,
        };
        let installer = Installer::new(installer_new_args).await?;
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
        let (report_html, report_contains_donation_links) = if let Some(r) = report {
            let options = PreparationReportMarkdownOptions {
                include_main_heading: false,
                include_donation_links: true,
                actually_installed_things,
                optimize_for_termimad: false,
            };
            let markdown = PreparationReportAsMarkdown::new(&r, options).to_string();
            let options = markdown::Options {
                parse: ParseOptions::gfm(),
                compile: CompileOptions {
                    allow_dangerous_html: true,
                    ..CompileOptions::gfm()
                },
            };
            let html = markdown::to_html_with_options(&markdown, &options).unwrap_or_default();
            let contains_donation_links = r
                .package_preparation_outcomes
                .iter()
                .any(|o| o.donation_url.is_some());
            (Some(html), contains_donation_links)
        } else {
            (None, false)
        };
        let done_event = ReabootEvent::InstallationDone {
            report_html,
            report_contains_donation_links,
            manual_reaper_install_path,
        };
        self.app_handle.emit_reaboot_event(done_event);
        Ok(())
    }
}
