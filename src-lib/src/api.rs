use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
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
    /// A list of recipes with packages to install.
    ///
    /// These recipes will be installed *in addition* to the packages that are going to be installed
    /// anyway (if the installer is branded).
    pub recipes: Vec<Recipe>,
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
}

/// Status of the installation process.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum InstallationStatus {
    /// Initial status.
    Initial,
    /// Checking which is the latest available REAPER version.
    CheckingLatestReaperVersion,
    /// Downloading REAPER.
    DownloadingReaper {
        download: DownloadInfo,
    },
    /// REAPER is already installed.
    ///
    /// This means that the desired REAPER resource directory already exists.
    InstalledReaper,
    /// Checking which is the latest available ReaPack version.
    CheckingLatestReaPackVersion,
    /// Downloading ReaPack.
    DownloadingReaPack {
        download: DownloadInfo,
    },
    /// ReaPack is already installed.
    ///
    /// This means that the ReaPack shared library already exists in the desired REAPER resource
    /// directory.
    InstalledReaPack,
    /// Downloading all repositories in parallel.
    DownloadingRepositories {
        download: MultiDownloadInfo,
    },
    /// Downloading all package files in parallel.
    DownloadingPackageFiles {
        download: MultiDownloadInfo,
    },
    /// Moving the files of each package to its correct location and updating the database.
    InstallingPackage {
        package: PackageInfo,
    },
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

/// A collection of repositories and packages to be installed.
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
pub struct Recipe {
    /// Short display name.
    pub name: String,
    /// List of repositories along with packages to install from them.
    pub package_sets: Vec<PackageSet>,
}

/// A collection of packages within one repository.
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
pub struct PackageSet {
    /// URL of the repository to which all packages in this set belong.
    pub repository_url: Url,
    /// Packages in this set.
    pub packages: Vec<PackageDescriptor>,
}

/// Uniquely identifies a specific version of a package, within the context of a repository.
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
pub struct PackageDescriptor {
    /// Category of the package.
    pub category: String,
    /// Package name.
    pub name: String,
    /// Describes the version to be installed.
    pub version: VersionDescriptor,
}

/// Describes a package version.
#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum VersionDescriptor {
    /// Refers to the latest available version of a package, including pre-releases.
    Latest,
    /// Refers to the latest available version of a package, excluding pre-releases.
    LatestStable,
    /// Refers to a specific version of a package.
    #[serde(untagged)]
    Specific(String),
}

impl Recipe {
    pub fn repository_urls(&self) -> impl Iterator<Item = &Url> {
        self.package_sets.iter().map(|set| &set.repository_url)
    }
}
