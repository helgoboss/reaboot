use crate::model::{LightPackageId, VersionName};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct LightVersionId<'a> {
    pub package_id: LightPackageId<'a>,
    pub version: &'a VersionName,
}

impl<'a> Display for LightVersionId<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.package_id.package, self.version)
    }
}
