use std::path::Path;
use thiserror::Error;

use crate::api::{InstallationStatus, ReabootConfig, ResolvedReabootConfig};
use crate::reapack_util::find_reapack_shared_lib_file;
use crate::{reapack_util, reaper_util};

pub fn resolve_config(config: &ReabootConfig) -> Result<ResolvedReabootConfig, ResolveConfigError> {
    let main_reaper_resource_dir = reaper_util::get_default_main_reaper_resource_dir()
        .ok_or(ResolveConfigError::UnableToIdentifyHomeDir)?;
    let resolved_config = match &config.custom_reaper_resource_dir {
        None => ResolvedReabootConfig {
            reaper_resource_dir: main_reaper_resource_dir,
            portable: false,
        },
        Some(dir) => ResolvedReabootConfig {
            reaper_resource_dir: dir.clone(),
            portable: dir != &main_reaper_resource_dir,
        },
    };
    Ok(resolved_config)
}

pub fn determine_initial_installation_status(
    reaper_resource_dir: impl AsRef<Path>,
) -> InstallationStatus {
    // TODO Maybe change the few sync fs usages to async
    let reaper_resource_dir = reaper_resource_dir.as_ref();
    let reaper_installed = reaper_util::is_valid_reaper_resource_dir(reaper_resource_dir);
    if !reaper_installed {
        return InstallationStatus::Initial;
    };
    let reapack_installed = reapack_util::get_default_reapack_shared_lib_file(reaper_resource_dir)
        .map(|f| f.exists())
        .unwrap_or(false);
    if !reapack_installed {
        return InstallationStatus::InstalledReaper;
    }
    InstallationStatus::InstalledReaPack
}

#[derive(Error, Debug)]
pub enum ResolveConfigError {
    #[error("unable to identify home directory")]
    UnableToIdentifyHomeDir,
}
