use crate::api::{
    ConfirmationRequest, DownloadInfo, InstallationStage, InstallerConfig, MultiDownloadInfo,
    PackageInfo, ResolvedInstallerConfig,
};
use crate::downloader::{Download, Downloader};
use crate::file_util::{
    create_parent_dirs, existing_file_or_dir_is_writable, file_or_dir_is_writable_or_creatable,
    get_first_existing_parent_dir, move_dir_contents, move_file, move_file_overwriting_with_backup,
};
use crate::installation_model::{
    make_first_plan, QualifiedSource, SinglePackageInstallationPlan, TempInstallFailure,
};
use crate::multi_downloader::{
    DownloadError, DownloadResult, DownloadWithPayload, MultiDownloader,
};
use crate::preparation_report::PreparationReport;

use crate::reaper_resource_dir::{
    ReaperResourceDir, REAPACK_INI_FILE_PATH, REAPACK_REGISTRY_DB_FILE_PATH,
};
use crate::reaper_util::{extract_reaper_to_dir, ReaperInstallerAsset};
use crate::task_tracker::{TaskSummary, TaskTrackerListener};
use crate::{reaboot_util, reaper_util, ToolDownload, ToolingChange};
use anyhow::{bail, ensure, Context, Error};
use enumset::EnumSet;
use reaboot_reapack::database::{Database, DatabaseTransaction};
use reaboot_reapack::index::{Index, IndexSection, NormalIndexSection};
use reaboot_reapack::model::{
    Config, InstalledFile, InstalledPackage, InstalledPackageType, InstalledVersionName,
    LightPackageId, LightVersionId, Remote, Section,
};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};

use std::fs;
use std::io::BufReader;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use tempdir::TempDir;
use thiserror::Error;
use tokio::sync::broadcast::Receiver;
use url::Url;

const REABOOT_TEMP_DIR_PREFIX: &str = "reaboot-";

/// Responsible for orchestrating and carrying out the actual installation.
pub struct Installer<L> {
    resolved_config: ResolvedInstallerConfig,
    downloader: Downloader,
    multi_downloader: MultiDownloader,
    /// Temporary directory used for most of the downloads.
    ///
    /// - The parent of this temp dir is user-customizable.
    /// - It's possible to keep it after running.
    /// - By default, this ends up in the final REAPER resource directory ("ReaBoot" dir).
    temp_dir: PathBuf,
    temp_reaper_resource_dir: ReaperResourceDir,
    /// A temporary directory just for REAPER download.
    ///
    /// The consumer must decide how long-lived this is (important for potential manual REAPER
    /// install).
    temp_dir_for_reaper_download: PathBuf,
    interactions: Receiver<bool>,
    listener: L,
    /// This will cause the temporary directory to be removed latest on drop.
    _temp_dir_guard: Option<TempDir>,
}

pub struct InstallerNewArgs<L: InstallerListener> {
    pub config: InstallerConfig,
    pub temp_dir_for_reaper_download: PathBuf,
    pub interactions: Receiver<bool>,
    pub listener: L,
}

impl<L: InstallerListener> Installer<L> {
    /// Creates a new installer with all the values that stay the same throughout the complete
    /// installation process.
    ///
    /// Creates a temporary directly already.
    pub async fn new(args: InstallerNewArgs<L>) -> anyhow::Result<Self> {
        let resolved_config = reaboot_util::resolve_config(args.config).await?;
        // Do some early sanity checks
        let dest_reapack_db_file = resolved_config
            .reaper_resource_dir
            .reapack_registry_db_file();
        if dest_reapack_db_file.exists() {
            Database::open(dest_reapack_db_file).await.context("ReaPack database is currently busy. Please close REAPER and/or stop existing ReaBoot processes and try again!")?;
        }
        if !resolved_config.dry_run {
            let resource_dir = resolved_config.reaper_resource_dir.get();
            ensure!(file_or_dir_is_writable_or_creatable(resource_dir), "REAPER resource directory {resource_dir:?} is read-only. Are you trying to write into a system directory? REAPER resource directories are usually accessible without root/admin privileges.");
        }
        // Create some directories
        fs::create_dir_all(&resolved_config.temp_parent_dir)?;
        let temp_dir_guard =
            TempDir::new_in(&resolved_config.temp_parent_dir, REABOOT_TEMP_DIR_PREFIX)
                .context("couldn't create temp directory")?;
        let (temp_dir, temp_dir_guard) = if resolved_config.keep_temp_dir {
            (temp_dir_guard.into_path(), None)
        } else {
            (temp_dir_guard.path().to_path_buf(), Some(temp_dir_guard))
        };
        let temp_reaper_resource_dir = temp_dir.join("REAPER");
        fs::create_dir_all(&temp_reaper_resource_dir)?;
        let installer = Self {
            multi_downloader: MultiDownloader::new(
                Downloader::new(resolved_config.num_download_retries),
                resolved_config.concurrent_downloads,
            ),
            downloader: Downloader::new(resolved_config.num_download_retries),
            temp_dir,
            temp_reaper_resource_dir: ReaperResourceDir::new(temp_reaper_resource_dir)?,
            temp_dir_for_reaper_download: args.temp_dir_for_reaper_download,
            interactions: args.interactions,
            listener: args.listener,
            _temp_dir_guard: temp_dir_guard,
            resolved_config,
        };
        Ok(installer)
    }

    pub fn resolved_config(&self) -> &ResolvedInstallerConfig {
        &self.resolved_config
    }

    pub async fn install(mut self) -> Result<InstallationOutcome, InstallError> {
        let result = self.install_internal().await;
        let final_stage = match &result {
            Ok(_) => InstallationStage::Finished,
            Err(e) => InstallationStage::Failed {
                display_msg: format!("{e:#}"),
            },
        };
        self.listener.installation_stage_changed(final_stage);
        self.clean_up();
        result
    }

    async fn install_internal(&mut self) -> Result<InstallationOutcome, InstallError> {
        // Download and extract REAPER if necessary
        let reaper_preparation_outcome = self.download_and_prepare_reaper_if_necessary().await?;
        // Prepare temporary directory
        self.prepare_temp_dir()?;
        // Download repository indexes
        let downloaded_indexes = self.download_repository_indexes().await?;
        // Check which packages are installed already
        let package_status_quo = self
            .gather_already_installed_packages(&downloaded_indexes)
            .await?;
        // Determine files to be downloaded, weeding out pre-download failures
        let first_plan = make_first_plan(
            &self.resolved_config.package_urls,
            &downloaded_indexes,
            &package_status_quo.installed_packages_to_keep,
            self.resolved_config.platform,
        );
        // Download packages
        let mut interactions = self.interactions.resubscribe();
        let package_download_results = self
            .download_packages(first_plan.files_to_be_downloaded)
            .await;
        // Weed out incomplete downloads
        let (successful_downloads, download_errors) =
            weed_out_download_errors(package_download_results);
        // Prepare and simulate installation
        let mut num_tries = 0;
        let (package_installation_plans, temp_install_failures) = loop {
            let enough_tries = num_tries >= 2;
            let simulation_future = self.prepare_and_simulate_installation(
                &downloaded_indexes,
                successful_downloads.clone(),
                &package_status_quo.installed_packages_to_be_replaced,
                &first_plan.installed_packages_to_be_removed,
            );
            match simulation_future.await {
                Ok((plans, failures)) => {
                    if failures.is_empty() || enough_tries {
                        break (plans, failures);
                    } else {
                        num_tries += 1;
                        self.prompt_user_to_exit_reaper(&mut interactions).await;
                    }
                }
                Err(error) => {
                    if enough_tries {
                        return Err(error.into());
                    }
                    num_tries += 1;
                    self.prompt_user_to_exit_reaper(&mut interactions).await;
                }
            }
        };
        // Create preparation report
        let mut tooling_changes = vec![];
        if let Some(
            ReaperPreparationOutcome::InstallByMovingFromExtractedReaper { download, .. }
            | ReaperPreparationOutcome::InstallWindowsMainViaInstaller(download),
        ) = reaper_preparation_outcome.as_ref()
        {
            tooling_changes.push(ToolingChange::new("REAPER".to_string(), download.clone()));
        }
        let preparation_report = PreparationReport::new(
            tooling_changes,
            first_plan.pre_download_failures,
            download_errors,
            temp_install_failures,
            &first_plan.installed_packages_to_be_removed,
            &package_installation_plans,
        );
        if self.resolved_config.dry_run {
            let outcome = InstallationOutcome {
                preparation_report,
                actually_installed_things: false,
                manual_reaper_install_path: None,
            };
            return Ok(outcome);
        }
        if !self.resolved_config.skip_failed_packages && preparation_report.summary().failures > 0 {
            return Err(InstallError::SomePackagesFailed(preparation_report));
        }
        // Actually apply/install the changes (by copying/moving all stuff to the destination dir)
        // Install REAPER
        let manual_reaper_install_path = if let Some(o) = &reaper_preparation_outcome {
            self.install_reaper(o)
        } else {
            None
        };
        // Apply ReaPack state
        // We do that *before* applying the packages. If something fails when
        // copying/moving the package files, the real ReaPack can still install the
        // packages via its synchronization feature.
        self.apply_reapack_state(&downloaded_indexes)
            .context("applying ReaPack state failed")?;
        // Apply packages
        self.install_packages(
            &first_plan.installed_packages_to_be_removed,
            package_installation_plans,
        )
        .context("moving packages failed")?;
        // Build final outcome
        let outcome = InstallationOutcome {
            preparation_report,
            actually_installed_things: true,
            manual_reaper_install_path,
        };
        Ok(outcome)
    }

    async fn prompt_user_to_exit_reaper(&self, interactions: &mut Receiver<bool>) {
        let confirmation_request = ConfirmationRequest {
            message: "It looks like REAPER is currently running. If it is, please close it before pressing \"Continue\"!".to_string(),
            yes_label: "Continue".to_string(),
            no_label: None,
        };
        self.listener.confirm(confirmation_request);
        let _ = interactions.recv().await;
    }

    fn clean_up(self) {
        // If the user didn't provide a custom temp parent dir, it's "REAPER_RESOURCE_DIR/ReaBoot".
        // This directory is going to be empty if the user or installer didn't decide to keep it.
        // In this case, we just delete it in order to not leave traces.
        let _ = fs::remove_dir(self.resolved_config.reaper_resource_dir.temp_reaboot_dir());
    }

    /// Returns path to REAPER installer in order to request manual installation.
    fn install_reaper(&self, preparation_outcome: &ReaperPreparationOutcome) -> Option<PathBuf> {
        self.listener
            .installation_stage_changed(InstallationStage::InstallingReaper);
        match preparation_outcome {
            ReaperPreparationOutcome::InstallManually(d) => Some(d.download.file.clone()),
            ReaperPreparationOutcome::InstallManuallyDueToError(download, error) => {
                self.install_manually_due_to_error(download, error)
            }
            ReaperPreparationOutcome::InstallWindowsMainViaInstaller(download) => {
                if let Err(e) =
                    reaper_util::install_reaper_for_windows_main(&download.download.file)
                {
                    self.install_manually_due_to_error(download, &e)
                } else {
                    None
                }
            }
            ReaperPreparationOutcome::InstallByMovingFromExtractedReaper {
                dir_containing_reaper,
                download,
            } => {
                debug_assert!(
                    self.resolved_config.portable,
                    "Installing main REAPER by moving not supported for any OS"
                );
                if let Err(e) = self.apply_reaper_portable_by_moving(dir_containing_reaper) {
                    self.install_manually_due_to_error(download, &e)
                } else {
                    None
                }
            }
        }
    }

    fn install_manually_due_to_error(
        &self,
        download: &ToolDownload,
        error: &Error,
    ) -> Option<PathBuf> {
        tracing::warn!("ReaBoot tried to install REAPER for you, but there was an error. Please install it manually. The installer is located here: {:?}\n\nError: {:#}", &download.download.file, &error);
        Some(download.download.file.clone())
    }

    fn apply_reaper_portable_by_moving(
        &self,
        dir_containing_reaper: &PathBuf,
    ) -> anyhow::Result<()> {
        move_dir_contents(
            dir_containing_reaper,
            self.resolved_config.reaper_resource_dir.get(),
        )?;
        // Copy reaper.ini file, but only if it doesn't exist already. E.g. the silent Windows installation
        // already contains such a file with some minimal content.
        let _ = move_file(
            self.temp_reaper_resource_dir.reaper_ini_file(),
            self.resolved_config.reaper_resource_dir.reaper_ini_file(),
            false,
        );
        Ok(())
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

    async fn prepare_and_simulate_installation<'a>(
        &'a self,
        downloaded_indexes: &'a HashMap<Url, DownloadedIndex>,
        successful_downloads: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
        installed_packages_to_be_replaced: &'a [InstalledPackage],
        installed_packages_to_be_removed: &'a [InstalledPackage],
    ) -> anyhow::Result<(
        Vec<SinglePackageInstallationPlan<'a>>,
        Vec<TempInstallFailure<'a>>,
    )> {
        self.listener
            .installation_stage_changed(InstallationStage::PreparingReaPackState);
        self.create_or_update_reapack_ini_file(downloaded_indexes)?;
        let outcome = self
            .create_or_update_reapack_db(
                successful_downloads,
                installed_packages_to_be_replaced,
                installed_packages_to_be_removed,
            )
            .await?;
        Ok((outcome.package_installation_plans, outcome.package_failures))
    }

    async fn create_or_update_reapack_db<'a>(
        &'a self,
        successful_downloads: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
        installed_packages_to_be_replaced: &'a [InstalledPackage],
        installed_packages_to_be_removed: &'a [InstalledPackage],
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
            .update_reapack_db(
                successful_downloads,
                installed_packages_to_be_replaced,
                installed_packages_to_be_removed,
            )
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
        installed_packages_to_be_removed: &'a [InstalledPackage],
    ) -> anyhow::Result<TempInstallOutcome> {
        // Make some transformations
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
        // Open temporary ReaPack DB
        let mut outcome = TempInstallOutcome::default();
        let mut db = self.open_temp_reapack_db().await?;
        // Remove existing packages
        for p in installed_packages_to_be_removed {
            self.update_reapack_db_for_one_removed_package(&mut db, p)
                .await?;
        }
        let files_of_removed_packages = installed_packages_to_be_removed
            .iter()
            .flat_map(|p| p.files.iter().map(|f| &f.path))
            .collect();
        // Add new packages
        for (version_id, downloads) in downloads_by_version {
            let replace_package = replace_package_by_id.remove(&version_id.package_id);
            let install_result = self
                .update_reapack_db_for_one_new_package(
                    &mut db,
                    version_id,
                    &downloads,
                    replace_package,
                    &files_of_removed_packages,
                )
                .await;
            match install_result {
                Ok(_) => {
                    let plan = SinglePackageInstallationPlan {
                        version: downloads
                            .first()
                            .expect("there must be at least one download at this point")
                            .payload
                            .version,
                        to_be_moved: downloads,
                        to_be_removed: replace_package,
                    };
                    outcome.package_installation_plans.push(plan);
                }
                Err(error) => {
                    let failure = TempInstallFailure { version_id, error };
                    outcome.package_failures.push(failure);
                }
            }
        }
        db.close().await?;
        Ok(outcome)
    }

    async fn update_reapack_db_for_one_removed_package(
        &self,
        db: &mut Database,
        removed_package: &InstalledPackage,
    ) -> anyhow::Result<()> {
        let transaction = db.with_transaction(|mut transaction| async {
            self.dry_remove_package(&mut transaction, removed_package)
                .await?;
            Ok(transaction)
        });
        transaction.await?;
        Ok(())
    }

    async fn dry_remove_package(
        &self,
        transaction: &mut DatabaseTransaction<'_>,
        p: &InstalledPackage,
    ) -> anyhow::Result<()> {
        // Dry remove installed package files
        for file in &p.files {
            let absolute_path = self
                .resolved_config
                .reaper_resource_dir
                .get()
                .join(&file.path);
            dry_remove_file(&absolute_path)?;
        }
        // Remove package from DB
        self.listener.info(format!("Remove package {p} from DB..."));
        transaction.remove_package(p.package_id()).await?;
        Ok(())
    }

    async fn update_reapack_db_for_one_new_package<'a>(
        &self,
        db: &mut Database,
        version_id: LightVersionId<'a>,
        downloads: &[DownloadWithPayload<QualifiedSource<'a>>],
        replace_package: Option<&InstalledPackage>,
        files_of_removed_packages: &HashSet<&String>,
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
            let relative_paths_to_be_replaced: HashSet<_> = if let Some(p) = replace_package {
                self.dry_remove_package(&mut transaction, p).await?;
                p.files.iter().map(|f| &f.path).collect()
            } else {
                HashSet::new()
            };
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
                if files_of_removed_packages.contains(&download.payload.relative_path) {
                    // Dry-moving this file would fail because there's still another ReaPack
                    // managed file at this location belonging to another package that has been
                    // marked for removal. We are going to remove it eventually, so that's fine.
                    continue;
                }
                let src_file = &download.download.file;
                let dest_file = self
                    .resolved_config
                    .reaper_resource_dir
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
            self.resolved_config.reaper_resource_dir.reapack_ini_file(),
            &self.resolved_config.backup_dir,
        )
        .context("moving ReaPack INI file failed")?;
        move_file_overwriting_with_backup(
            self.temp_reaper_resource_dir.reapack_registry_db_file(),
            self.resolved_config
                .reaper_resource_dir
                .reapack_registry_db_file(),
            &self.resolved_config.backup_dir,
        )
        .context("moving ReaPack registry DB file failed")?;
        let dest_cache_dir = self.resolved_config.reaper_resource_dir.reapack_cache_dir();
        // It can happen that we downloaded one repository index by 2 different URLs. This would
        // result in only one XML file on disk, but the hash map refers to it with 2 different
        // URLs. It's important that we move the XML file only once.
        let downloaded_index_files: HashSet<_> = downloaded_indexes
            .values()
            .map(|i| i.temp_download_file.clone())
            .collect();
        for downloaded_index_file in downloaded_index_files {
            let src_index_file_name = downloaded_index_file
                .file_name()
                .context("ReaPack index file should have a name at this point")?;
            let dest_index_file = dest_cache_dir.join(src_index_file_name);
            move_file_overwriting_with_backup(
                &downloaded_index_file,
                dest_index_file,
                &self.resolved_config.backup_dir,
            )
            .context("moving cached ReaPack repository index failed")?;
        }
        Ok(())
    }

    fn install_packages(
        &self,
        packages_to_be_removed: &[InstalledPackage],
        plans: Vec<SinglePackageInstallationPlan>,
    ) -> anyhow::Result<()> {
        for p in packages_to_be_removed {
            self.uninstall_package(p)?;
        }
        for plan in plans {
            self.install_package(plan)?;
        }
        Ok(())
    }

    fn install_package(&self, plan: SinglePackageInstallationPlan) -> anyhow::Result<()> {
        self.listener
            .installation_stage_changed(InstallationStage::InstallingPackage {
                package: PackageInfo {
                    name: plan.version.id().package_id.package.to_string(),
                },
            });
        if let Some(p) = plan.to_be_removed {
            self.uninstall_package(p)?;
        }
        // Copy/move
        self.listener.info(format!(
            "Moving files of new package {}",
            &plan.version.id()
        ));
        for download in plan.to_be_moved.into_iter() {
            let src_file = download.download.file;
            let dest_file = self
                .resolved_config
                .reaper_resource_dir
                .get()
                .join(download.payload.relative_path);
            // Overwriting the destination file is important! We take exclusive ownership
            // of any previously unmanaged file and overwrite it.
            // See https://github.com/helgoboss/reaboot/issues/2
            move_file(&src_file, &dest_file, true)?;
        }
        Ok(())
    }

    fn uninstall_package(&self, p: &InstalledPackage) -> Result<(), Error> {
        // Remove files
        self.listener
            .info(format!("Deleting files of existing package {p}"));
        for file in &p.files {
            let path = self
                .resolved_config
                .reaper_resource_dir
                .get()
                .join(&file.path);
            if path.try_exists().is_ok_and(|exists| !exists) {
                continue;
            }
            fs::remove_file(path).context("couldn't remove file of package to be replaced")?;
        }
        Ok(())
    }

    async fn gather_already_installed_packages_internal(
        &self,
        downloaded_indexes: &HashMap<Url, DownloadedIndex>,
        reapack_db_file: &Path,
    ) -> anyhow::Result<PackageStatusQuo> {
        let mut db = Database::open(reapack_db_file).await?;
        // When querying the installed packages, the database might be migrated to the latest
        // schema. That's okay because we are working on a copy of the original DB file only.
        let already_installed_packages = db.installed_packages().await?;
        let package_ids_to_be_installed: HashSet<LightPackageId> = self
            .resolved_config
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

    async fn download_and_prepare_reaper_if_necessary(
        &self,
    ) -> anyhow::Result<Option<ReaperPreparationOutcome>> {
        if (self.resolved_config.reaper_exe_exists && !self.resolved_config.update_reaper)
            || (!self.resolved_config.reaper_exe_exists && !self.resolved_config.install_reaper)
        {
            return Ok(None);
        }
        // Check for latest REAPER version
        self.listener
            .installation_stage_changed(InstallationStage::CheckingLatestReaperVersion);
        let reaper_installer_asset = reaper_util::get_latest_reaper_installer_asset(
            self.resolved_config.platform,
            &self.resolved_config.reaper_version,
        )
        .await?;
        // Skip download if version is up-to-date
        if self.resolved_config.reaper_exe_exists {
            if let Some(v) = self
                .resolved_config
                .reaper_resource_dir
                .read_installed_version()
            {
                if v >= reaper_installer_asset.version {
                    // No update necessary
                    return Ok(None);
                }
            }
        }
        // Download
        let download = self.download_reaper(reaper_installer_asset).await?;
        let outcome = self.prepare_reaper(download).await?;
        Ok(Some(outcome))
    }

    async fn download_reaper(
        &self,
        installer_asset: ReaperInstallerAsset,
    ) -> anyhow::Result<ToolDownload> {
        let file = self
            .temp_dir_for_reaper_download
            .join(installer_asset.file_name);
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

    async fn prepare_reaper(
        &self,
        reaper_download: ToolDownload,
    ) -> anyhow::Result<ReaperPreparationOutcome> {
        if !self.resolved_config.reaper_is_installable {
            let outcome = ReaperPreparationOutcome::InstallManually(reaper_download);
            return Ok(outcome);
        }
        self.listener
            .installation_stage_changed(InstallationStage::PreparingReaper);
        if self.resolved_config.portable && !self.resolved_config.reaper_ini_exists {
            fs::File::create(self.temp_reaper_resource_dir.reaper_ini_file())?;
        }
        if cfg!(target_os = "windows") && !self.resolved_config.portable {
            Ok(ReaperPreparationOutcome::InstallWindowsMainViaInstaller(
                reaper_download,
            ))
        } else {
            let reaper_dest_dir = self.temp_dir.join("reaper-binaries");
            let result = extract_reaper_to_dir(
                &reaper_download.download.file,
                &reaper_dest_dir,
                &self.temp_dir,
            );
            let outcome = if let Err(error) = result {
                ReaperPreparationOutcome::InstallManuallyDueToError(reaper_download, error)
            } else {
                ReaperPreparationOutcome::InstallByMovingFromExtractedReaper {
                    download: reaper_download,
                    dir_containing_reaper: reaper_dest_dir,
                }
            };
            Ok(outcome)
        }
    }

    async fn download_repository_indexes(&self) -> anyhow::Result<HashMap<Url, DownloadedIndex>> {
        let temp_cache_dir = self.temp_reaper_resource_dir.reapack_cache_dir();
        let repository_urls: HashSet<_> = self
            .resolved_config
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
                    // We still need to include this URL-index pair because the package URLs
                    // refer to specific repository URLs. If not, we would get a "Repository
                    // unavailable" error at the end.
                    tracing::warn!(
                        msg = "Encountered multiple URLs returning index with same name",
                        index_name,
                        ignored_url = %download.download.url
                    );
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
        sources: Vec<QualifiedSource<'a>>,
    ) -> Vec<DownloadResult<QualifiedSource<'a>>> {
        let downloads = sources.into_iter().map(|source| DownloadWithPayload {
            download: Download {
                label: source.simple_file_name().to_string(),
                url: source.source.content.clone(),
                file: self
                    .temp_reaper_resource_dir
                    .get()
                    .join(&source.relative_path),
                expected_multihash: source.source.hash.clone(),
            },
            payload: source,
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
        let file_in_final_dir = self.resolved_config.reaper_resource_dir.join(&rel_path);
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

    fn warn(&self, message: impl Display + Debug);

    fn info(&self, message: impl Display + Debug);

    fn debug(&self, message: impl Display + Debug);

    fn confirm(&self, request: ConfirmationRequest);
}

#[derive(Debug)]
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

#[allow(clippy::enum_variant_names)]
enum ReaperPreparationOutcome {
    /// It's a main REAPER install, and we don't support automatic REAPER installation for that.
    InstallManually(ToolDownload),
    /// It's a portable REAPER install but there was an error extracting the executables.
    InstallManuallyDueToError(ToolDownload, anyhow::Error),
    /// REAPER has been extracted successfully.
    InstallByMovingFromExtractedReaper {
        download: ToolDownload,
        /// The REAPER executable along with other files (Windows & Linux) or the bundle dir
        /// `REAPER.app` (macOS) has been placed **into** the given directory.
        dir_containing_reaper: PathBuf,
    },
    /// This happens on main REAPER install on Windows.
    InstallWindowsMainViaInstaller(ToolDownload),
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
    // It's okay if the destination file exists already, not managed by ReaPack! See
    // https://github.com/helgoboss/reaboot/issues/2!
    // In this case, ReaPack/ReaBoot takes ownership.
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

#[derive(Debug)]
pub struct InstallationOutcome {
    pub preparation_report: PreparationReport,
    pub actually_installed_things: bool,
    pub manual_reaper_install_path: Option<PathBuf>,
}

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("ReaBoot didn't install anything because some packages failed.")]
    SomePackagesFailed(PreparationReport),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Default)]
struct TempInstallOutcome<'a> {
    package_failures: Vec<TempInstallFailure<'a>>,
    package_installation_plans: Vec<SinglePackageInstallationPlan<'a>>,
}
