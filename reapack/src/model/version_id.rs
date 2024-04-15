use crate::model::{LightPackageId, PackageId, VersionName};
use std::fmt::{Display, Formatter};

/// Owned version ID.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VersionId {
    pub package_id: PackageId,
    pub version: VersionName,
}

/// Borrowed version ID.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct LightVersionId<'a> {
    pub package_id: LightPackageId<'a>,
    pub version: &'a VersionName,
}

impl VersionId {
    fn borrowed(&self) -> LightVersionId {
        LightVersionId {
            package_id: self.package_id.to_borrowed(),
            version: &self.version,
        }
    }
}

impl Display for VersionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.borrowed().fmt(f)
    }
}

impl<'a> Display for LightVersionId<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.package_id.package, self.version)
    }
}
