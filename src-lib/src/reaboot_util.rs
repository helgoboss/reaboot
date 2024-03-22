use std::path::Path;
use thiserror::Error;

use crate::api::{InstallationStatus, ReabootConfig, ResolvedReabootConfig};
use crate::reaper_target::ReaperTarget;
use crate::{reapack_util, reaper_util};

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

pub fn determine_initial_installation_status(
    reaper_resource_dir: impl AsRef<Path>,
    reaper_target: ReaperTarget,
) -> InstallationStatus {
    // TODO Maybe change the few sync fs usages to async
    let reaper_resource_dir = reaper_resource_dir.as_ref();
    let reaper_installed = reaper_util::is_valid_reaper_resource_dir(reaper_resource_dir);
    if !reaper_installed {
        return InstallationStatus::Initial;
    };
    let reapack_installed =
        reapack_util::get_default_reapack_shared_lib_file(reaper_resource_dir, reaper_target)
            .exists();
    if !reapack_installed {
        return InstallationStatus::InstalledReaper;
    }
    InstallationStatus::InstalledReaPack
}

#[derive(Error, Debug)]
pub enum ResolveConfigError {
    #[error("Unable to identify home directory")]
    UnableToIdentifyHomeDir,
    #[error("ReaBoot is running on a platform that's not supported by REAPER. It's still possible do do an install for a supported platform but you need to choose it manually.")]
    RequireCustomReaperTarget,
}
