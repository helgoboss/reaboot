use std::path::{Path, PathBuf};

pub struct ReaperResourceDir<P>(P);

impl<P: AsRef<Path>> ReaperResourceDir<P> {
    pub fn new(dir: P) -> Self {
        Self(dir)
    }

    pub fn get(&self) -> &Path {
        self.0.as_ref()
    }

    pub fn user_plugins(&self) -> PathBuf {
        self.get().join("UserPlugins")
    }

    pub fn reapack(&self) -> PathBuf {
        self.get().join("ReaPack")
    }

    pub fn reapack_cache(&self) -> PathBuf {
        self.reapack().join("Cache")
    }

    pub fn reapack_registry(&self) -> PathBuf {
        self.reapack().join("registry.db")
    }
}

impl<P: AsRef<Path>> AsRef<Path> for ReaperResourceDir<P> {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}
