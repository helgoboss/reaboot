use std::path::{Path, PathBuf};

pub struct ReaperResourceDir<P>(P);

impl<P: AsRef<Path>> ReaperResourceDir<P> {
    pub fn new(dir: P) -> Self {
        Self(dir)
    }

    pub fn get(&self) -> &Path {
        self.0.as_ref()
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

impl<P: AsRef<Path>> AsRef<Path> for ReaperResourceDir<P> {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
