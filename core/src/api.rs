use crate::reaper_platform::ReaperPlatform;

use serde::{Deserialize, Serialize};

use crate::reaper_resource_dir::ReaperResourceDir;
use reaboot_reapack::model::VersionRef;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use strum::{AsRefStr, EnumIs};
use ts_rs::TS;
use url::Url;

/// Basic configuration-independent information gathered by the backend.
#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct ReabootBackendInfo {
    /// Resource directory of the main REAPER installation.
    ///
    /// If `None`, that means it couldn't be determined for some reason. Should be rare.
    pub main_reaper_resource_dir: Option<PathBuf>,
    /// Whether this resource directory exists already.
    pub main_reaper_resource_dir_exists: bool,
    /// Targeted REAPER platform, derived from the OS and architecture for which ReaBoot itself
    /// was compiled.
    ///
    /// If `None`, that means it couldn't be determined for some reason. Should be rare.
    pub inherent_reaper_platform: Option<ReaperPlatform>,
}

/// Data structure for configuring the installer from the frontend.
#[derive(Clone, Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct InstallerConfig {
    /// Custom REAPER resource directory to which to install ReaPack and packages.
    ///
    /// If not provided, it will take the main resource directory for the currently logged-in user.
    ///
    /// If this directory doesn't contain `reaper.ini`, ReaBoot assumes that REAPER is
    /// not installed yet. In this case, it will attempt to install REAPER. It will
    /// make a main installation if this corresponds to the typical REAPER resource path of
    /// a main installation, otherwise it will make a portable install **into** this
    /// directory.
    #[ts(optional)]
    pub custom_reaper_resource_dir: Option<PathBuf>,
    /// OS and architecture for which to install things.
    ///
    /// Influences the choice of binaries (REAPER, ReaPack and packages).
    ///
    /// If not provided, this will be derived from the OS and architecture for which ReaBoot itself
    /// was compiled.
    #[ts(optional)]
    pub custom_platform: Option<ReaperPlatform>,
    /// A list of package URLs pointing to packages to be installed.
    ///
    /// These recipes will be installed *in addition* to those that are going to be installed
    /// anyway (if the installer is branded).
    pub package_urls: Vec<Url>,
    /// Maximum number of retries if a download fails.
    #[ts(optional)]
    pub num_download_retries: Option<u32>,
    /// The directory in which the installer creates a randomly-named temporary directory for
    /// download purposes.
    ///
    /// If not provided, this will be `{REAPER_RESOURCE_DIR}/ReaBoot`.
    #[ts(optional)]
    pub temp_parent_dir: Option<PathBuf>,
    /// Whether to keep the temporary directory or not.
    pub keep_temp_dir: bool,
    /// Maximum number of concurrent downloads.
    #[ts(optional)]
    pub concurrent_downloads: Option<u32>,
    /// If `true`, nothing will be installed.
    ///
    /// A good way to check if the installation would (most likely) succeed.
    pub dry_run: bool,
    /// Which REAPER version to install if it doesn't exist already.
    #[ts(optional)]
    pub reaper_version: Option<VersionRef>,
    /// If `true`, the installer will succeed even if there are failed packages.
    pub skip_failed_packages: bool,
}

/// Resolved installer configuration (derived from the frontend installer config).
#[derive(Clone, Debug, Serialize, TS)]
pub struct ResolvedInstallerConfig {
    /// Resolved REAPER resource directory.
    pub reaper_resource_dir: ReaperResourceDir,
    /// Whether the resolved REAPER resource directory exists.
    pub reaper_resource_dir_exists: bool,
    /// Whether the resolved REAPER resource directory belongs to the main REAPER installation
    /// or to a portable REAPER installation.
    pub portable: bool,
    /// Resolved REAPER platform.
    pub platform: ReaperPlatform,
    /// Resolved package URLs (includes URLs of packages that will be installed anyway).
    pub package_urls: Vec<Url>,
    pub num_download_retries: u32,
    pub temp_parent_dir: PathBuf,
    pub keep_temp_dir: bool,
    pub concurrent_downloads: u32,
    pub dry_run: bool,
    pub reaper_version: VersionRef,
    pub skip_failed_packages: bool,
}

/// Status of the installation process.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, AsRefStr, Serialize, EnumIs, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum InstallationStage {
    /// Initial status.
    #[strum(serialize = "Nothing is installed yet")]
    NothingInstalled,
    /// Checking which is the latest available REAPER version.
    #[strum(serialize = "Checking latest REAPER version")]
    CheckingLatestReaperVersion,
    /// Downloading REAPER.
    #[strum(serialize = "Downloading REAPER")]
    DownloadingReaper { download: DownloadInfo },
    /// Installing REAPER (to a temporary directory at first).
    #[strum(serialize = "Extracting REAPER")]
    ExtractingReaper,
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
    /// ReaPack is already installed in the minimum supported version.
    ///
    /// This means that the ReaPack shared library already exists in the desired REAPER resource
    /// directory and that its version is high enough (verified by checking that the major pragma
    /// user version of the ReaPack database is high enough)
    #[strum(to_string = "REAPER and ReaPack are installed")]
    InstalledReaPack,

    // =================================================================================
    // === Everything above is only done if REAPER/ReaPack was not already available ===
    // =================================================================================
    /// Copying any existing ReaPack INI and registry DB file to the temporary directory.
    #[strum(serialize = "Preparing temporary directory")]
    PreparingTempDirectory,
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
    /// Creating/updating ReaPack INI file and registry database.
    #[strum(serialize = "Updating ReaPack state")]
    UpdatingReaPackState,

    // ===========================================================
    // === Everything below is not done if it's just a dry run ===
    // ===========================================================
    /// Moving ReaPack INI file, registry database and cached indexes to the final destination.
    #[strum(serialize = "Applying ReaPack state")]
    ApplyingReaPackState,
    /// Moving the files of each package to its final destination and updating the database.
    #[strum(serialize = "Applying package")]
    ApplyingPackage { package: PackageInfo },
    #[strum(serialize = "Installation failed")]
    Failed { display_msg: String },
    /// Installation has errored or is finished.
    #[strum(serialize = "Finished successfully")]
    Finished,
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
}

impl Display for InstallationStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let simple_name: &str = self.as_ref();
        match self {
            InstallationStage::DownloadingReaper { download }
            | InstallationStage::DownloadingReaPack { download } => {
                write!(
                    f,
                    "{simple_name} {} ({:?})",
                    &download.label,
                    download.file.file_name().unwrap()
                )?;
            }
            InstallationStage::DownloadingRepositoryIndexes { download }
            | InstallationStage::DownloadingPackageFiles { download } => {
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
            InstallationStage::ApplyingPackage { package } => {
                write!(f, "{simple_name}: {}", &package.name)?;
            }
            InstallationStage::Failed {
                display_msg: message,
            } => {
                write!(f, "{simple_name}: {}", message)?;
            }
            _ => {
                simple_name.fmt(f)?;
            }
        }
        Ok(())
    }
}
