use anyhow::{anyhow, bail};
use reaboot_reapack::database::{CompatibilityInfo, Database, REAPACK_DB_USER_VERSION};
use std::path::Path;
use thiserror::Error;

use crate::api::{InstallationStage, ReabootConfig, ResolvedReabootConfig};
use crate::reaper_resource_dir::ReaperResourceDir;
use crate::reaper_target::ReaperTarget;
use crate::{reapack_util, reaper_resource_dir, reaper_util};

pub fn resolve_config(config: &ReabootConfig) -> Result<ResolvedReabootConfig, ResolveConfigError> {
    let main_reaper_resource_dir = reaper_util::get_default_main_reaper_resource_dir()
        .ok_or(ResolveConfigError::UnableToIdentifyHomeDir)?;
    let reaper_target = config
        .custom_reaper_target
        .or(ReaperTarget::BUILD)
        .ok_or(ResolveConfigError::RequireCustomReaperTarget)?;
    let resolved_config = match &config.custom_reaper_resource_dir {
        None => ResolvedReabootConfig {
            reaper_resource_dir: main_reaper_resource_dir,
            portable: false,
            reaper_target,
        },
        Some(dir) => ResolvedReabootConfig {
            reaper_resource_dir: dir.clone(),
            portable: dir != &main_reaper_resource_dir,
            reaper_target,
        },
    };
    Ok(resolved_config)
}

pub async fn determine_initial_installation_status(
    reaper_resource_dir: &ReaperResourceDir,
    reaper_target: ReaperTarget,
) -> anyhow::Result<InstallationStage> {
    let reaper_installed = reaper_resource_dir.is_valid();
    if !reaper_installed {
        return Ok(InstallationStage::Initial);
    };
    let reapack_lib_file =
        reapack_util::get_default_reapack_shared_lib_file(reaper_resource_dir, reaper_target);
    if !reapack_lib_file.exists() {
        return Ok(InstallationStage::InstalledReaper);
    }
    let reapack_db_file = reaper_resource_dir.reapack_registry_db_file();
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
        CompatibilityInfo::PerfectlyCompatible | CompatibilityInfo::DbNewerButCompatible => {
            Ok(InstallationStage::InstalledReaPack)
        }
        CompatibilityInfo::CompatibleButNeedsMigration => {
            // This is a good indicator that the installed ReaPack version is too old, so we should
            // install the latest (and do a migration later!).
            Ok(InstallationStage::InstalledReaper)
        }
        CompatibilityInfo::DbTooNew => {
            bail!("This ReaBoot version is too old to handle your installed ReaPack database. Please download the latest ReaBoot version! If this already is the latest ReaBoot version, please send a quick message to info@helgoboss.org!");
        }
    }
}

#[derive(Error, Debug)]
pub enum ResolveConfigError {
    #[error("Unable to identify home directory")]
    UnableToIdentifyHomeDir,
    #[error("ReaBoot is running on a platform that's not supported by REAPER. It's still possible do do an install for a supported platform but you need to choose it manually.")]
    RequireCustomReaperTarget,
}
