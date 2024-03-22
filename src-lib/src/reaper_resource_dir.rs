use std::path::{Path, PathBuf};

pub struct ReaperResourceDir<P>(P);

impl<P: AsRef<Path>> ReaperResourceDir<P> {
    pub fn new(dir: P) -> Self {
        Self(dir)
    }

    pub fn get(&self) -> &Path {
        self.0.as_ref()
    }

    pub fn automation_items(&self) -> PathBuf {
        self.get().join("AutomationItems")
    }

    pub fn color_themes(&self) -> PathBuf {
        self.get().join("ColorThemes")
    }

    pub fn data(&self) -> PathBuf {
        self.get().join("Data")
    }

    pub fn effects(&self) -> PathBuf {
        self.get().join("Effects")
    }

    pub fn lang_pack(&self) -> PathBuf {
        self.get().join("LangPack")
    }

    pub fn midi_note_names(&self) -> PathBuf {
        self.get().join("MIDINoteNames")
    }

    pub fn project_templates(&self) -> PathBuf {
        self.get().join("ProjectTemplates")
    }

    pub fn scripts(&self) -> PathBuf {
        self.get().join("Scripts")
    }

    pub fn track_templates(&self) -> PathBuf {
        self.get().join("TrackTemplates")
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
