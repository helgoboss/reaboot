use crate::reapack_util::get_os_specific_reapack_file_name;
use crate::reaper_target::ReaperTarget;
use ref_cast::RefCast;
use std::path::{Path, PathBuf};

#[derive(RefCast)]
#[repr(transparent)]
pub struct ReaperResourceDir(PathBuf);

impl ReaperResourceDir {
    pub fn new(dir: PathBuf) -> Self {
        Self(dir)
    }

    /// Returns whether the given directory is a valid REAPER resource directory.
    pub fn is_valid(&self) -> bool {
        self.reaper_ini_file().exists()
    }

    pub fn get(&self) -> &Path {
        &self.0
    }

    pub fn reaper_ini_file(&self) -> PathBuf {
        self.get().join("reaper.ini")
    }

    pub fn user_plugins_dir(&self) -> PathBuf {
        self.get().join("UserPlugins")
    }

    pub fn reapack_dir(&self) -> PathBuf {
        self.get().join("ReaPack")
    }

    pub fn reapack_cache_dir(&self) -> PathBuf {
        self.reapack_dir().join("Cache")
    }

    pub fn reapack_registry_db_file(&self) -> PathBuf {
        self.reapack_dir().join("registry.db")
    }

    pub fn reapack_ini_file(&self) -> PathBuf {
        self.get().join("reapack.ini")
    }
}

impl AsRef<Path> for ReaperResourceDir {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
