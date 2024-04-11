use crate::file_util::get_first_existing_parent_dir;

use crate::api::InstallationStage;
use anyhow::{ensure, Context};
use reaboot_reapack::model::VersionName;
use ref_cast::RefCast;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::{env, fs};
use ts_rs::TS;

#[derive(Clone, Eq, PartialEq, Debug, Serialize, TS, RefCast)]
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

        Ok(Self(absolute_dir))
    }

    /// Returns whether the given directory is a valid REAPER resource directory.
    pub fn contains_reaper_ini(&self) -> bool {
        self.reaper_ini_file().exists()
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.0.join(path)
    }

    pub fn get(&self) -> &Path {
        &self.0
    }

    pub fn into_inner(self) -> PathBuf {
        self.0
    }

    pub fn reaper_ini_file(&self) -> PathBuf {
        self.join(REAPER_INI_FILE_PATH)
    }

    pub fn reaper_install_rev_file(&self) -> PathBuf {
        self.join("reaper-install-rev.txt")
    }

    pub fn read_installed_version(&self) -> Option<VersionName> {
        let rev = fs::read_to_string(self.reaper_install_rev_file()).ok()?;
        rev.trim().parse().ok()
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

    pub fn temp_reaboot_dir(&self) -> PathBuf {
        self.join("ReaBoot")
    }
}

impl From<PathBuf> for ReaperResourceDir {
    fn from(value: PathBuf) -> Self {
        Self(value)
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
