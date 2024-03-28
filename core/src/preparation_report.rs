use crate::display_util::Separated;
use crate::installation_model::{
    PackageDescError, PackageInstallationPlan, PreDownloadFailures, QualifiedSource,
    QualifiedVersion, TempInstallFailure,
};
use crate::multi_downloader::DownloadError;
use reaboot_reapack::model::{PackageId, VersionName, VersionRef};
use std::fmt::{Display, Formatter, Write};

#[derive(Debug)]
pub struct PreparationReport {
    pub package_preparation_outcomes: Vec<PackagePreparationOutcome>,
}

#[derive(Debug)]
pub struct PackagePreparationOutcome {
    pub package_id: PackageId,
    pub version: Option<VersionRef>,
    pub status: PackagePrepStatus,
}

#[derive(Debug)]
pub enum PackagePrepStatus {
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
    ToBeAdded,
    /// Nothing failed in the preparation phase. The actual installation step is very likely
    /// to be successful.
    ToBeReplaced { old_version: String },
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
                version: Some(failure.package_url.version_ref().clone()),
                status: PackagePrepStatus::NotFoundInRepo(failure.error),
            });
        let version_conflicts =
            download_plan
                .version_conflicts
                .into_iter()
                .map(|conflict| PackagePreparationOutcome {
                    package_id: conflict.package_id.to_owned(),
                    version: None,
                    status: PackagePrepStatus::VersionConflict(
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
                    version: Some(version.id().version.clone().into()),
                    status: PackagePrepStatus::Incompatible,
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
                        version: Some(f.version.version.name.clone().into()),
                        status: PackagePrepStatus::ConflictWithOtherPackagesToBeInstalled {
                            relative_path: conflict.relative_path.clone(),
                        },
                    })
            });
        let conflicts_with_already_installed_packages = download_plan
            .files_conflicting_with_already_installed_files
            .into_iter()
            .map(|s| PackagePreparationOutcome {
                package_id: s.package_id().to_owned(),
                version: Some(s.version.version.name.clone().into()),
                status: PackagePrepStatus::ConflictWithAlreadyInstalledFiles {
                    relative_path: s.relative_path,
                },
            });
        let failed_downloads = download_errors
            .into_iter()
            .map(|e| PackagePreparationOutcome {
                package_id: e.download.payload.package_id().to_owned(),
                version: Some(e.download.payload.version.version.name.clone().into()),
                status: PackagePrepStatus::DownloadFailed(e.error),
            });
        let temp_install_fails =
            temp_install_failures
                .into_iter()
                .map(|failure| PackagePreparationOutcome {
                    package_id: failure.version_id.package_id.to_owned(),
                    version: Some(failure.version_id.version.clone().into()),
                    status: PackagePrepStatus::TempInstallFailed(failure.error),
                });
        let ready = package_installation_plans
            .iter()
            .map(|a| PackagePreparationOutcome {
                package_id: a.version_id.package_id.to_owned(),
                version: Some(a.version_id.version.clone().into()),
                status: if let Some(p) = a.to_be_removed {
                    PackagePrepStatus::ToBeReplaced {
                        old_version: p.version.to_string(),
                    }
                } else {
                    PackagePrepStatus::ToBeAdded
                },
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

    pub fn summary(&self) -> PreparationReportSummary {
        let mut summary = PreparationReportSummary::default();
        for o in &self.package_preparation_outcomes {
            use PackageStatusCategory::*;
            match o.status.category() {
                Failure => {
                    summary.failures += 1;
                }
                Replacement => {
                    summary.replacements += 1;
                }
                Addition => {
                    summary.additions += 1;
                }
            }
        }
        summary
    }
}

#[derive(Default)]
pub struct PreparationReportSummary {
    pub failures: usize,
    pub replacements: usize,
    pub additions: usize,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PackageStatusCategory {
    Failure,
    Replacement,
    Addition,
}

impl PackagePrepStatus {
    pub fn category(&self) -> PackageStatusCategory {
        use PackageStatusCategory::*;
        match self {
            PackagePrepStatus::ToBeAdded => Addition,
            PackagePrepStatus::ToBeReplaced { .. } => Replacement,
            _ => Failure,
        }
    }
}

impl Display for PackagePrepStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PackagePrepStatus::NotFoundInRepo(e) => e.fmt(f)?,
            PackagePrepStatus::VersionConflict(versions) => {
                let csv = Separated::new(|| versions.iter(), ",");
                write!(f, "Competing version requirements: {csv}")?;
            }
            PackagePrepStatus::Incompatible => {
                f.write_str("Incompatible with operating system or ReaBoot")?
            }
            PackagePrepStatus::ConflictWithOtherPackagesToBeInstalled { relative_path } => {
                write!(
                    f,
                    "File conflict with other package to be installed: {relative_path}"
                )?;
            }
            PackagePrepStatus::ConflictWithAlreadyInstalledFiles { relative_path } => {
                write!(
                    f,
                    "File conflict with already installed package: {relative_path}"
                )?;
            }
            PackagePrepStatus::DownloadFailed(e) => {
                write!(f, "Download error: {e}")?;
            }
            PackagePrepStatus::TempInstallFailed(e) => {
                write!(f, "Simulated installation failed: {e}")?;
            }
            PackagePrepStatus::ToBeAdded => {
                f.write_str("Addition")?;
            }
            PackagePrepStatus::ToBeReplaced { old_version } => {
                write!(f, "Replacement of {old_version}")?;
            }
        }
        Ok(())
    }
}

pub struct PreparationReportAsMarkdown<'a> {
    report: &'a PreparationReport,
    packages_have_been_installed: bool,
}

impl<'a> PreparationReportAsMarkdown<'a> {
    pub fn new(report: &'a PreparationReport, packages_have_been_installed: bool) -> Self {
        Self {
            report,
            packages_have_been_installed,
        }
    }
}

impl<'a> Display for PreparationReportAsMarkdown<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let summary = self.report.summary();
        writeln!(f, "# Installation report")?;
        if summary.failures > 0 {
            writeln!(f, "## {} package failure(s)", summary.failures)?;
            write_3col_table_header(f, "Package", "Version", "Error")?;
            for o in &self.report.package_preparation_outcomes {
                if o.status.category() == PackageStatusCategory::Failure {
                    outcome_row(f, o, &o.status)?;
                }
            }
        }
        let suffix = if self.packages_have_been_installed {
            ""
        } else {
            " **[SKIPPED]**"
        };
        if summary.replacements > 0 {
            writeln!(
                f,
                "## {} package replacement(s){suffix}\n",
                summary.replacements
            )?;
            write_3col_table_header(
                f,
                "Package",
                "Version",
                "Replaces previously installed version",
            )?;
            for o in &self.report.package_preparation_outcomes {
                if let PackagePrepStatus::ToBeReplaced { old_version } = &o.status {
                    outcome_row(f, o, old_version)?;
                }
            }
        }
        if summary.additions > 0 {
            writeln!(f, "## {} package addition(s){suffix}\n", summary.additions)?;
            for o in &self.report.package_preparation_outcomes {
                if o.status.category() == PackageStatusCategory::Addition {
                    writeln!(f, "- {}", PackageVersionAsMarkdown(o))?;
                }
            }
        }
        Ok(())
    }
}

fn write_3col_table_header(
    f: &mut Formatter,
    label1: &str,
    label2: &str,
    label3: &str,
) -> std::fmt::Result {
    writeln!(f, "|:-|:-|:-")?;
    writeln!(f, "|**{label1}**|**{label2}**|**{label3}**")?;
    writeln!(f, "|:-|:-|:-")?;
    Ok(())
}

fn write_3col_table_divider(f: &mut Formatter) -> std::fmt::Result {
    f.write_str("|-|-|-\n")
}

fn outcome_row(
    f: &mut Formatter,
    outcome: &PackagePreparationOutcome,
    label3: impl Display,
) -> std::fmt::Result {
    write!(f, "| {} | ", &outcome.package_id.package)?;
    if let Some(v) = outcome.version.as_ref() {
        v.fmt(f)?;
    } else {
        f.write_char('-')?;
    }
    writeln!(f, " | {label3}")?;
    write_3col_table_divider(f)?;
    Ok(())
}

fn write_3col_table_row(
    f: &mut Formatter,
    label1: impl Display,
    label2: impl Display,
    label3: impl Display,
) -> std::fmt::Result {
    writeln!(f, "| {label1} | {label2} | {label3} ")?;
    Ok(())
}

struct PackageVersionAsMarkdown<'a>(&'a PackagePreparationOutcome);

impl<'a> Display for PackageVersionAsMarkdown<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "**{}**", &self.0.package_id.package)?;
        if let Some(v) = self.0.version.as_ref() {
            write!(f, " {v}")?;
        }
        write!(f, " ({})", &self.0.package_id.category)?;
        Ok(())
    }
}