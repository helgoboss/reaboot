use crate::model::{PackageType, Section, VersionName};

#[derive(Debug)]
pub struct InstalledPackage {
    pub remote: String,
    pub category: String,
    pub package: String,
    pub desc: String,
    pub typ: PackageType,
    pub version: VersionName,
    pub author: String,
    pub files: Vec<InstalledFile>,
}

#[derive(Debug)]
pub struct InstalledFile {
    /// Path relative to REAPER resource folder.
    pub path: String,
    pub section: Option<Section>,
    /// Overrides the package type.
    pub typ: Option<PackageType>,
}
