use crate::installation_model::{
    PackageDescError, PackageInstallationPlan, PreDownloadFailures, QualifiedSource,
    QualifiedVersion, TempInstallFailure,
};
use crate::multi_downloader::DownloadError;
use reaboot_reapack::model::{PackageId, VersionName};

#[derive(Debug)]
pub struct PreparationReport {
    pub package_preparation_outcomes: Vec<PackagePreparationOutcome>,
}

#[derive(Debug)]
pub struct PackagePreparationOutcome {
    pub package_id: PackageId,
    pub version_name: Option<VersionName>,
    pub status: PackagePreparationStatus,
}

#[derive(Debug)]
pub enum PackagePreparationStatus {
    /// Packages that were mentioned in the recipes but are not in the repository index.
    NotFoundInRepo(PackageDescError),
    /// Packages for which it's unclear which version to install.
    VersionConflict(Vec<VersionName>),
    /// Package versions that have an unsupported package type override or don't have any files to
    /// install and therefore can be considered as not supported on this operating system.
    Incompatible,
    /// Package files that clash with other package files of packages mentioned in the recipes,
    /// because they would be installed to exactly the same destination (directory and name).
    ConflictWithOtherPackagesToBeInstalled { relative_path: String },
    /// Package files that clash with other package files of already installed packages,
    /// because they would be installed to exactly the same destination (directory and name).
    ConflictWithAlreadyInstalledFiles { relative_path: String },
    /// Some files of the package couldn't be downloaded.
    DownloadFailed(anyhow::Error),
    /// Applying the package to the temporary ReaPack DB and simulating an installation to
    /// the final destination failed.
    TempInstallFailed(anyhow::Error),
    /// Nothing failed in the preparation phase. The actual installation step is very likely
    /// to be successful.
    ReadyForInstallation,
}

impl PreparationReport {
    pub fn new(
        download_plan: PreDownloadFailures,
        download_errors: Vec<DownloadError<QualifiedSource>>,
        temp_install_failures: Vec<TempInstallFailure>,
        package_installation_plans: &[PackageInstallationPlan],
    ) -> Self {
        let not_found = download_plan
            .package_descriptors_with_failures
            .into_iter()
            .map(|failure| PackagePreparationOutcome {
                package_id: PackageId {
                    remote: failure
                        .remote
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| failure.package_url.repository_url().to_string()),
                    category: failure.package_url.category().to_string(),
                    package: failure.package_url.package_name().to_string(),
                },
                version_name: None,
                status: PackagePreparationStatus::NotFoundInRepo(failure.error),
            });
        let version_conflicts =
            download_plan
                .version_conflicts
                .into_iter()
                .map(|conflict| PackagePreparationOutcome {
                    package_id: conflict.package_id.to_owned(),
                    version_name: None,
                    status: PackagePreparationStatus::VersionConflict(
                        conflict
                            .conflicting_versions
                            .into_iter()
                            .map(|v| v.version.name.clone())
                            .collect(),
                    ),
                });
        let incompatible_versions =
            download_plan
                .incompatible_versions
                .into_iter()
                .map(|version| PackagePreparationOutcome {
                    package_id: version.id().package_id.to_owned(),
                    version_name: Some(version.id().version.clone()),
                    status: PackagePreparationStatus::Incompatible,
                });
        let conflicts_with_other_packages_to_be_installed = download_plan
            .recipe_file_conflicts
            .into_iter()
            .flat_map(|conflict| {
                conflict
                    .conflicting_files
                    .into_iter()
                    .map(move |f| PackagePreparationOutcome {
                        package_id: f.package_id().to_owned(),
                        version_name: Some(f.version.version.name.to_owned()),
                        status: PackagePreparationStatus::ConflictWithOtherPackagesToBeInstalled {
                            relative_path: conflict.relative_path.clone(),
                        },
                    })
            });
        let conflicts_with_already_installed_packages = download_plan
            .files_conflicting_with_already_installed_files
            .into_iter()
            .map(|s| PackagePreparationOutcome {
                package_id: s.package_id().to_owned(),
                version_name: Some(s.version.version.name.to_owned()),
                status: PackagePreparationStatus::ConflictWithAlreadyInstalledFiles {
                    relative_path: s.relative_path,
                },
            });
        let failed_downloads = download_errors
            .into_iter()
            .map(|e| PackagePreparationOutcome {
                package_id: e.download.payload.package_id().to_owned(),
                version_name: Some(e.download.payload.version.version.name.clone()),
                status: PackagePreparationStatus::DownloadFailed(e.error),
            });
        let temp_install_fails =
            temp_install_failures
                .into_iter()
                .map(|failure| PackagePreparationOutcome {
                    package_id: failure.version_id.package_id.to_owned(),
                    version_name: Some(failure.version_id.version.clone()),
                    status: PackagePreparationStatus::TempInstallFailed(failure.error),
                });
        let ready = package_installation_plans
            .iter()
            .map(|a| PackagePreparationOutcome {
                package_id: a.version_id.package_id.to_owned(),
                version_name: Some(a.version_id.version.clone()),
                status: PackagePreparationStatus::ReadyForInstallation,
            });
        let package_preparation_outcomes = not_found
            .chain(version_conflicts)
            .chain(incompatible_versions)
            .chain(conflicts_with_other_packages_to_be_installed)
            .chain(conflicts_with_already_installed_packages)
            .chain(failed_downloads)
            .chain(temp_install_fails)
            .chain(ready)
            .collect();
        Self {
            package_preparation_outcomes,
        }
    }

    pub fn has_failed_packages(&self) -> bool {
        self.package_preparation_outcomes
            .iter()
            .any(|p| p.status.failed())
    }
}

impl PackagePreparationStatus {
    pub fn failed(&self) -> bool {
        !matches!(self, Self::ReadyForInstallation)
    }
}
