use anyhow::{bail, Context};
use std::env;

use reaboot_reapack::database::{CompatibilityInfo, Database};
use reaboot_reapack::model::{PackageUrl, ParsePackageUrlError};

use crate::api::{InstallationStage, InstallerConfig, ReabootBackendInfo, ResolvedInstallerConfig};
use crate::file_util::file_or_dir_is_writable_or_creatable;
use crate::reaper_platform::ReaperPlatform;
use crate::reaper_resource_dir::ReaperResourceDir;
use crate::{reapack_util, reaper_util};

pub fn collect_backend_info() -> ReabootBackendInfo {
    let (main_reaper_resource_dir, exists) =
        if let Ok(dir) = reaper_util::get_default_main_reaper_resource_dir() {
            let exists = dir.contains_reaper_ini();
            (Some(dir.into_inner()), exists)
        } else {
            (None, false)
        };
    ReabootBackendInfo {
        main_reaper_resource_dir,
        main_reaper_resource_dir_exists: exists,
        inherent_reaper_platform: ReaperPlatform::BUILD,
    }
}

pub fn resolve_config(config: InstallerConfig) -> anyhow::Result<ResolvedInstallerConfig> {
    let main_reaper_resource_dir = reaper_util::get_default_main_reaper_resource_dir()?;
    let (reaper_resource_dir, portable) = if let Some(d) = config.custom_reaper_resource_dir {
        let d = ReaperResourceDir::new(d)?;
        let portable = d != main_reaper_resource_dir;
        (d, portable)
    } else {
        (main_reaper_resource_dir, false)
    };
    let exists = reaper_resource_dir.contains_reaper_ini();
    // Determine platform
    let reaper_platform = config
        .custom_platform
        .or(ReaperPlatform::BUILD)
        .context("ReaBoot is running on a platform that's not supported by REAPER. It's still possible do do an install for a supported platform but you need to choose it manually.")?;

    // Prefer a sub dir of the destination REAPER resource dir as location for all temporary
    // files, e.g. downloads. Rationale: Eventually, all downloaded files need to be moved
    // to their final destination. This move is very fast if it's just a rename, not a copy
    // operation. A rename is only possible if source file and destination directory are on
    // the same mount ... and this is much more likely using this approach than to choose
    // a random OS temp directory.
    let temp_parent_dir = if let Some(d) = config.temp_parent_dir {
        if file_or_dir_is_writable_or_creatable(&d) {
            d
        } else {
            bail!("The custom temp parent dir you have provided is not writable");
        }
    } else {
        let temp_reaboot_dir = reaper_resource_dir.temp_reaboot_dir();
        if file_or_dir_is_writable_or_creatable(&temp_reaboot_dir) {
            temp_reaboot_dir
        } else {
            // Fall back to OS temp dir as parent. This can be useful in dry mode.
            env::temp_dir()
        }
    };
    let package_urls: Result<Vec<_>, ParsePackageUrlError> = config
        .package_urls
        .into_iter()
        .map(PackageUrl::parse)
        .collect();
    let resolved = ResolvedInstallerConfig {
        reaper_resource_dir,
        reaper_resource_dir_exists: exists,
        portable,
        platform: reaper_platform,
        package_urls: package_urls?,
        num_download_retries: config.num_download_retries.unwrap_or(3),
        temp_parent_dir,
        keep_temp_dir: config.keep_temp_dir,
        concurrent_downloads: config.concurrent_downloads.unwrap_or(5),
        dry_run: config.dry_run,
        reaper_version: config.reaper_version.unwrap_or_default(),
        skip_failed_packages: config.skip_failed_packages,
    };
    Ok(resolved)
}

pub async fn determine_initial_installation_stage(
    resolved_config: &ResolvedInstallerConfig,
) -> anyhow::Result<InstallationStage> {
    let reaper_installed = resolved_config.reaper_resource_dir.contains_reaper_ini();
    if !reaper_installed {
        return Ok(InstallationStage::NothingInstalled);
    };
    // At this point, we can be sure that REAPER is installed
    let reapack_lib_file = reapack_util::get_default_reapack_shared_lib_file(
        &resolved_config.reaper_resource_dir,
        resolved_config.platform,
    );
    if !reapack_lib_file.exists() {
        return Ok(InstallationStage::InstalledReaper);
    }
    let reapack_db_file = resolved_config
        .reaper_resource_dir
        .reapack_registry_db_file();
    if !reapack_db_file.exists() {
        // ReaPack library is installed but the DB file doesn't exist. There's no easy way to
        // find out if the ReaPack installation is up-to-date, so we better download the latest
        // version of ReaPack.
        return Ok(InstallationStage::InstalledReaper);
    }
    let Ok(mut reapack_db) = Database::open(reapack_db_file).await else {
        return Ok(InstallationStage::InstalledReaper);
    };
    match reapack_db.compatibility_info().await? {
        CompatibilityInfo::CompatibleButNeedsMigration => {
            // This is a good indicator that the installed ReaPack version is too old, so we should
            // install the latest (and do a migration later!).
            return Ok(InstallationStage::InstalledReaper);
        }
        CompatibilityInfo::DbTooNew => {
            bail!("This ReaBoot version is too old to handle your installed ReaPack database. Please download the latest ReaBoot version! If this already is the latest ReaBoot version, please send a quick message to info@helgoboss.org!");
        }
        _ => {}
    }
    // At this point, we can be sure that ReaPack is installed
    if !resolved_config.package_urls.is_empty() {
        return Ok(InstallationStage::InstalledReaper);
    }
    // No packages to be installed
    Ok(InstallationStage::Finished)
}
