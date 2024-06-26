use std::fmt::{Display, Formatter, Write};

use reaboot_reapack::model::{
    InstalledPackage, LightPackageId, PackageId, VersionName, VersionRef,
};

use crate::display_util::Separated;
use crate::downloader::Download;
use crate::installation_model::{
    PackageDescError, PreDownloadFailures, QualifiedSource, SinglePackageInstallationPlan,
    TempInstallFailure,
};
use crate::multi_downloader::DownloadError;

#[derive(Debug)]
pub struct PreparationReport {
    pub package_preparation_outcomes: Vec<PackagePreparationOutcome>,
    pub tooling_changes: Vec<ToolingChange>,
    /// This contains only removals without replacements
    pub package_removals: Vec<InstalledPackage>,
}

#[derive(Clone, Debug)]
pub struct ToolDownload {
    pub version: String,
    pub download: Download,
}

impl ToolDownload {
    pub fn new(version: String, download: Download) -> Self {
        Self { version, download }
    }
}

#[derive(Debug)]
pub struct ToolingChange {
    pub name: String,
    pub download: ToolDownload,
}

impl ToolingChange {
    pub fn new(name: String, download: ToolDownload) -> Self {
        Self { name, download }
    }
}

#[derive(Debug)]
pub struct PackagePreparationOutcome {
    pub package_id: PackageId,
    pub version: Option<VersionRef>,
    pub status: PackagePrepStatus,
    /// A donation URL. Should only be set if the package was installed, not just replaced.
    pub donation_url: Option<String>,
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
    ConflictWithAlreadyInstalledFiles {
        relative_path: String,
        installed_package_id: PackageId,
    },
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
        tooling_changes: Vec<ToolingChange>,
        pre_download_failures: PreDownloadFailures,
        download_errors: Vec<DownloadError<QualifiedSource>>,
        temp_install_failures: Vec<TempInstallFailure>,
        packages_to_be_removed: &[InstalledPackage],
        package_installation_plans: &[SinglePackageInstallationPlan],
    ) -> Self {
        let not_found = pre_download_failures
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
                donation_url: None,
            });
        let version_conflicts =
            pre_download_failures
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
                    donation_url: None,
                });
        let incompatible_versions =
            pre_download_failures
                .incompatible_versions
                .into_iter()
                .map(|version| PackagePreparationOutcome {
                    package_id: version.id().package_id.to_owned(),
                    version: Some(version.id().version.clone().into()),
                    status: PackagePrepStatus::Incompatible,
                    donation_url: None,
                });
        let conflicts_with_other_packages_to_be_installed = pre_download_failures
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
                        donation_url: None,
                    })
            });
        let conflicts_with_already_installed_packages = pre_download_failures
            .conflicts_with_already_installed_files
            .into_iter()
            .map(|s| PackagePreparationOutcome {
                package_id: s.new_file.package_id().to_owned(),
                version: Some(s.new_file.version.version.name.clone().into()),
                status: PackagePrepStatus::ConflictWithAlreadyInstalledFiles {
                    relative_path: s.new_file.relative_path,
                    installed_package_id: s.installed_package.package_id().to_owned(),
                },
                donation_url: None,
            });
        let failed_downloads = download_errors
            .into_iter()
            .map(|e| PackagePreparationOutcome {
                package_id: e.download.payload.package_id().to_owned(),
                version: Some(e.download.payload.version.version.name.clone().into()),
                status: PackagePrepStatus::DownloadFailed(e.error),
                donation_url: None,
            });
        let temp_install_fails =
            temp_install_failures
                .into_iter()
                .map(|failure| PackagePreparationOutcome {
                    package_id: failure.version_id.package_id.to_owned(),
                    version: Some(failure.version_id.version.clone().into()),
                    status: PackagePrepStatus::TempInstallFailed(failure.error),
                    donation_url: None,
                });
        let ready = package_installation_plans
            .iter()
            .map(|a| PackagePreparationOutcome {
                package_id: a.version.id().package_id.to_owned(),
                version: Some(a.version.id().version.clone().into()),
                status: if let Some(p) = a.to_be_removed {
                    PackagePrepStatus::ToBeReplaced {
                        old_version: p.version.to_string(),
                    }
                } else {
                    PackagePrepStatus::ToBeAdded
                },
                donation_url: if a.to_be_removed.is_some() {
                    None
                } else {
                    a.version.package.package.metadata().and_then(|md| {
                        let url = md.donation_urls().next()?;
                        Some(url.to_string())
                    })
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
            tooling_changes,
            package_removals: packages_to_be_removed.to_vec(),
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
            PackagePrepStatus::ConflictWithAlreadyInstalledFiles {
                relative_path,
                installed_package_id,
            } => {
                write!(
                    f,
                    "File `{relative_path}` conflicts with already installed package `{}`",
                    &installed_package_id.package
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
    options: PreparationReportMarkdownOptions,
}

#[derive(Copy, Clone)]
pub struct PreparationReportMarkdownOptions {
    /// Whether to include the main heading.
    pub include_main_heading: bool,
    /// Whether to show donation links.
    ///
    /// This will produce real HTML (in order to set `target="_blank"`).
    pub include_donation_links: bool,
    /// Whether the packages have actually been installed (`true`) or if installation failed
    /// or this was just a dry run (`false`).
    pub actually_installed_things: bool,
    /// Whether to optimize markdown output for [Termimad](https://github.com/Canop/termimad).
    pub optimize_for_termimad: bool,
}

impl<'a> PreparationReportAsMarkdown<'a> {
    pub fn new(report: &'a PreparationReport, options: PreparationReportMarkdownOptions) -> Self {
        Self { report, options }
    }

    fn write_heading(
        &self,
        f: &mut Formatter,
        label: &str,
        count: usize,
        suffix: &str,
    ) -> std::fmt::Result {
        write!(f, "\n## {count} {label}")?;
        if count > 1 {
            f.write_str("s")?;
        }
        writeln!(f, "{suffix}")?;
        Ok(())
    }

    fn write_3col_table_header(
        &self,
        f: &mut Formatter,
        label1: &str,
        label2: &str,
        label3: &str,
    ) -> std::fmt::Result {
        if self.options.optimize_for_termimad {
            // Termimad needs this in order to print the top table border. Standard
            // GFM parsers don't understand this.
            writeln!(f, "|:-|:-|:-")?;
        }
        writeln!(f, "|**{label1}**|**{label2}**|**{label3}**")?;
        writeln!(f, "|:-|:-|:-")?;
        Ok(())
    }

    fn write_3col_table_divider(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.options.optimize_for_termimad {
            // Termimad needs this in order to print the bottom table border. Standard
            // GFM parsers don't understand this.
            f.write_str("|-|-|-\n")?;
        }
        Ok(())
    }

    fn outcome_row(
        &self,
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
        self.write_3col_table_divider(f)?;
        Ok(())
    }
}

impl<'a> Display for PreparationReportAsMarkdown<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let summary = self.report.summary();
        if self.options.include_main_heading {
            writeln!(f, "\n# Installation report")?;
        }
        let mut heading_count = 0;
        if summary.failures > 0 {
            heading_count += 1;
            self.write_heading(f, "package failure", summary.failures, "")?;
            self.write_3col_table_header(f, "Package", "Version", "Error")?;
            for o in &self.report.package_preparation_outcomes {
                if o.status.category() == PackageStatusCategory::Failure {
                    self.outcome_row(f, o, &o.status)?;
                }
            }
        }
        let skipped_suffix = if self.options.actually_installed_things {
            ""
        } else {
            " **[SKIPPED]**"
        };
        if !self.report.tooling_changes.is_empty() {
            heading_count += 1;
            self.write_heading(
                f,
                "tooling change",
                self.report.tooling_changes.len(),
                skipped_suffix,
            )?;
            for c in &self.report.tooling_changes {
                f.write_str("- ")?;
                write_name_and_version(f, &c.name, Some(&c.download.version))?;
                f.write_char('\n')?;
            }
        }
        if !self.report.package_removals.is_empty() {
            heading_count += 1;
            self.write_heading(
                f,
                "package removal",
                self.report.package_removals.len(),
                skipped_suffix,
            )?;
            for removal in &self.report.package_removals {
                let version_markdown = PackageVersionAsMarkdown {
                    package_id: removal.package_id(),
                    version: Some(&removal.version),
                };
                writeln!(f, "- {version_markdown}")?;
            }
        }
        if summary.replacements > 0 {
            heading_count += 1;
            self.write_heading(
                f,
                "package replacement",
                summary.replacements,
                skipped_suffix,
            )?;
            self.write_3col_table_header(
                f,
                "Package",
                "Version",
                "Replaces previously installed version",
            )?;
            for o in &self.report.package_preparation_outcomes {
                if let PackagePrepStatus::ToBeReplaced { old_version } = &o.status {
                    self.outcome_row(f, o, old_version)?;
                }
            }
        }
        if summary.additions > 0 {
            heading_count += 1;
            self.write_heading(f, "package addition", summary.additions, skipped_suffix)?;
            if self.options.include_donation_links {
                self.write_3col_table_header(f, "Package", "Version", "Donate")?;
                for o in &self.report.package_preparation_outcomes {
                    if let Some(url) = &o.donation_url {
                        self.outcome_row(f, o, FormatAsLink(url))?;
                    } else {
                        self.outcome_row(f, o, "")?;
                    }
                }
            } else {
                for o in &self.report.package_preparation_outcomes {
                    if o.status.category() == PackageStatusCategory::Addition {
                        let version_markdown = PackageVersionAsMarkdown {
                            package_id: o.package_id.to_borrowed(),
                            version: o.version.as_ref(),
                        };
                        writeln!(f, "- {version_markdown}")?;
                    }
                }
            }
        }
        if heading_count == 0 {
            f.write_str("No changes were necessary.")?;
        }
        Ok(())
    }
}

struct FormatAsLink<'a>(&'a str);

impl<'a> Display for FormatAsLink<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<a href=\"{}\" target=\"_blank\">Donate</a>", self.0)
    }
}

struct PackageVersionAsMarkdown<'a, V> {
    package_id: LightPackageId<'a>,
    version: Option<V>,
}

impl<'a, V: Display> Display for PackageVersionAsMarkdown<'a, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_name_and_version(f, self.package_id.package, self.version.as_ref())?;
        write!(f, " ({})", &self.package_id.category)?;
        Ok(())
    }
}

fn write_name_and_version(
    f: &mut Formatter,
    name: impl Display,
    version: Option<impl Display>,
) -> std::fmt::Result {
    write!(f, "**{name}**")?;
    if let Some(v) = version {
        write!(f, " {v}")?;
    }
    Ok(())
}
