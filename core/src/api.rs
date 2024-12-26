use crate::reaper_platform::ReaperPlatform;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::reaper_resource_dir::ReaperResourceDir;
use crate::recipe::Recipe;
use reaboot_reapack::model::{PackageUrl, VersionRef};
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
    #[ts(optional)]
    pub main_reaper_resource_dir: Option<PathBuf>,
    /// Whether "reaper.ini" exists in that resource directory.
    pub main_reaper_ini_exists: bool,
    /// Path of the executable belonging to this installation.
    ///
    /// If this is a portable installation, this corresponds to the REAPER executable in the
    /// portable directory (or `REAPER.app` bundle on macOS).
    ///
    /// If this is a main installation, this corresponds to the system-wide REAPER executable
    /// created by a default REAPER installation. On Linux, this first tries the system-wide
    /// location (`/opt/REAPER`) and then the user location (`$HOME/opt/reaper`).
    /// install locations.
    pub main_reaper_exe: PathBuf,
    /// Whether the executable belonging to this installation exists.
    pub main_reaper_exe_exists: bool,
    /// Targeted REAPER platform, derived from the OS and architecture for which ReaBoot itself
    /// was compiled.
    pub inherent_reaper_platform: ReaperPlatform,
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
    pub package_urls: Vec<String>,
    /// Maximum number of retries if a download fails.
    #[ts(optional)]
    pub num_download_retries: Option<u32>,
    /// The directory in which the installer creates a randomly-named temporary directory for
    /// download purposes.
    ///
    /// If not provided, this will be `{REAPER_RESOURCE_DIR}/ReaBoot`.
    #[ts(optional)]
    pub temp_parent_dir: Option<PathBuf>,
    /// Installation ID.
    ///
    /// Currently used in order to get a unique backup directory for this particular installation.
    /// If provided, it should be unique and contain only characters that are file-system-friendly.
    ///
    /// By default, an ID based on the current date is generated.
    #[ts(optional)]
    pub installation_id: Option<String>,
    /// Whether to keep the temporary directory or not (by default false).
    pub keep_temp_dir: bool,
    /// Maximum number of concurrent downloads.
    #[ts(optional)]
    pub concurrent_downloads: Option<u32>,
    /// If `true`, nothing will be installed (by default false).
    ///
    /// A good way to check if the installation would (most likely) succeed.
    pub dry_run: bool,
    /// Which REAPER version to install if it doesn't exist already.
    #[ts(optional)]
    pub reaper_version: Option<VersionRef>,
    /// If `true`, the installer will succeed even if there are failed packages (by default false).
    pub skip_failed_packages: bool,
    /// An optional recipe.
    #[ts(optional)]
    pub recipe: Option<Recipe>,
    /// The set of recipe features to be installed.
    ///
    /// Features not contained in the recipe will be ignored.
    pub selected_features: HashSet<String>,
    /// Whether to install REAPER or not if ReaBoot detects that it's missing (by default true).
    ///
    /// Important because ReaBoot's detection can only detect main installations in the default
    /// location.
    #[ts(optional)]
    pub install_reaper: Option<bool>,
    /// Update REAPER if there's a new version available (by default false).
    pub update_reaper: bool,
    /// Install ReaPack (by default true).
    #[ts(optional)]
    pub install_reapack: Option<bool>,
}

/// Resolved installer configuration (derived from the frontend installer config).
#[derive(Clone, Eq, PartialEq, Debug, Serialize, TS)]
pub struct ResolvedInstallerConfig {
    /// Resolved REAPER resource directory.
    pub reaper_resource_dir: ReaperResourceDir,
    /// Path of the executable belonging to this installation.
    ///
    /// If this is a portable installation, this corresponds to the REAPER executable in the
    /// portable directory (or `REAPER.app` bundle on macOS).
    ///
    /// If this is a main installation, this corresponds to the system-wide REAPER executable
    /// created by a default REAPER installation. On Linux, this first tries the system-wide
    /// location (`/opt/REAPER`) and then the user location (`$HOME/opt/reaper`).
    /// install locations.
    pub reaper_exe: PathBuf,
    /// Whether the resolved REAPER resource directory exists and has a "reaper.ini" file.
    pub reaper_ini_exists: bool,
    /// Whether the executable belonging to this installation exists.
    pub reaper_exe_exists: bool,
    /// Whether the resolved REAPER resource directory belongs to the main REAPER installation
    /// or to a portable REAPER installation.
    pub portable: bool,
    /// Whether ReaBoot would be capable of installing REAPER automatically.
    pub reaper_is_installable: bool,
    /// Resolved REAPER platform.
    pub platform: ReaperPlatform,
    /// Resolved package URLs.
    ///
    /// This includes manually configured packages, packages that will be installed anyway
    /// and packages that were selected via features.
    pub package_urls: Vec<PackageUrl>,
    /// Directory into which ReaBoot writes backups of modified configuration files.
    pub backup_dir: PathBuf,
    pub num_download_retries: u32,
    pub temp_parent_dir: PathBuf,
    pub keep_temp_dir: bool,
    pub concurrent_downloads: u32,
    pub dry_run: bool,
    pub reaper_version: VersionRef,
    pub skip_failed_packages: bool,
    /// Whether to install REAPER if necessary.
    pub install_reaper: bool,
    pub update_reaper: bool,
    pub install_reapack: bool,
    #[ts(optional)]
    pub recipe: Option<Recipe>,
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
    /// Preparing REAPER (e.g. install to a temporary directory at first).
    #[strum(serialize = "Preparing REAPER")]
    PreparingReaper,
    /// REAPER is already installed.
    ///
    /// This means that the desired REAPER resource directory already exists.
    #[strum(serialize = "REAPER is installed, ReaPack not yet")]
    InstalledReaper,

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
    PreparingReaPackState,

    // ===========================================================
    // === Everything below is not done if it's just a dry run ===
    // ===========================================================
    /// Move/install REAPER.
    #[strum(serialize = "Installing REAPER")]
    InstallingReaper,
    /// Moving ReaPack INI file, registry database and cached indexes to the final destination.
    #[strum(serialize = "Applying ReaPack state")]
    ApplyingReaPackState,
    /// Moving the files of each package to its final destination and updating the database.
    #[strum(serialize = "Installing package")]
    InstallingPackage { package: PackageInfo },
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

/// Request for confirmation by the user.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, TS)]
#[ts(export)]
pub struct ConfirmationRequest {
    pub message: String,
    pub yes_label: String,
    pub no_label: Option<String>,
}

impl Display for InstallationStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let simple_name: &str = self.as_ref();
        match self {
            InstallationStage::DownloadingReaper { download } => {
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
            InstallationStage::InstallingPackage { package } => {
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
