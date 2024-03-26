use crate::api::{
    DownloadInfo, InstallationStatus, MultiDownloadInfo, PackageInfo, ReabootConfig,
    ResolvedReabootConfig,
};
use crate::downloader::{Download, DownloadProgress, Downloader};
use crate::file_util::{find_first_existing_parent, get_first_existing_parent_dir};
use crate::multi_downloader::{
    DownloadError, DownloadResult, DownloadWithPayload, MultiDownloader,
};
use crate::package_application_plan::{PackageApplication, PackageApplicationPlan};
use crate::package_download_plan::{PackageDownloadPlan, QualifiedSource};
use crate::reaboot_util::resolve_config;
use crate::reaper_resource_dir::{
    ReaperResourceDir, REAPACK_INI_FILE_PATH, REAPACK_REGISTRY_DB_FILE_PATH,
};
use crate::reaper_target::ReaperTarget;
use crate::reaper_util::unpack_reaper_to_portable_dir;
use crate::task_tracker::{Summary, TaskTracker};
use crate::{reaboot_util, reapack_util, reaper_util};
use anyhow::{anyhow, bail, ensure, Context};
use enumset::EnumSet;
use fs_extra::dir::{CopyOptions, TransitProcessResult, TransitState};
use fs_extra::error::ErrorKind;
use futures::{stream, StreamExt};
use reaboot_reapack::database::Database;
use reaboot_reapack::index::{Index, IndexSection, NormalIndexSection};
use reaboot_reapack::model::{
    Config, InstalledFile, InstalledPackage, InstalledPackageType, InstalledVersionName,
    LightPackageId, PackageUrl, ParsePackageUrlError, Remote, Section, VersionRef,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;
use url::Url;

/// Responsible for orchestrating and carrying out the actual installation.
pub struct Installer<L> {
    downloader: Downloader,
    multi_downloader: MultiDownloader,
    temp_download_dir: PathBuf,
    temp_reaper_resource_dir: ReaperResourceDir,
    final_reaper_resource_dir: ReaperResourceDir,
    package_urls: Vec<PackageUrl>,
    reaper_target: ReaperTarget,
    dry_run: bool,
    listener: L,
    reaper_version: VersionRef,
    portable: bool,
}

pub struct InstallerConfig<L> {
    pub resolved_config: ResolvedReabootConfig,
    pub package_urls: Vec<Url>,
    pub downloader: Downloader,
    pub temp_download_dir: PathBuf,
    pub concurrent_downloads: u32,
    pub dry_run: bool,
    pub listener: L,
    pub reaper_version: VersionRef,
}

pub struct InstallationReport {}

impl<L: InstallerListener> Installer<L> {
    /// Creates a new installer with all the values that stay the same throughout the complete
    /// installation process.
    ///
    /// Creates a temporary directly already.
    pub fn new(config: InstallerConfig<L>) -> anyhow::Result<Self> {
        tracing::debug!(
            "Creating installer with temp download dir {:?}",
            &config.temp_download_dir
        );
        let temp_reaper_resource_dir = config.temp_download_dir.join("REAPER");
        fs::create_dir_all(&temp_reaper_resource_dir)?;
        let package_urls: Result<Vec<_>, ParsePackageUrlError> = config
            .package_urls
            .into_iter()
            .map(PackageUrl::parse_from_url)
            .collect();
        let installer = Self {
            multi_downloader: MultiDownloader::new(
                config.downloader.clone(),
                config.concurrent_downloads,
            ),
            downloader: config.downloader,
            package_urls: package_urls?,
            temp_download_dir: config.temp_download_dir,
            temp_reaper_resource_dir: ReaperResourceDir::new(temp_reaper_resource_dir)?,
            final_reaper_resource_dir: ReaperResourceDir::new(
                config.resolved_config.reaper_resource_dir,
            )?,
            reaper_target: config.resolved_config.reaper_target,
            listener: config.listener,
            dry_run: config.dry_run,
            reaper_version: config.reaper_version,
            portable: config.resolved_config.portable,
        };
        Ok(installer)
    }

    pub async fn install(&self) -> anyhow::Result<InstallationReport> {
        // Determine initial installation status, so that we know where to start off
        let initial_installation_status = reaboot_util::determine_initial_installation_status(
            &self.final_reaper_resource_dir,
            self.reaper_target,
        )
        .await?;
        // Download and extract REAPER if necessary
        let reaper_installation_outcome =
            if initial_installation_status < InstallationStatus::InstalledReaper {
                let download = self.download_reaper().await?;
                let outcome = self.extract_reaper(download).await;
                Some(outcome)
            } else {
                None
            };
        // Download ReaPack if necessary
        let reapack_download = if initial_installation_status < InstallationStatus::InstalledReaPack
        {
            Some(self.download_reapack().await?)
        } else {
            None
        };
        // Prepare temporary directory
        self.prepare_temp_dir()?;
        // Download repositories
        let downloaded_indexes = self.download_repository_indexes().await?;
        // Check which packages are installed already
        let package_status_quo = self
            .gather_already_installed_packages(&downloaded_indexes)
            .await?;
        // Make package download plan
        let download_plan = PackageDownloadPlan::make(
            &self.package_urls,
            &downloaded_indexes,
            &package_status_quo.installed_packages_to_keep,
            self.reaper_target,
        );
        // Download packages
        let package_download_results = self.download_packages(download_plan.final_files).await;
        // Weed out incomplete downloads
        let (successful_downloads, download_errors) =
            weed_out_download_errors(package_download_results);
        // Update ReaPack state
        let package_application_plan = self
            .update_reapack_state(
                &downloaded_indexes,
                successful_downloads,
                &package_status_quo.installed_packages_to_be_replaced,
            )
            .await?;
        // Actually apply the changes (by copying/moving all stuff to the destination dir)
        if !self.dry_run {
            // Apply ReaPack state
            // We do that *before* applying the packages. If something fails when
            // copying/moving the package files, the real ReaPack can still install the
            // packages via its synchronization feature.
            self.apply_reapack_state(&downloaded_indexes)
                .context("applying ReaPack state failed")?;
            // Apply packages
            self.apply_packages(package_application_plan)
                .context("applying packages failed")?;
        }
        // Create report
        let report = InstallationReport {};
        Ok(report)
    }

    async fn gather_already_installed_packages(
        &self,
        downloaded_indexes: &HashMap<Url, DownloadedIndex>,
    ) -> anyhow::Result<PackageStatusQuo> {
        let reapack_db_file = self.temp_reaper_resource_dir.reapack_registry_db_file();
        let quo = if reapack_db_file.exists() {
            self.gather_already_installed_packages_internal(&downloaded_indexes, &reapack_db_file)
                .await?
        } else {
            PackageStatusQuo::default()
        };
        Ok(quo)
    }

    fn prepare_temp_dir(&self) -> anyhow::Result<()> {
        self.listener
            .emit_installation_status(InstallationStatus::PreparingTempDirectory);
        self.copy_file_from_final_to_temp_dir_if_exists(REAPACK_INI_FILE_PATH)?;
        self.copy_file_from_final_to_temp_dir_if_exists(REAPACK_REGISTRY_DB_FILE_PATH)?;
        Ok(())
    }

    async fn update_reapack_state<'a>(
        &'a self,
        downloaded_indexes: &'a HashMap<Url, DownloadedIndex>,
        successful_downloads: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
        installed_packages_to_be_replaced: &'a [InstalledPackage],
    ) -> anyhow::Result<PackageApplicationPlan> {
        self.listener
            .emit_installation_status(InstallationStatus::UpdatingReaPackState);
        self.create_or_update_reapack_ini_file(downloaded_indexes)?;
        let plan = self
            .create_or_update_reapack_db(successful_downloads, installed_packages_to_be_replaced)
            .await?;
        Ok(plan)
    }

    async fn create_or_update_reapack_db<'a>(
        &'a self,
        successful_downloads: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
        installed_packages_to_be_replaced: &'a [InstalledPackage],
    ) -> anyhow::Result<PackageApplicationPlan> {
        // Create/migrate ReaPack database
        let reapack_db_file = self.temp_reaper_resource_dir.reapack_registry_db_file();
        if reapack_db_file.exists() {
            Database::open(reapack_db_file).await?.migrate().await?;
        } else {
            Database::create(reapack_db_file).await?;
        }
        // Update ReaPack database
        let application_plan = self
            .update_reapack_db(successful_downloads, installed_packages_to_be_replaced)
            .await?;
        Ok(application_plan)
    }

    fn create_or_update_reapack_ini_file(
        &self,
        downloaded_indexes: &HashMap<Url, DownloadedIndex>,
    ) -> anyhow::Result<()> {
        let reapack_ini_file = self.temp_reaper_resource_dir.reapack_ini_file();
        // Create/migrate config
        let mut config = if reapack_ini_file.exists() {
            let mut config = Config::load_from_ini_file(&reapack_ini_file)?;
            config.migrate();
            config
        } else {
            Config::default()
        };
        // Update config
        self.update_reapack_ini_file(&mut config, downloaded_indexes);
        // Apply to INI file on disk
        config.apply_to_ini_file(&reapack_ini_file)?;
        Ok(())
    }

    fn update_reapack_ini_file<'a>(
        &self,
        config: &mut Config,
        downloaded_indexes: &HashMap<Url, DownloadedIndex>,
    ) {
        for index in downloaded_indexes.values() {
            let remote = Remote {
                name: index.name.to_string(),
                url: index.url.clone(),
                enabled: true,
                auto_install: None,
            };
            config.add_remote(remote);
        }
    }

    async fn update_reapack_db<'a>(
        &'a self,
        downloads: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
        installed_packages_to_be_replaced: &'a [InstalledPackage],
    ) -> anyhow::Result<PackageApplicationPlan> {
        let mut replace_package_by_id: HashMap<_, _> = installed_packages_to_be_replaced
            .iter()
            .map(|p| (p.package_id(), p))
            .collect();
        let mut downloads_by_package: HashMap<LightPackageId, Vec<_>> = HashMap::new();
        for download in downloads {
            downloads_by_package
                .entry(download.payload.package_id())
                .or_default()
                .push(download);
        }
        let mut plan = PackageApplicationPlan::default();
        for (package_id, downloads) in downloads_by_package {
            let replace_package = replace_package_by_id.remove(&package_id);
            let install_result = self
                .update_reapack_db_for_one_package(package_id, &downloads, replace_package)
                .await;
            match install_result {
                Ok(_) => {
                    let package_move = PackageApplication {
                        package_id,
                        to_be_moved: downloads,
                        to_be_removed: replace_package,
                    };
                    plan.package_applications.push(package_move);
                }
                Err(e) => {
                    tracing::warn!(msg = "Couldn't update ReaPack DB for one package", ?e)
                }
            }
        }
        Ok(plan)
    }

    async fn update_reapack_db_for_one_package<'a>(
        &self,
        package_id: LightPackageId<'a>,
        downloads: &[DownloadWithPayload<QualifiedSource<'a>>],
        replace_package: Option<&InstalledPackage>,
    ) -> anyhow::Result<()> {
        // Prepare data to be inserted into DB
        let first_download = downloads
            .first()
            .context("Package has no downloads. Shouldn't happen at this point.")?;
        let first_version = first_download.payload.version;
        let description = first_version
            .package
            .package
            .desc
            .as_ref()
            .cloned()
            .unwrap_or_default();
        let installed_package = InstalledPackage {
            remote: package_id.remote.to_string(),
            category: package_id.category.to_string(),
            package: package_id.package.to_string(),
            desc: description,
            // ReaBoot weeds out packages from installation that have an unknown type
            // (because it wouldn't know where to put the files then). So we are in the nice
            // position to have a known package type here.
            typ: InstalledPackageType::Known(first_download.payload.version.package.typ),
            // ReaBoot refuses to parse an index that contains an invalid version name. So we
            // are in the nice position to have a valid version name at this point.
            version: InstalledVersionName::Valid(first_version.version.name.clone()),
            author: first_version.version.author.clone().unwrap_or_default(),
            files: downloads
                .iter()
                .map(|download| InstalledFile {
                    path: download.payload.relative_path.clone(),
                    sections: convert_index_section_to_model(&download.payload.source.main),
                    // See package type comment
                    typ: download.payload.typ.map(InstalledPackageType::Known),
                })
                .collect(),
        };
        // Dry-remove files, dry-remove package, add package to DB and dry-move/copy files ...
        // all within one database transaction. Our atomic unit at this stage is a package.
        self.listener
            .log_write_activity(format!("Installing package {}...", package_id.package));
        let mut db = self.open_temp_reapack_db().await?;
        let transaction = db.with_transaction(|mut transaction| async {
            if let Some(p) = replace_package {
                // Dry remove installed package files
                for file in &p.files {
                    let path = self.final_reaper_resource_dir.get().join(&file.path);
                    dry_remove_file(&path)?;
                }
                // Remove package from DB
                transaction.remove_package(package_id).await?;
            }
            // Add package to DB
            transaction.add_package(installed_package).await?;
            // Dry copy/move
            for download in downloads {
                let src_file = &download.download.file;
                let dest_file = self
                    .final_reaper_resource_dir
                    .join(&download.payload.relative_path);
                dry_move_file(src_file, dest_file)?;
            }
            Ok(transaction)
        });
        transaction.await?;
        Ok(())
    }

    fn apply_reapack_state<'a>(
        &self,
        downloaded_indexes: &HashMap<Url, DownloadedIndex>,
    ) -> anyhow::Result<()> {
        self.listener
            .emit_installation_status(InstallationStatus::ApplyingReaPackState);
        self.move_file(
            self.temp_reaper_resource_dir.reapack_ini_file(),
            self.final_reaper_resource_dir.reapack_ini_file(),
            0,
            2,
            true,
        )
        .context("moving ReaPack INI file failed")?;
        // TODO We must make sure that the DB file was written, otherwise we end up with an empty one
        self.move_file(
            self.temp_reaper_resource_dir.reapack_registry_db_file(),
            self.final_reaper_resource_dir.reapack_registry_db_file(),
            1,
            2,
            true,
        )
        .context("moving ReaPack registry DB file failed")?;
        let dest_cache_dir = self.final_reaper_resource_dir.reapack_cache_dir();
        let total_count = downloaded_indexes.len();
        for (i, index) in downloaded_indexes.values().enumerate() {
            let src_index_file_name = index
                .temp_download_file
                .file_name()
                .context("ReaPack index file should have a name at this point")?;
            let dest_index_file = dest_cache_dir.join(src_index_file_name);
            self.move_file(
                &index.temp_download_file,
                dest_index_file,
                i,
                total_count,
                true,
            )
            .context("moving cached ReaPack repository index failed")?;
        }
        Ok(())
    }

    fn apply_packages<'a>(&self, plan: PackageApplicationPlan) -> anyhow::Result<()> {
        for package_application in plan.package_applications {
            self.apply_package(package_application)?;
        }
        Ok(())
    }

    fn apply_package(&self, package_application: PackageApplication) -> anyhow::Result<()> {
        self.listener
            .emit_installation_status(InstallationStatus::ApplyingPackage {
                package: PackageInfo {
                    name: package_application.package_id.package.to_string(),
                },
            });
        if let Some(p) = package_application.to_be_removed {
            // Remove files
            for file in &p.files {
                let path = self.final_reaper_resource_dir.get().join(&file.path);
                std::fs::remove_file(path)
                    .context("couldn't remove file of package to be replaced")?;
            }
        }
        // Copy/move
        let total_file_count = package_application.to_be_moved.len();
        for (i, download) in package_application.to_be_moved.into_iter().enumerate() {
            let src_file = download.download.file;
            let dest_file = self
                .final_reaper_resource_dir
                .get()
                .join(download.payload.relative_path);
            self.move_file(&src_file, &dest_file, i, total_file_count, false)?;
        }
        Ok(())
    }

    async fn gather_already_installed_packages_internal(
        &self,
        downloaded_indexes: &HashMap<Url, DownloadedIndex>,
        reapack_db_file: &Path,
    ) -> anyhow::Result<PackageStatusQuo> {
        let mut db = Database::open(reapack_db_file).await?;
        let already_installed_packages = db.installed_packages().await?;
        let package_ids_to_be_installed: HashSet<LightPackageId> = self
            .package_urls
            .iter()
            .filter_map(|purl| {
                let downloaded_index = downloaded_indexes.get(&purl.repository_url())?;
                let package_path = purl.package_version_ref().package_path();
                let package_id = LightPackageId {
                    remote: &downloaded_index.name,
                    category: package_path.category(),
                    package: package_path.package_name(),
                };
                Some(package_id)
            })
            .collect();
        let (installed_packages_to_be_replaced, installed_packages_to_keep) =
            already_installed_packages
                .into_iter()
                .partition(|p| package_ids_to_be_installed.contains(&p.package_id()));
        let quo = PackageStatusQuo {
            installed_packages_to_be_replaced,
            installed_packages_to_keep,
        };
        Ok(quo)
    }

    async fn open_temp_reapack_db(&self) -> anyhow::Result<Database> {
        Database::open(self.temp_reaper_resource_dir.reapack_registry_db_file()).await
    }

    async fn download_reaper(&self) -> anyhow::Result<Download> {
        self.listener
            .emit_installation_status(InstallationStatus::CheckingLatestReaperVersion);
        let installer_asset = reaper_util::get_latest_reaper_installer_asset(
            self.reaper_target,
            &self.reaper_version,
        )
        .await?;
        let file = self.temp_download_dir.join(installer_asset.file_name);
        self.listener
            .emit_installation_status(InstallationStatus::DownloadingReaper {
                download: DownloadInfo {
                    label: installer_asset.version.to_string(),
                    url: installer_asset.url.clone(),
                    file: file.clone(),
                },
            });
        let download = Download::new(installer_asset.url, file.clone());
        self.downloader
            .download(download.clone(), |s| {
                self.listener.emit_progress(s.to_simple_progress())
            })
            .await?;
        Ok(download)
    }

    async fn extract_reaper(
        &self,
        reaper_download: Download,
    ) -> anyhow::Result<ReaperInstallationOutcome> {
        if !self.portable {
            let outcome =
                ReaperInstallationOutcome::InstallManuallyBecauseNoPortableInstall(reaper_download);
            return Ok(outcome);
        }
        self.listener
            .emit_installation_status(InstallationStatus::ExtractingReaper);
        // Because we support automatic REAPER installation only for portable installs,
        // the REAPER resource dir is at the same time the base directory, that's the one which
        // includes the executable (or bundle on macOS).
        let reaper_base_dir = self.temp_reaper_resource_dir.get();
        let result = unpack_reaper_to_portable_dir(
            &reaper_download.file,
            reaper_base_dir,
            &self.temp_download_dir,
        );
        if let Err(e) = result.await {
            return Ok(ReaperInstallationOutcome::InstallManuallyBecauseError(e));
        }
        fs::File::create(reaper_base_dir.join("reaper.ini"))?;
        Ok(ReaperInstallationOutcome::UnpackedToTempReaperResourceDir)
    }

    /// Downloads the ReaPack shared library to the temporary directory and returns the path to the
    /// downloaded file.
    async fn download_reapack(&self) -> anyhow::Result<Download> {
        let latest_release = reapack_util::get_latest_reapack_release().await?;
        let asset =
            reapack_util::get_correct_reapack_asset(latest_release, self.reaper_target).await?;
        let file = self
            .temp_reaper_resource_dir
            .user_plugins_dir()
            .join(asset.name);
        let download_url = asset.browser_download_url;
        self.listener
            .emit_installation_status(InstallationStatus::DownloadingReaPack {
                download: DownloadInfo {
                    label: file.to_string_lossy().to_string(),
                    url: download_url.clone(),
                    file: file.clone(),
                },
            });
        let download = Download::new(download_url, file.clone());
        self.downloader
            .download(download.clone(), |s| {
                self.listener.emit_progress(s.to_simple_progress())
            })
            .await?;
        Ok(download)
    }

    async fn download_repository_indexes(&self) -> anyhow::Result<HashMap<Url, DownloadedIndex>> {
        let temp_cache_dir = self.temp_reaper_resource_dir.reapack_cache_dir();
        let repository_urls: HashSet<_> = self
            .package_urls
            .iter()
            .map(|purl| purl.repository_url())
            .collect();
        let downloads = repository_urls.into_iter().enumerate().map(|(i, url)| {
            DownloadWithPayload::new(
                Download::new(url.clone(), temp_cache_dir.join(i.to_string())),
                (),
            )
        });
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
        // Parse
        self.listener
            .emit_installation_status(InstallationStatus::ParsingRepositoryIndexes);
        let mut index_names_so_far = HashSet::new();
        let repos = download_results
            .into_iter()
            .flatten()
            .filter_map(|mut download| {
                let temp_file = std::fs::File::open(&download.download.file).ok()?;
                let index = Index::parse(BufReader::new(temp_file))
                    .inspect_err(|e| {
                        tracing::warn!(msg = "Couldn't parse index", ?e);
                    })
                    .ok()?;
                let index_name = index.name.as_ref()?;
                if !index_names_so_far.insert(index_name.clone()) {
                    // Another index was downloaded with the same index name. The first one wins!
                    tracing::warn!(
                        msg = "Encountered multiple URLs returning index with same name",
                        index_name,
                        ignored_url = %download.download.url
                    );
                    return None;
                }
                let final_cache_file_name = format!("{index_name}.xml");
                let final_cache_file = download.download.file.with_file_name(final_cache_file_name);
                std::fs::rename(&download.download.file, &final_cache_file).ok()?;
                download.download.file = final_cache_file;
                let repo = DownloadedIndex {
                    url: download.download.url.clone(),
                    name: index_name.clone(),
                    index,
                    temp_download_file: download.download.file,
                };
                Some((download.download.url, repo))
            })
            .collect();
        Ok(repos)
    }

    async fn download_packages<'a>(
        &'a self,
        files: Vec<QualifiedSource<'a>>,
    ) -> Vec<DownloadResult<QualifiedSource<'a>>> {
        let downloads = files.into_iter().map(|file| DownloadWithPayload {
            download: Download {
                url: file.source.content.clone(),
                file: self
                    .temp_reaper_resource_dir
                    .get()
                    .join(&file.relative_path),
            },
            payload: file,
        });
        self.multi_downloader
            .download_multiple(
                downloads,
                |info| {
                    let installation_status =
                        InstallationStatus::DownloadingPackageFiles { download: info };
                    self.listener.emit_installation_status(installation_status)
                },
                |progress| self.listener.emit_progress(progress),
            )
            .await
    }

    fn copy_file_from_final_to_temp_dir_if_exists(
        &self,
        rel_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let file_in_final_dir = self.final_reaper_resource_dir.join(&rel_path);
        if file_in_final_dir.exists() {
            let file_in_temp_dir = self.temp_reaper_resource_dir.join(&rel_path);
            create_parent_dirs(&file_in_temp_dir)?;
            fs::copy(&file_in_final_dir, &file_in_temp_dir)?;
        }
        Ok(())
    }

    fn move_file(
        &self,
        src_file: impl AsRef<Path>,
        dest_file: impl AsRef<Path>,
        index: usize,
        count: usize,
        overwrite: bool,
    ) -> anyhow::Result<()> {
        create_parent_dirs(&dest_file)?;
        let options = fs_extra::file::CopyOptions {
            overwrite,
            skip_exist: false,
            buffer_size: 0,
        };
        let result =
            fs_extra::file::move_file_with_progress(src_file, &dest_file, &options, |transit| {
                let file_total_bytes = transit.total_bytes as f64;
                let file_progress = if file_total_bytes == 0.0 {
                    1.0
                } else {
                    transit.copied_bytes as f64 / file_total_bytes
                };
                let total_progress = (index as f64 / count as f64) * file_progress;
                self.listener.emit_progress(total_progress);
            });
        result.map_err(|e| anyhow!("Error when moving file: {:?} - {}", e.kind, e))?;
        Ok(())
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
    fn log_write_activity(&self, activity: String);
}

async fn sleep(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await
}

pub struct DownloadedIndex {
    pub url: Url,
    pub temp_download_file: PathBuf,
    /// This name is the actual repository ID (not the URL)! Multiple URLs can point to the same
    /// logical index.
    pub name: String,
    pub index: Index,
}

fn weed_out_download_errors(
    package_download_results: Vec<DownloadResult<QualifiedSource>>,
) -> (
    Vec<DownloadWithPayload<QualifiedSource>>,
    Vec<DownloadError<QualifiedSource>>,
) {
    let mut download_errors = vec![];
    let mut failed_package_ids = HashSet::new();
    let successful_downloads: Vec<_> = package_download_results
        .into_iter()
        .filter_map(|r| {
            match r {
                Ok(d) => {
                    let package_id = d.payload.package_id();
                    if failed_package_ids.contains(&package_id) {
                        // Another file of the same package was not downloaded correctly.
                        // Skip install of this package. We don't want an incomplete installation!
                        return None;
                    }
                    Some(d)
                }
                Err(e) => {
                    failed_package_ids.insert(e.download.payload.package_id());
                    download_errors.push(e);
                    None
                }
            }
        })
        .collect();
    (successful_downloads, download_errors)
}

/// ReaBoot *doesn't* weed out packages from installation just because
/// it encounters an unknown section string. That would be too brutal.
/// Instead, it just collects the sections it understands (and knows how
/// to convert them to the integer in the DB). The consequence is that the
/// actions won't show up in the unknown sections, a minor issue.
fn convert_index_section_to_model(index_section: &IndexSection) -> Option<EnumSet<Section>> {
    match index_section {
        IndexSection::Implicit => None,
        IndexSection::Normal(sections) => {
            // Insert all known sections, ignore the rest
            let mut enum_set = EnumSet::new();
            for section in sections {
                if let NormalIndexSection::Known(s) = section {
                    enum_set.insert(*s);
                }
            }
            Some(enum_set)
        }
    }
}

enum ReaperInstallationOutcome {
    /// It's a main REAPER install, and we don't support automatic REAPER installation for that.
    InstallManuallyBecauseNoPortableInstall(Download),
    /// It's a portable REAPER install but there was an error extracting the executables.
    InstallManuallyBecauseError(anyhow::Error),
    /// It's a portable install and the REAPER executable has successfully been placed
    /// in the temporary REAPER resource directory.
    UnpackedToTempReaperResourceDir,
}

#[derive(Default)]
struct PackageStatusQuo {
    installed_packages_to_be_replaced: Vec<InstalledPackage>,
    installed_packages_to_keep: Vec<InstalledPackage>,
}

fn dry_remove_file(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        if fs::metadata(&path)?.permissions().readonly() {
            bail!("Removing the file would not work");
        }
    }
    Ok(())
}

fn dry_move_file(src: &Path, dest: PathBuf) -> anyhow::Result<()> {
    ensure!(src.exists(), "Source file doesn't exist");
    ensure!(!dest.exists(), "Destination file exists already");
    let first_existing_parent = get_first_existing_parent_dir(dest.clone())?;
    let readonly = fs::metadata(first_existing_parent)?
        .permissions()
        .readonly();
    ensure!(!readonly, "Parent of destination file is not writable");
    Ok(())
}

fn create_parent_dirs(path: impl AsRef<Path>) -> anyhow::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}