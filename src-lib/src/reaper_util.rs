use std::path::{Path, PathBuf};

/// Returns the expected location of the REAPER main resource directory, even if it doesn't exist.
///
/// Returns `None` if the home directory couldn't be identified.
pub fn get_default_main_reaper_resource_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("REAPER"))
}

/// Returns whether the given directory is a valid REAPER resource directory.
pub fn is_valid_reaper_resource_dir(dir: &Path) -> bool {
    dir.join("reaper.ini").exists()
}
