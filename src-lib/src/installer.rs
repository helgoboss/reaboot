use crate::api::{
    DownloadInfo, InstallationStatus, MultiDownloadInfo, ReabootConfig, Recipe,
    ResolvedReabootConfig,
};
use crate::downloader::{DownloadProgress, Downloader};
use crate::reaper_resource_dir::ReaperResourceDir;
use crate::task_tracker::{Summary, TaskTracker};
use crate::{reaboot_util, reapack_util};
use anyhow::{bail, Context};
use futures::{stream, StreamExt};
use reaboot_reapack::index::Index;
use std::collections::HashSet;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;

/// Responsible for orchestrating and carrying out the actual installation.
pub struct Installer<L> {
    downloader: Downloader,
    concurrent_downloads: usize,
    temp_download_dir: PathBuf,
    temp_reaper_resource_dir: ReaperResourceDir<PathBuf>,
    final_reaper_resource_dir: ReaperResourceDir<PathBuf>,
    recipes: Vec<Recipe>,
    listener: L,
}

impl<L: InstallerListener> Installer<L> {
    /// Creates a new installer with all the values that stay the same throughout the complete
    /// installation process.
    pub async fn new(
        reaper_resource_dir: PathBuf,
        recipes: Vec<Recipe>,
        downloader: Downloader,
        temp_download_dir: PathBuf,
        concurrent_downloads: usize,
        listener: L,
    ) -> anyhow::Result<Self> {
        tracing::debug!("Creating installer with temp download dir {temp_download_dir:?}");
        let temp_reaper_resource_dir = temp_download_dir.join("REAPER");
        fs::create_dir_all(&temp_reaper_resource_dir).await?;
        let installer = Self {
            downloader,
            concurrent_downloads,
            recipes,
            temp_download_dir,
            temp_reaper_resource_dir: ReaperResourceDir::new(temp_reaper_resource_dir),
            final_reaper_resource_dir: ReaperResourceDir::new(reaper_resource_dir),
            listener,
        };
        Ok(installer)
    }

    pub async fn install(&self) -> anyhow::Result<()> {
        // Create temporary directory structure
        // Determine initial installation status, so that we know where to start off
        let initial_installation_status =
            reaboot_util::determine_initial_installation_status(&self.final_reaper_resource_dir);
        // Download REAPER if necessary
        if initial_installation_status < InstallationStatus::InstalledReaper {
            bail!("installing REAPER automatically currently not supported");
            // installer.download_reaper(&self.app_handle).await?;
        }
        // Download ReaPack if necessary
        let reapack_file = if initial_installation_status < InstallationStatus::InstalledReaPack {
            Some(self.download_reapack().await?)
        } else {
            None
        };
        // Download repositories
        self.download_repositories().await?;
        Ok(())
    }

    async fn download_reaper(&self) -> anyhow::Result<()> {
        // TODO
        let url = "https://www.reaper.fm/files/7.x/reaper711_universal.dmg";
        let file = self.temp_download_dir.join("reaboot-reaper.dmg");
        self.listener
            .emit_installation_status(InstallationStatus::DownloadingReaper {
                download: DownloadInfo {
                    label: "bla".to_string(),
                    url: url.to_string(),
                    file: file.clone(),
                },
            });
        self.downloader
            .download(url, file.clone(), |s| {
                self.listener.emit_progress(s.to_simple_progress())
            })
            .await?;
        Ok(())
    }

    /// Downloads the ReaPack shared library to the temporary directory and returns the path to the
    /// downloaded file.
    async fn download_reapack(&self) -> anyhow::Result<PathBuf> {
        // TODO Initial reapack.ini:
        //  a) Set general/version to 4
        //     => no repo screen shown
        //     => no default repos will be added: we can add them on our own
        //  b) Don't set general/version
        //     => repo screen will be shown
        //     => default repos will be added (but *after* existing ones)
        //
        let latest_release = reapack_util::get_latest_reapack_release().await?;
        let asset = reapack_util::get_correct_reapack_asset(latest_release).await?;
        let file = self
            .temp_reaper_resource_dir
            .user_plugins()
            .join(asset.name);
        self.listener
            .emit_installation_status(InstallationStatus::DownloadingReaPack {
                download: DownloadInfo {
                    label: file.to_string_lossy().to_string(),
                    url: asset.url.to_string(),
                    file: file.clone(),
                },
            });
        self.downloader
            .download(asset.url, file.clone(), |s| {
                self.listener.emit_progress(s.to_simple_progress())
            })
            .await?;
        Ok(file)
    }

    async fn download_repositories(&self) -> anyhow::Result<Vec<ParsedRepository>> {
        let repository_urls: HashSet<_> = self
            .recipes
            .iter()
            .flat_map(|r| r.repository_urls().map(|u| u.to_string()))
            .collect();
        let temp_cache_dir = self.temp_reaper_resource_dir.reapack_cache();
        let (task_tracker, tasks) = TaskTracker::new(repository_urls);
        let progress_reporting_future = async move {
            loop {
                let summary = task_tracker.summary();
                let multi_download_info = MultiDownloadInfo {
                    in_progress_count: summary.in_progress_count,
                    success_count: summary.success_count,
                    error_count: summary.error_count,
                    total_count: summary.total_count,
                };
                let installation_status = InstallationStatus::DownloadingRepositories {
                    download: multi_download_info,
                };
                self.listener.emit_progress(summary.total_progress);
                self.listener.emit_installation_status(installation_status);
                if summary.done() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        };
        let download_future = async {
            let repos: Vec<_> = stream::iter(tasks)
                .enumerate()
                .map(|(i, task)| {
                    let url = task.payload;
                    let temp_download_file = temp_cache_dir.join(i.to_string());
                    async move {
                        task.record.start();
                        let download_result = self
                            .downloader
                            .download(&url, temp_download_file.clone(), |progress| {
                                task.record.set_progress(progress.to_simple_progress());
                            })
                            .await;
                        if download_result.is_ok() {
                            task.record.finish();
                        } else {
                            task.record.fail();
                        }
                        let repo = DownloadedRepository {
                            url,
                            temp_download_file,
                        };
                        Some(repo)
                    }
                })
                .buffer_unordered(self.concurrent_downloads)
                .filter_map(|downloaded_repo| async {
                    let downloaded_repo = downloaded_repo?;
                    let temp_file =
                        std::fs::File::open(&downloaded_repo.temp_download_file).ok()?;
                    let index = Index::parse(BufReader::new(temp_file)).ok()?;
                    let index_name = index.name.as_ref()?;
                    let final_cache_file_name = format!("{index_name}.xml");
                    let final_cache_file = downloaded_repo
                        .temp_download_file
                        .with_file_name(final_cache_file_name);
                    tokio::fs::rename(&downloaded_repo.temp_download_file, final_cache_file)
                        .await
                        .ok()?;
                    let repo = ParsedRepository {
                        url: downloaded_repo.url,
                        index,
                        temp_download_file: downloaded_repo.temp_download_file,
                    };
                    Some(repo)
                })
                .collect()
                .await;
            repos
        };
        let (_, repos) = futures::join!(progress_reporting_future, download_future);
        Ok(repos)
    }
}

async fn simulate_download(millis: u64, listener: &impl InstallerListener) {
    for i in (0..millis).step_by(2) {
        let download_progress = DownloadProgress::Downloading(i as f64 / millis as f64);
        listener.emit_progress(download_progress.to_simple_progress());
        sleep(1).await;
    }
}

pub trait InstallerListener {
    fn emit_installation_status(&self, event: InstallationStatus);
    fn emit_progress(&self, progress: f64);
}

async fn sleep(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await
}

struct DownloadedRepository {
    url: String,
    temp_download_file: PathBuf,
}

struct ParsedRepository {
    url: String,
    temp_download_file: PathBuf,
    index: Index,
}
