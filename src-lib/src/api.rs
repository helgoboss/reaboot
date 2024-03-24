use crate::reaper_target::ReaperTarget;
use reaboot_reapack::model::VersionName;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use strum::{AsRefStr, Display};
use ts_rs::TS;
use url::Url;

/// A simple fire-and-forget command sent from the frontend to the backend.
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum ReabootCommand {
    /// Applies the given configuration.
    Configure { config: ReabootConfig },
    /// Starts the installation process.
    StartInstallation,
    /// Cancels the installation process.
    CancelInstallation,
}

/// Command for configuring the installation process.
#[derive(Clone, Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct ReabootConfig {
    /// Custom REAPER resource directory, most likely the one of a portable REAPER installation.
    ///
    /// If `None`, we will use the resource directory of the main installation.
    #[ts(optional)]
    pub custom_reaper_resource_dir: Option<PathBuf>,
    /// A list of package URLs pointing to packages to be installed.
    ///
    /// These recipes will be installed *in addition* to those that are going to be installed
    /// anyway (if the installer is branded).
    pub package_urls: Vec<Url>,
    /// Custom REAPER target.
    #[ts(optional)]
    pub custom_reaper_target: Option<ReaperTarget>,
}

/// Event emitted by the backend.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum ReabootEvent {
    Error { error: ReabootError },
    ConfigResolved { state: ResolvedReabootConfig },
    InstallationStatusChanged { status: InstallationStatus },
}

/// Error.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct ReabootError {
    pub display_msg: String,
}

/// State that should only change on configuration changes and a REAPER install, not during
/// the further installation process.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct ResolvedReabootConfig {
    /// The resolved REAPER resource directory which is going to be used for the installation.
    ///
    /// [`InstallationStatus`] indicates, whether this directory exists and is a valid
    /// REAPER resource directory.
    pub reaper_resource_dir: PathBuf,
    /// `true` if the resource directory is part of a portable REAPER installation (not the one of
    /// the main REAPER installation).
    pub portable: bool,
    /// Resolved REAPER target.
    pub reaper_target: ReaperTarget,
}

/// Status of the installation process.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, AsRefStr, Serialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum InstallationStatus {
    /// Initial status.
    #[strum(serialize = "Nothing is installed yet")]
    Initial,
    /// Checking which is the latest available REAPER version.
    #[strum(serialize = "Checking latest REAPER version")]
    CheckingLatestReaperVersion,
    /// Downloading REAPER.
    #[strum(serialize = "Downloading REAPER")]
    DownloadingReaper { download: DownloadInfo },
    /// REAPER is already installed.
    ///
    /// This means that the desired REAPER resource directory already exists.
    #[strum(serialize = "REAPER is installed, ReaPack not yet")]
    InstalledReaper,
    /// Checking which is the latest available ReaPack version.
    #[strum(serialize = "Checking latest ReaPack version")]
    CheckingLatestReaPackVersion,
    /// Downloading ReaPack.
    #[strum(to_string = "Download ReaPack")]
    DownloadingReaPack { download: DownloadInfo },
    /// ReaPack is already installed.
    ///
    /// This means that the ReaPack shared library already exists in the desired REAPER resource
    /// directory.
    #[strum(to_string = "REAPER and ReaPack are installed")]
    InstalledReaPack,
    /// Downloading all repository indexes in parallel.
    #[strum(serialize = "Downloading repository indexes")]
    DownloadingRepositoryIndexes { download: MultiDownloadInfo },
    /// Checking each downloaded repository index whether it's valid and converting it into a
    /// suitable data structure.
    #[strum(serialize = "Parsing repository indexes")]
    ParsingRepositoryIndexes,
    /// Preparing package installation.
    ///
    /// This includes:
    /// - Deduplicate package descriptors
    /// - Search within repository indexes for packages mentioned in the given recipes
    /// - Deduplicate package versions
    /// - Check for conflicting package versions
    /// - Check for duplicate files
    #[strum(serialize = "Preparing package downloading")]
    PreparingPackageDownloading,
    /// Downloading all package files in parallel.
    #[strum(serialize = "Downloading package files")]
    DownloadingPackageFiles { download: MultiDownloadInfo },
    /// Moving the files of each package to its correct location and updating the database.
    #[strum(serialize = "Installing package")]
    InstallingPackage { package: PackageInfo },
    #[strum(serialize = "Done")]
    Done,
}

/// Information about an ongoing file download.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, TS)]
#[ts(export)]
pub struct MultiDownloadInfo {
    pub in_progress_count: u32,
    pub success_count: u32,
    pub error_count: u32,
    pub total_count: u32,
}

/// Information about an ongoing file download.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, TS)]
#[ts(export)]
pub struct DownloadInfo {
    /// A human-friendly label for the download.
    pub label: String,
    /// Remote URL from which we are downloading.
    pub url: Url,
    /// Destination file on the local file system.
    pub file: PathBuf,
}

/// Basic information about a package.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, TS)]
#[ts(export)]
pub struct PackageInfo {
    pub name: String,
    pub desc: String,
    pub version: String,
    pub author: String,
}

impl Display for InstallationStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let simple_name: &str = self.as_ref();
        match self {
            InstallationStatus::DownloadingRepositoryIndexes { download }
            | InstallationStatus::DownloadingPackageFiles { download } => {
                let error_count = download.error_count;
                let actual_count = download.success_count + error_count;
                let total_count = download.total_count;
                write!(
                    f,
                    "{simple_name}: Downloaded {actual_count} of {total_count}"
                )?;
                if error_count > 0 {
                    write!(f, " ({error_count} errors)")?;
                }
            }
            InstallationStatus::InstallingPackage { .. } => {}
            _ => {
                simple_name.fmt(f)?;
            }
        }
        Ok(())
    }
}
