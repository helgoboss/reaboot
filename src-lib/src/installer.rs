use crate::api::{
    DownloadInfo, InstallationStatus, MultiDownloadInfo, ReabootConfig, Recipe,
    ResolvedReabootConfig,
};
use crate::downloader::{Download, DownloadProgress, Downloader};
use crate::multi_downloader::MultiDownloader;
use crate::package_installation_plan::{PackageInstallationPlan, QualifiedFile, QualifiedIndex};
use crate::reaper_resource_dir::ReaperResourceDir;
use crate::reaper_target::ReaperTarget;
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
use url::Url;

/// Responsible for orchestrating and carrying out the actual installation.
pub struct Installer<L> {
    downloader: Downloader,
    multi_downloader: MultiDownloader,
    temp_download_dir: PathBuf,
    temp_reaper_resource_dir: ReaperResourceDir<PathBuf>,
    final_reaper_resource_dir: ReaperResourceDir<PathBuf>,
    recipes: Vec<Recipe>,
    reaper_target: ReaperTarget,
    listener: L,
}

impl<L: InstallerListener> Installer<L> {
    /// Creates a new installer with all the values that stay the same throughout the complete
    /// installation process.
    pub async fn new(
        resolved_config: ResolvedReabootConfig,
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
            multi_downloader: MultiDownloader::new(downloader.clone(), concurrent_downloads),
            downloader,
            recipes,
            temp_download_dir,
            temp_reaper_resource_dir: ReaperResourceDir::new(temp_reaper_resource_dir),
            final_reaper_resource_dir: ReaperResourceDir::new(resolved_config.reaper_resource_dir),
            reaper_target: resolved_config.reaper_target,
            listener,
        };
        Ok(installer)
    }

    pub async fn install(&self) -> anyhow::Result<()> {
        // Create temporary directory structure
        // Determine initial installation status, so that we know where to start off
        let initial_installation_status = reaboot_util::determine_initial_installation_status(
            &self.final_reaper_resource_dir,
            self.reaper_target,
        );
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
        let downloaded_indexes = self.download_repository_indexes().await?;
        // Make package installation plan
        let plan = self.make_package_installation_plan(&downloaded_indexes);
        self.download_packages(plan.final_files).await;
        Ok(())
    }

    fn make_package_installation_plan<'a>(
        &'a self,
        downloaded_indexes: &'a [DownloadedIndex],
    ) -> PackageInstallationPlan {
        let qualified_indexes = downloaded_indexes.iter().map(|i| QualifiedIndex {
            url: &i.url,
            name: &i.name,
            index: &i.index,
        });
        PackageInstallationPlan::make(&self.recipes, qualified_indexes, self.reaper_target)
    }

    async fn download_reaper(&self) -> anyhow::Result<()> {
        // TODO
        let url = Url::parse("https://www.reaper.fm/files/7.x/reaper711_universal.dmg")?;
        let file = self.temp_download_dir.join("reaboot-reaper.dmg");
        self.listener
            .emit_installation_status(InstallationStatus::DownloadingReaper {
                download: DownloadInfo {
                    label: "bla".to_string(),
                    url: url.clone(),
                    file: file.clone(),
                },
            });
        let download = Download::new(url, file.clone());
        self.downloader
            .download(download, |s| {
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
        let asset =
            reapack_util::get_correct_reapack_asset(latest_release, self.reaper_target).await?;
        let file = self
            .temp_reaper_resource_dir
            .user_plugins()
            .join(asset.name);
        self.listener
            .emit_installation_status(InstallationStatus::DownloadingReaPack {
                download: DownloadInfo {
                    label: file.to_string_lossy().to_string(),
                    url: asset.url.clone(),
                    file: file.clone(),
                },
            });
        let download = Download::new(asset.url, file.clone());
        self.downloader
            .download(download, |s| {
                self.listener.emit_progress(s.to_simple_progress())
            })
            .await?;
        Ok(file)
    }

    async fn download_repository_indexes(&self) -> anyhow::Result<Vec<DownloadedIndex>> {
        let temp_cache_dir = self.temp_reaper_resource_dir.reapack_cache();
        let repository_urls: HashSet<_> = self
            .recipes
            .iter()
            .flat_map(|recipe| recipe.all_repository_urls())
            .collect();
        let downloads = repository_urls
            .into_iter()
            .enumerate()
            .map(|(i, url)| Download::new(url.clone(), temp_cache_dir.join(i.to_string())));
        let download_results = self
            .multi_downloader
            .download_multiple(
                downloads,
                |info| {
                    let installation_status =
                        InstallationStatus::DownloadingRepositoryIndexes { download: info };
                    self.listener.emit_installation_status(installation_status)
                },
                |progress| self.listener.emit_progress(progress),
            )
            .await;
        // TODO Emit new installation status
        let repos = download_results
            .into_iter()
            .flatten()
            .filter_map(|download| {
                let temp_file = std::fs::File::open(&download.file).ok()?;
                let index = Index::parse(BufReader::new(temp_file)).ok()?;
                let index_name = index.name.as_ref()?;
                let final_cache_file_name = format!("{index_name}.xml");
                let final_cache_file = download.file.with_file_name(final_cache_file_name);
                std::fs::rename(&download.file, final_cache_file).ok()?;
                let repo = DownloadedIndex {
                    url: download.url,
                    name: index_name.clone(),
                    index,
                    temp_download_file: download.file,
                };
                Some(repo)
            })
            .collect();
        Ok(repos)
    }

    async fn download_packages(&self, files: Vec<QualifiedFile<'_>>) {
        let downloads = files.into_iter().map(|file| Download {
            url: file.source.source.content.clone(),
            file: self.temp_reaper_resource_dir.get().join(file.relative_path),
        });
        let download_results = self
            .multi_downloader
            .download_multiple(
                downloads,
                |info| {
                    let installation_status =
                        InstallationStatus::DownloadingPackageFiles { download: info };
                    self.listener.emit_installation_status(installation_status)
                },
                |progress| self.listener.emit_progress(progress),
            )
            .await;
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

struct DownloadedIndex {
    url: Url,
    temp_download_file: PathBuf,
    name: String,
    index: Index,
}
