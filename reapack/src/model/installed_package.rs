use crate::model::package_id::LightPackageId;
use crate::model::{PackageType, Section, VersionName};
use enumset::EnumSet;

use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct InstalledPackage {
    pub remote: String,
    pub category: String,
    pub package: String,
    pub desc: String,
    pub typ: InstalledPackageType,
    pub version: InstalledVersionName,
    pub author: String,
    pub files: Vec<InstalledFile>,
}

#[derive(Debug)]
pub struct InstalledFile {
    /// Path relative to REAPER resource folder.
    pub path: String,
    /// `None` means implicit section (= -1 in database = "true" in index)
    pub sections: Option<EnumSet<Section>>,
    /// Overrides the package type.
    pub typ: Option<InstalledPackageType>,
}

/// When loading packages from an existing ReaPack database, we don't want to fail just because
/// we encounter an invalid version name (even though it's very unlikely to encounter one).
/// The version name is not relevant to ReaBoot anyway because all it needs to know is the location
/// of currently ReaPack-managed files.
#[derive(Clone, Debug)]
pub enum InstalledVersionName {
    Valid(VersionName),
    Invalid(String),
}

/// When loading packages from an existing ReaPack database, we don't want to fail just because
/// we encounter an unknown package type. For ReaBoot, it's not relevant to understand the
/// package type coming from the DB because it reads packages from the DB just for the purpose
/// of checking which file locations are currently managed by ReaPack. And these file locations
/// are available explicitly in the `path` column.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InstalledPackageType {
    Known(PackageType),
    Unknown(i32),
}

impl InstalledPackage {
    pub fn package_id(&self) -> LightPackageId {
        LightPackageId {
            remote: &self.remote,
            category: &self.category,
            package: &self.package,
        }
    }
}

impl Display for InstalledPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.package, &self.version)
    }
}

impl Display for InstalledVersionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstalledVersionName::Valid(v) => v.fmt(f),
            InstalledVersionName::Invalid(v) => v.fmt(f),
        }
    }
}
