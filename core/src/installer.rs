use crate::api::{
    DownloadInfo, InstallationStage, InstallerConfig, MultiDownloadInfo, PackageInfo,
};
use crate::downloader::{Download, Downloader};
use crate::file_util::{
    create_parent_dirs, existing_file_or_dir_is_writable, file_or_dir_is_writable_or_creatable,
    get_first_existing_parent_dir, move_dir_contents, move_file, move_file_overwriting_with_backup,
};
use crate::installation_model::{
    determine_files_to_be_downloaded, PackageInstallationPlan, QualifiedSource, TempInstallFailure,
};
use crate::multi_downloader::{
    DownloadError, DownloadResult, DownloadWithPayload, MultiDownloader,
};
use crate::preparation_report::PreparationReport;

use crate::reaper_platform::ReaperPlatform;
use crate::reaper_resource_dir::{
    ReaperResourceDir, REAPACK_INI_FILE_PATH, REAPACK_REGISTRY_DB_FILE_PATH,
};
use crate::reaper_util::extract_reaper_to_portable_dir;
use crate::task_tracker::{TaskSummary, TaskTrackerListener};
use crate::{reaboot_util, reapack_util, reaper_util, ToolDownload, ToolingChange};
use anyhow::{bail, ensure, Context};
use enumset::EnumSet;
use reaboot_reapack::database::Database;
use reaboot_reapack::index::{Index, IndexSection, NormalIndexSection};
use reaboot_reapack::model::{
    Config, InstalledFile, InstalledPackage, InstalledPackageType, InstalledVersionName,
    LightPackageId, LightVersionId, PackageUrl, ParsePackageUrlError, Remote, Section, VersionRef,
};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use std::fs;
use std::future::Future;
use std::io::BufReader;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tempdir::TempDir;
use thiserror::Error;
use url::Url;

/// Responsible for orchestrating and carrying out the actual installation.
pub struct Installer<L> {
    downloader: Downloader,
    multi_downloader: MultiDownloader,
    temp_dir: PathBuf,
    temp_reaper_resource_dir: ReaperResourceDir,
    dest_reaper_resource_dir: ReaperResourceDir,
    package_urls: Vec<PackageUrl>,
    platform: ReaperPlatform,
    dry_run: bool,
    listener: L,
    reaper_version: VersionRef,
    portable: bool,
    skip_failed_packages: bool,
    /// This is set if [`InstallerConfig::keep_temp_dir`] is `false`. It will cause
    /// the temporary directory to be removed latest on drop.
    ///
    /// It's wrapped in a mutex because we might decide in the middle of the installation process
    /// that it's better to keep the temporary directory.
    temp_dir_guard: Mutex<Option<TempDir>>,
}

impl<L: InstallerListener> Installer<L> {
    /// Creates a new installer with all the values that stay the same throughout the complete
    /// installation process.
    ///
    /// Creates a temporary directly already.
    pub async fn new(config: InstallerConfig, listener: L) -> anyhow::Result<Self> {
        let config = reaboot_util::resolve_config(config)?;
        // Do some early sanity checks
        let dest_reapack_db_file = config.reaper_resource_dir.reapack_registry_db_file();
        if dest_reapack_db_file.exists() {
            Database::open(dest_reapack_db_file).await.context("ReaPack database is currently busy. Please close REAPER and/or stop existing ReaBoot processes and try again!")?;
        }
        if !config.dry_run {
            let resource_dir = config.reaper_resource_dir.get();
            ensure!(file_or_dir_is_writable_or_creatable(resource_dir), "REAPER resource directory {resource_dir:?} is read-only. Are you trying to write into a system directory? REAPER resource directories are usually accessible without root/admin privileges.");
        }
        // Create some directories
        fs::create_dir_all(&config.temp_parent_dir)?;
        let temp_dir_guard = TempDir::new_in(&config.temp_parent_dir, "reaboot-")
            .context("couldn't create temp directory")?;
        let (temp_dir, temp_dir_guard) = if config.keep_temp_dir {
            (temp_dir_guard.into_path(), None)
        } else {
            (temp_dir_guard.path().to_path_buf(), Some(temp_dir_guard))
        };
        let temp_reaper_resource_dir = temp_dir.join("REAPER");
        fs::create_dir_all(&temp_reaper_resource_dir)?;
        let package_urls: Result<Vec<_>, ParsePackageUrlError> = config
            .package_urls
            .into_iter()
            .map(PackageUrl::parse_from_url)
            .collect();
        let installer = Self {
            multi_downloader: MultiDownloader::new(
                Downloader::new(config.num_download_retries),
                config.concurrent_downloads,
            ),
            downloader: Downloader::new(config.num_download_retries),
            package_urls: package_urls?,
            temp_dir,
            temp_reaper_resource_dir: ReaperResourceDir::new(temp_reaper_resource_dir)?,
            dest_reaper_resource_dir: config.reaper_resource_dir,
            platform: config.platform,
            listener,
            dry_run: config.dry_run,
            skip_failed_packages: config.skip_failed_packages,
            reaper_version: config.reaper_version,
            portable: config.portable,
            temp_dir_guard: Mutex::new(temp_dir_guard),
        };
        Ok(installer)
    }

    /// Returns whether ReaBoot is capable of installing REAPER automatically.
    pub fn reaper_is_installable(&self) -> bool {
        self.portable
    }

    pub async fn determine_initial_installation_stage(&self) -> anyhow::Result<InstallationStage> {
        reaboot_util::determine_initial_installation_stage(
            &self.dest_reaper_resource_dir,
            self.platform,
        )
        .await
    }

    pub async fn install(mut self) -> Result<PreparationReport, InstallError> {
        let result = self.install_internal().await;
        let final_stage = match &result {
            Ok(_) => InstallationStage::Finished,
            Err(e) => InstallationStage::Failed {
                display_msg: e.to_string(),
            },
        };
        self.listener.installation_stage_changed(final_stage);
        self.clean_up();
        result
    }

    async fn install_internal(&mut self) -> Result<PreparationReport, InstallError> {
        // Determine initial installation status, so that we know where to start off
        let initial_installation_stage = self.determine_initial_installation_stage().await?;
        // Download and extract REAPER if necessary
        let reaper_installation_outcome =
            if initial_installation_stage < InstallationStage::InstalledReaper {
                let download = self.download_reaper().await?;
                let outcome = self.extract_reaper(download).await?;
                Some(outcome)
            } else {
                None
            };
        // Download ReaPack if necessary
        let reapack_download = if initial_installation_stage < InstallationStage::InstalledReaPack {
            Some(self.download_reapack().await?)
        } else {
            None
        };
        // Prepare temporary directory
        self.prepare_temp_dir()?;
        // Download repository indexes
        let downloaded_indexes = self.download_repository_indexes().await?;
        // Check which packages are installed already
        let package_status_quo = self
            .gather_already_installed_packages(&downloaded_indexes)
            .await?;
        // Determine files to be downloaded, weeding out pre-download failures
        let (files_to_be_downloaded, pre_download_failures) = determine_files_to_be_downloaded(
            &self.package_urls,
            &downloaded_indexes,
            &package_status_quo.installed_packages_to_keep,
            self.platform,
        );
        // Download packages
        let package_download_results = self.download_packages(files_to_be_downloaded).await;
        // Weed out incomplete downloads
        let (successful_downloads, download_errors) =
            weed_out_download_errors(package_download_results);
        // Update ReaPack state
        let (package_installation_plans, temp_install_failures) = self
            .update_reapack_state(
                &downloaded_indexes,
                successful_downloads,
                &package_status_quo.installed_packages_to_be_replaced,
            )
            .await?;
        // Create preparation report
        let mut tooling_changes = vec![];
        if let Some(ReaperInstallationOutcome::ExtractedSuccessfully { download, .. }) =
            reaper_installation_outcome.as_ref()
        {
            tooling_changes.push(ToolingChange::new("REAPER".to_string(), download.clone()));
        }
        if let Some(download) = reapack_download.as_ref() {
            tooling_changes.push(ToolingChange::new("ReaPack".to_string(), download.clone()));
        }
        let preparation_report = PreparationReport::new(
            tooling_changes,
            pre_download_failures,
            download_errors,
            temp_install_failures,
            &package_installation_plans,
        );
        if self.dry_run {
            return Ok(preparation_report);
        }
        if !self.skip_failed_packages && preparation_report.summary().failures > 0 {
            return Err(InstallError::SomePackagesFailed(preparation_report));
        }
        // Actually apply the changes (by copying/moving all stuff to the destination dir)
        // Apply REAPER
        self.apply_reaper(&reaper_installation_outcome)
            .context("installing REAPER failed")?;
        // Apply ReaPack lib
        self.apply_reapack_lib(reapack_download.as_ref())
            .context("installing ReaPack library failed")?;
        // Apply ReaPack state
        // We do that *before* applying the packages. If something fails when
        // copying/moving the package files, the real ReaPack can still install the
        // packages via its synchronization feature.
        self.apply_reapack_state(&downloaded_indexes)
            .context("updating ReaPack state failed")?;
        // Apply packages
        self.apply_packages(package_installation_plans)
            .context("moving packages failed")?;
        Ok(preparation_report)
    }

    fn clean_up(self) {
        let _ = self.take_temp_dir_guard_if_existing();
        // If the user didn't provide a custom temp parent dir, it's "REAPER_RESOURCE_DIR/ReaBoot".
        // This directory is going to be empty if the user or installer didn't decide to keep it.
        // In this case, we just delete it in order to not leave traces.
        let _ = fs::remove_dir(self.dest_reaper_resource_dir.temp_reaboot_dir());
    }

    fn apply_reapack_lib(&self, download: Option<&ToolDownload>) -> anyhow::Result<()> {
        if let Some(download) = download {
            let file_name = download
                .download
                .file
                .file_name()
                .context("ReaPack lib file should have name")?;
            let dest_file = self
                .dest_reaper_resource_dir
                .user_plugins_dir()
                .join(file_name);
            move_file(&download.download.file, dest_file, true)?;
        }
        Ok(())
    }

    fn apply_reaper(
        &self,
        reaper_installation_outcome: &Option<ReaperInstallationOutcome>,
    ) -> anyhow::Result<()> {
        if let Some(outcome) = &reaper_installation_outcome {
            match outcome {
                ReaperInstallationOutcome::InstallManuallyBecauseNoPortableInstall(d) => {
                    self.keep_temp_directory();
                    self.listener.warn(format!("ReaBoot can install REAPER for you, but this currently only works for portable installations. Please install it manually. The installer is located here: {:?}", &d.download.file));
                }
                ReaperInstallationOutcome::InstallManuallyBecauseError(download, error) => {
                    self.keep_temp_directory();
                    self.listener.warn(format!("ReaBoot tried to install REAPER for you, but there was an error. Please install it manually. The installer is located here: {:?}\n\nError: {:#}", &download.download.file, &error));
                }
                ReaperInstallationOutcome::ExtractedSuccessfully {
                    dir_containing_reaper,
                    ..
                } => {
                    move_dir_contents(dir_containing_reaper, self.dest_reaper_resource_dir.get())?;
                    // Copy reaper.ini file, but only if it doesn't exist already. E.g. the silent Windows installation
                    // already contains such a file with some minimal content.
                    let _ = move_file(
                        self.temp_reaper_resource_dir.reaper_ini_file(),
                        self.dest_reaper_resource_dir.reaper_ini_file(),
                        false,
                    );
                }
            }
        }
        Ok(())
    }

    /// Makes sure the temp directory is kept, even if the user didn't choose manually to keep it.
    fn keep_temp_directory(&self) {
        if let Some(temp_dir) = self.take_temp_dir_guard_if_existing() {
            // This drops the temp dir struct without deleting the temporary directory
            temp_dir.into_path();
        }
    }

    /// This explicitly drops the temp dir guard if one exists.
    ///
    /// If the return value is just dropped, this will effectively remove the temp dir.
    #[must_use]
    fn take_temp_dir_guard_if_existing(&self) -> Option<TempDir> {
        self.temp_dir_guard.lock().unwrap().take()
    }

    async fn gather_already_installed_packages(
        &self,
        downloaded_indexes: &HashMap<Url, DownloadedIndex>,
    ) -> anyhow::Result<PackageStatusQuo> {
        let reapack_db_file = self.temp_reaper_resource_dir.reapack_registry_db_file();
        let quo = if reapack_db_file.exists() {
            self.gather_already_installed_packages_internal(downloaded_indexes, &reapack_db_file)
                .await?
        } else {
            PackageStatusQuo::default()
        };
        Ok(quo)
    }

    fn prepare_temp_dir(&self) -> anyhow::Result<()> {
        self.listener
            .installation_stage_changed(InstallationStage::PreparingTempDirectory);
        self.copy_file_from_final_to_temp_dir_if_exists(REAPACK_INI_FILE_PATH)?;
        self.copy_file_from_final_to_temp_dir_if_exists(REAPACK_REGISTRY_DB_FILE_PATH)?;
        Ok(())
    }

    async fn update_reapack_state<'a>(
        &'a self,
        downloaded_indexes: &'a HashMap<Url, DownloadedIndex>,
        successful_downloads: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
        installed_packages_to_be_replaced: &'a [InstalledPackage],
    ) -> anyhow::Result<(
        Vec<PackageInstallationPlan<'a>>,
        Vec<TempInstallFailure<'a>>,
    )> {
        self.listener
            .installation_stage_changed(InstallationStage::UpdatingReaPackState);
        self.create_or_update_reapack_ini_file(downloaded_indexes)?;
        let outcome = self
            .create_or_update_reapack_db(successful_downloads, installed_packages_to_be_replaced)
            .await?;
        Ok((outcome.package_installation_plans, outcome.package_failures))
    }

    async fn create_or_update_reapack_db<'a>(
        &'a self,
        successful_downloads: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
        installed_packages_to_be_replaced: &'a [InstalledPackage],
    ) -> anyhow::Result<TempInstallOutcome> {
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

    fn update_reapack_ini_file(
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
    ) -> anyhow::Result<TempInstallOutcome> {
        let mut replace_package_by_id: HashMap<_, _> = installed_packages_to_be_replaced
            .iter()
            .map(|p| (p.package_id(), p))
            .collect();
        let mut downloads_by_version: HashMap<LightVersionId, Vec<_>> = HashMap::new();
        for download in downloads {
            downloads_by_version
                .entry(download.payload.version.id())
                .or_default()
                .push(download);
        }
        let mut outcome = TempInstallOutcome::default();
        let mut db = self.open_temp_reapack_db().await?;
        for (version_id, downloads) in downloads_by_version {
            let replace_package = replace_package_by_id.remove(&version_id.package_id);
            let install_result = self
                .update_reapack_db_for_one_package(&mut db, version_id, &downloads, replace_package)
                .await;
            match install_result {
                Ok(_) => {
                    let plan = PackageInstallationPlan {
                        version_id,
                        to_be_moved: downloads,
                        to_be_removed: replace_package,
                    };
                    outcome.package_installation_plans.push(plan);
                }
                Err(error) => {
                    self.listener.warn(format!(
                        "Couldn't update ReaPack DB for package {version_id} because {error}"
                    ));
                    let failure = TempInstallFailure { version_id, error };
                    outcome.package_failures.push(failure);
                }
            }
        }
        db.close().await?;
        Ok(outcome)
    }

    async fn update_reapack_db_for_one_package<'a>(
        &self,
        db: &mut Database,
        version_id: LightVersionId<'a>,
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
        let package_id = version_id.package_id;
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
        let transaction = db.with_transaction(|mut transaction| async {
            let mut relative_paths_to_be_replaced = HashSet::new();
            if let Some(p) = replace_package {
                // Dry remove installed package files
                for file in &p.files {
                    relative_paths_to_be_replaced.insert(&file.path);
                    let absolute_path = self.dest_reaper_resource_dir.get().join(&file.path);
                    dry_remove_file(&absolute_path)?;
                }
                // Remove package from DB
                self.listener.info(format!("Remove package {p} from DB..."));
                transaction.remove_package(package_id).await?;
            }
            // Add package to DB
            self.listener
                .info(format!("Add package {} to DB...", version_id));
            transaction.add_package(installed_package).await?;
            // Dry copy/move
            for download in downloads {
                if relative_paths_to_be_replaced.contains(&download.payload.relative_path) {
                    // Dry-moving this file would fail because there's still another ReaPack
                    // managed file at this location belonging to this package. We are going to
                    // remove it eventually when applying the package, so that's fine.
                    continue;
                }
                let src_file = &download.download.file;
                let dest_file = self
                    .dest_reaper_resource_dir
                    .join(&download.payload.relative_path);
                dry_move_file(src_file, dest_file)?;
            }
            Ok(transaction)
        });
        transaction.await?;
        Ok(())
    }

    fn apply_reapack_state(
        &self,
        downloaded_indexes: &HashMap<Url, DownloadedIndex>,
    ) -> anyhow::Result<()> {
        tracing::debug!("Applying ReaPack state");
        self.listener
            .installation_stage_changed(InstallationStage::ApplyingReaPackState);
        move_file_overwriting_with_backup(
            self.temp_reaper_resource_dir.reapack_ini_file(),
            self.dest_reaper_resource_dir.reapack_ini_file(),
        )
        .context("moving ReaPack INI file failed")?;
        move_file_overwriting_with_backup(
            self.temp_reaper_resource_dir.reapack_registry_db_file(),
            self.dest_reaper_resource_dir.reapack_registry_db_file(),
        )
        .context("moving ReaPack registry DB file failed")?;
        let dest_cache_dir = self.dest_reaper_resource_dir.reapack_cache_dir();
        for index in downloaded_indexes.values() {
            let src_index_file_name = index
                .temp_download_file
                .file_name()
                .context("ReaPack index file should have a name at this point")?;
            let dest_index_file = dest_cache_dir.join(src_index_file_name);
            move_file_overwriting_with_backup(&index.temp_download_file, dest_index_file)
                .context("moving cached ReaPack repository index failed")?;
        }
        Ok(())
    }

    fn apply_packages(&self, plans: Vec<PackageInstallationPlan>) -> anyhow::Result<()> {
        for plan in plans {
            self.apply_package(plan)?;
        }
        Ok(())
    }

    fn apply_package(&self, plan: PackageInstallationPlan) -> anyhow::Result<()> {
        self.listener
            .installation_stage_changed(InstallationStage::ApplyingPackage {
                package: PackageInfo {
                    name: plan.version_id.package_id.package.to_string(),
                },
            });
        if let Some(p) = plan.to_be_removed {
            // Remove files
            self.listener
                .info(format!("Deleting files of existing package {p}"));
            for file in &p.files {
                let path = self.dest_reaper_resource_dir.get().join(&file.path);
                fs::remove_file(path).context("couldn't remove file of package to be replaced")?;
            }
        }
        // Copy/move
        self.listener
            .info(format!("Moving files of new package {}", &plan.version_id));
        for download in plan.to_be_moved.into_iter() {
            let src_file = download.download.file;
            let dest_file = self
                .dest_reaper_resource_dir
                .get()
                .join(download.payload.relative_path);
            move_file(&src_file, &dest_file, false)?;
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
                let downloaded_index = downloaded_indexes.get(purl.repository_url())?;
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

    async fn download_reaper(&self) -> anyhow::Result<ToolDownload> {
        self.listener
            .installation_stage_changed(InstallationStage::CheckingLatestReaperVersion);
        let installer_asset =
            reaper_util::get_latest_reaper_installer_asset(self.platform, &self.reaper_version)
                .await?;
        let file = self.temp_dir.join(installer_asset.file_name);
        let version = installer_asset.version;
        self.listener
            .installation_stage_changed(InstallationStage::DownloadingReaper {
                download: DownloadInfo {
                    label: version.to_string(),
                    url: installer_asset.url.clone(),
                    file: file.clone(),
                },
            });
        let download = Download::new(
            format!("REAPER {version}"),
            installer_asset.url,
            file.clone(),
            None,
        );
        self.downloader
            .download(download.clone(), |s| {
                self.listener
                    .installation_stage_progressed(s.to_simple_progress())
            })
            .await?;
        let tool_download = ToolDownload {
            version: version.to_string(),
            download,
        };
        Ok(tool_download)
    }

    async fn extract_reaper(
        &self,
        reaper_download: ToolDownload,
    ) -> anyhow::Result<ReaperInstallationOutcome> {
        if !self.portable {
            let outcome =
                ReaperInstallationOutcome::InstallManuallyBecauseNoPortableInstall(reaper_download);
            return Ok(outcome);
        }
        self.listener
            .installation_stage_changed(InstallationStage::ExtractingReaper);
        fs::File::create(self.temp_reaper_resource_dir.reaper_ini_file())?;
        let reaper_dest_dir = self.temp_dir.join("reaper-binaries");
        let result = extract_reaper_to_portable_dir(
            &reaper_download.download.file,
            &reaper_dest_dir,
            &self.temp_dir,
        );
        let outcome = if let Err(error) = result {
            ReaperInstallationOutcome::InstallManuallyBecauseError(reaper_download, error)
        } else {
            ReaperInstallationOutcome::ExtractedSuccessfully {
                download: reaper_download,
                dir_containing_reaper: reaper_dest_dir,
            }
        };
        Ok(outcome)
    }

    /// Downloads the ReaPack shared library to the temporary directory and returns the path to the
    /// downloaded file.
    async fn download_reapack(&self) -> anyhow::Result<ToolDownload> {
        let latest_release = reapack_util::get_latest_reapack_release().await?;
        let tag_name = latest_release.tag_name.clone();
        let asset = reapack_util::get_correct_reapack_asset(latest_release, self.platform).await?;
        let file = self
            .temp_reaper_resource_dir
            .user_plugins_dir()
            .join(asset.name);
        let download_url = asset.browser_download_url;
        self.listener
            .installation_stage_changed(InstallationStage::DownloadingReaPack {
                download: DownloadInfo {
                    label: tag_name.clone(),
                    url: download_url.clone(),
                    file: file.clone(),
                },
            });
        let download = Download::new(
            format!("ReaPack {tag_name}"),
            download_url,
            file.clone(),
            None,
        );
        self.downloader
            .download(download.clone(), |s| {
                self.listener
                    .installation_stage_progressed(s.to_simple_progress())
            })
            .await?;
        let outcome = ToolDownload {
            version: tag_name,
            download,
        };
        Ok(outcome)
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
                Download::new(
                    url.to_string(),
                    url.clone(),
                    temp_cache_dir.join(i.to_string()),
                    None,
                ),
                (),
            )
        });
        let multi_download_listener = MultiDownloadListener::new(self, |info| {
            InstallationStage::DownloadingRepositoryIndexes { download: info }
        });
        let download_results = self
            .multi_downloader
            .download_multiple(downloads, multi_download_listener)
            .await;
        // Parse
        self.listener
            .installation_stage_changed(InstallationStage::ParsingRepositoryIndexes);
        let mut index_names_so_far = HashSet::new();
        let repos = download_results
            .into_iter()
            .flatten()
            .filter_map(|mut download| {
                let temp_file = fs::File::open(&download.download.file).ok()?;
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
                fs::rename(&download.download.file, &final_cache_file).ok()?;
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
                label: file
                    .source
                    .file
                    .clone()
                    .unwrap_or_else(|| file.version.package.package.name.clone()),
                url: file.source.content.clone(),
                file: self
                    .temp_reaper_resource_dir
                    .get()
                    .join(&file.relative_path),
                expected_multihash: file.source.hash.clone(),
            },
            payload: file,
        });
        let multi_download_listener = MultiDownloadListener::new(self, |info| {
            InstallationStage::DownloadingPackageFiles { download: info }
        });
        self.multi_downloader
            .download_multiple(downloads, multi_download_listener)
            .await
    }

    fn copy_file_from_final_to_temp_dir_if_exists(
        &self,
        rel_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let file_in_final_dir = self.dest_reaper_resource_dir.join(&rel_path);
        if file_in_final_dir.exists() {
            let file_in_temp_dir = self.temp_reaper_resource_dir.join(&rel_path);
            create_parent_dirs(&file_in_temp_dir)?;
            fs::copy(&file_in_final_dir, &file_in_temp_dir)?;
        }
        Ok(())
    }
}

pub trait InstallerListener {
    /// Called when the current stage of the installation process has changed.
    fn installation_stage_changed(&self, event: InstallationStage);

    /// Called when progress has been made within the installation stage.
    ///
    /// `progress` is a number between 0 and 1, representing 0% to 100%.
    fn installation_stage_progressed(&self, progress: f64);

    /// Called when a concurrent task within the installation stage has started.
    fn task_started(&self, task_id: u32, task: InstallerTask);

    /// Just like [`Self::installation_stage_progressed`] but for a specific task.
    fn task_progressed(&self, task_id: u32, progress: f64);

    /// Called when a concurrent task within the installation stage has finished.
    fn task_finished(&self, task_id: u32);

    fn warn(&self, message: impl Display);

    fn info(&self, message: impl Display);

    fn debug(&self, message: impl Display);
}

pub struct InstallerTask {
    pub label: String,
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
    InstallManuallyBecauseNoPortableInstall(ToolDownload),
    /// It's a portable REAPER install but there was an error extracting the executables.
    InstallManuallyBecauseError(ToolDownload, anyhow::Error),
    /// REAPER has been extracted successfully.
    ExtractedSuccessfully {
        download: ToolDownload,
        /// It's a portable install and the REAPER executable along with other files (Windows & Linux) or the bundle dir
        /// (macOS) has been placed **into** the given directory.
        dir_containing_reaper: PathBuf,
    },
}

#[derive(Default)]
struct PackageStatusQuo {
    installed_packages_to_be_replaced: Vec<InstalledPackage>,
    installed_packages_to_keep: Vec<InstalledPackage>,
}

fn dry_remove_file(path: &Path) -> anyhow::Result<()> {
    if path.exists() && !existing_file_or_dir_is_writable(path) {
        let suffix = if cfg!(windows) {
            " Is REAPER running already? If yes, exit REAPER and try again."
        } else {
            ""
        };
        bail!("Removing the file would not work.{suffix}");
    }
    Ok(())
}

fn dry_move_file(src: &Path, dest: PathBuf) -> anyhow::Result<()> {
    ensure!(src.exists(), "Source file {src:?} doesn't exist");
    ensure!(!dest.exists(), "Destination file {dest:?} exists already");
    let first_existing_parent = get_first_existing_parent_dir(dest.clone())?;
    let writable = existing_file_or_dir_is_writable(&first_existing_parent);
    ensure!(
        writable,
        "Parent of destination file {dest:?} is not writable"
    );
    Ok(())
}

struct MultiDownloadListener<'a, P, L, C> {
    installer: &'a Installer<L>,
    create_installation_stage: C,
    _p: PhantomData<P>,
}

impl<'a, P, L, C> MultiDownloadListener<'a, P, L, C> {
    pub fn new(installer: &'a Installer<L>, create_installation_stage: C) -> Self {
        Self {
            installer,
            create_installation_stage,
            _p: Default::default(),
        }
    }
}

impl<'a, P, L, C> TaskTrackerListener for MultiDownloadListener<'a, P, L, C>
where
    C: Fn(MultiDownloadInfo) -> InstallationStage,
    L: InstallerListener,
{
    type Payload = DownloadWithPayload<P>;

    fn summary_changed(&self, summary: TaskSummary) {
        let multi_download_info = MultiDownloadInfo {
            in_progress_count: summary.in_progress_count,
            success_count: summary.success_count,
            error_count: summary.error_count,
            total_count: summary.total_count,
        };
        let installation_stage = (self.create_installation_stage)(multi_download_info);
        self.installer
            .listener
            .installation_stage_changed(installation_stage);
    }

    fn total_progressed(&self, progress: f64) {
        self.installer
            .listener
            .installation_stage_progressed(progress);
    }

    fn task_started(&self, task_index: usize, payload: &Self::Payload) {
        self.installer.listener.task_started(
            task_index as u32,
            InstallerTask {
                label: payload.download.label.clone(),
            },
        );
    }

    fn task_progressed(&self, task_index: usize, progress: f64) {
        self.installer
            .listener
            .task_progressed(task_index as u32, progress);
    }

    fn task_finished(&self, task_index: usize) {
        self.installer.listener.task_finished(task_index as u32);
    }
}

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("some packages failed")]
    SomePackagesFailed(PreparationReport),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Default)]
struct TempInstallOutcome<'a> {
    package_failures: Vec<TempInstallFailure<'a>>,
    package_installation_plans: Vec<PackageInstallationPlan<'a>>,
}
