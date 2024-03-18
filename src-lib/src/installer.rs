use crate::api::{InstallationStatusEvent, RemoteFile};
use crate::downloader::{DownloadStatus, Downloader};
use anyhow::Context;
use octocrab::Octocrab;
use std::path::PathBuf;
use std::time::Duration;

pub struct Installer {
    downloader: Downloader,
    download_dir: PathBuf,
}

impl Installer {
    pub fn new(downloader: Downloader, download_dir: PathBuf) -> Self {
        Self {
            downloader,
            download_dir,
        }
    }

    pub async fn download_reaper(&self, listener: &impl InstallerListener) -> anyhow::Result<()> {
        let reaper_url = "https://www.reaper.fm/files/7.x/reaper711_universal.dmg";
        listener.emit_installation_status(InstallationStatusEvent::DownloadingReaper {
            file: RemoteFile {
                label: "bla".to_string(),
                url: reaper_url.to_string(),
            },
        });
        self.downloader
            .download(
                reaper_url,
                &self.download_dir.join("reaboot-reaper.dmg"),
                |s| listener.emit_download_status(s),
            )
            .await?;
        Ok(())
    }

    pub async fn download_reapack(&self, listener: &impl InstallerListener) -> anyhow::Result<()> {
        // TODO Initial reapack.ini:
        //  a) Set general/version to 4
        //     => no repo screen shown
        //     => no default repos will be added: we can add them on our own
        //  b) Don't set general/version
        //     => repo screen will be shown
        //     => default repos will be added (but *after* existing ones)
        //
        let octocrab = Octocrab::builder().build().unwrap();
        let latest_release = octocrab
            .repos("cfillion", "reapack")
            .releases()
            .get_latest()
            .await
            .context("Couldn't find latest ReaPack release")?;
        let asset = latest_release
            .assets
            .into_iter()
            .find(|asset| asset.name == "reaper_reapack-arm64.dylib")
            .context("Couldn't find macOS file for ReaPack")?;
        let reapack_url = asset.browser_download_url;
        listener.emit_installation_status(InstallationStatusEvent::DownloadingReaPack {
            file: RemoteFile {
                label: "bla".to_string(),
                url: reapack_url.to_string(),
            },
        });
        self.downloader
            .download(
                reapack_url,
                &self.download_dir.join("reaboot-reapack.dylib"),
                |s| listener.emit_download_status(s),
            )
            .await?;
        Ok(())
    }
}

pub async fn simulate_download(millis: u64, listener: &impl InstallerListener) {
    for i in (0..millis).step_by(2) {
        listener.emit_download_status(DownloadStatus::Downloading(i as f64 / millis as f64));
        sleep(1).await;
    }
}

pub trait InstallerListener {
    fn emit_installation_status(&self, event: InstallationStatusEvent);
    fn emit_download_status(&self, event: DownloadStatus);
}

async fn sleep(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await
}
