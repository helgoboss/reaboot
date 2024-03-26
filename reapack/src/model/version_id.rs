use crate::model::{LightPackageId, VersionName};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct LightVersionId<'a> {
    pub package_id: LightPackageId<'a>,
    pub version: &'a VersionName,
}
