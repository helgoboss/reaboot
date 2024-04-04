use tauri::async_runtime::Receiver;

use reaboot_core::api::InstallerConfig;
use reaboot_core::installer::{InstallError, Installer};
use reaboot_core::{PreparationReportAsMarkdown, PreparationReportMarkdownOptions};

use crate::api::ReabootEvent;
use crate::app_handle::ReabootAppHandle;

pub struct ReabootWorker {
    receiver: Receiver<ReabootWorkerCommand>,
    app_handle: ReabootAppHandle,
}

pub enum ReabootWorkerCommand {
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
        }
        Ok(())
    }

    async fn process_install(&self, config: InstallerConfig) -> anyhow::Result<()> {
        let listener = self.app_handle.clone();
        let installer = Installer::new(config, listener).await?;
        let (report, packages_have_been_installed) = match installer.install().await {
            Ok(report) => (Some(report), true),
            Err(e) => match e {
                InstallError::SomePackagesFailed(report) => (Some(report), false),
                InstallError::Other(_) => (None, false),
            },
        };
        if let Some(r) = report {
            let options = PreparationReportMarkdownOptions {
                packages_have_been_installed,
                optimize_for_termimad: false,
            };
            let markdown = PreparationReportAsMarkdown::new(&r, options).to_string();
            let html = markdown::to_html_with_options(&markdown, &markdown::Options::gfm())
                .unwrap_or_default();
            self.app_handle
                .emit_reaboot_event(ReabootEvent::InstallationReportReady { html });
        }
        Ok(())
    }
}
