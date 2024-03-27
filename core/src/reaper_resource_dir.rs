use crate::file_util::{find_first_existing_parent, get_first_existing_parent_dir};
use crate::reapack_util::get_os_specific_reapack_file_name;
use crate::reaper_target::ReaperTarget;
use anyhow::{ensure, Context};
use ref_cast::RefCast;
use std::env;
use std::path::{Path, PathBuf};

#[derive(RefCast)]
#[repr(transparent)]
pub struct ReaperResourceDir(PathBuf);

impl ReaperResourceDir {
    pub fn new(dir: PathBuf) -> anyhow::Result<Self> {
        let absolute_dir = if dir.exists() {
            // Dir exists
            ensure!(dir.is_dir(), "REAPER resource dir is a file");
            dir.canonicalize()?
        } else if dir.is_absolute() {
            // Dir doesn't exist and is absolute. Make sure that this actually can be
            // a directory.
            get_first_existing_parent_dir(dir.clone())
                .context("Problem with provided REAPER resource directory")?;
            dir
        } else {
            // Dir doesn't exist and is relative. Make it absolute.
            env::current_dir()?.join(dir)
        };

        Ok((Self(absolute_dir)))
    }

    /// Returns whether the given directory is a valid REAPER resource directory.
    pub fn is_valid(&self) -> bool {
        self.reaper_ini_file().exists()
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }

    pub fn get(&self) -> &Path {
        &self.0
    }

    pub fn reaper_ini_file(&self) -> PathBuf {
        self.join(REAPER_INI_FILE_PATH)
    }

    pub fn user_plugins_dir(&self) -> PathBuf {
        self.join("UserPlugins")
    }

    pub fn reapack_dir(&self) -> PathBuf {
        self.join("ReaPack")
    }

    pub fn reapack_cache_dir(&self) -> PathBuf {
        self.join(REAPACK_CACHE_DIR_PATH)
    }

    pub fn reapack_registry_db_file(&self) -> PathBuf {
        self.join(REAPACK_REGISTRY_DB_FILE_PATH)
    }

    pub fn reapack_ini_file(&self) -> PathBuf {
        self.join(REAPACK_INI_FILE_PATH)
    }
}

impl AsRef<Path> for ReaperResourceDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

pub const REAPER_INI_FILE_PATH: &str = "reaper.ini";
pub const REAPACK_REGISTRY_DB_FILE_PATH: &str = "ReaPack/registry.db";
pub const REAPACK_CACHE_DIR_PATH: &str = "ReaPack/Cache";
pub const REAPACK_INI_FILE_PATH: &str = "reapack.ini";