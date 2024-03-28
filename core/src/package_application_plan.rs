use crate::multi_downloader::DownloadWithPayload;
use crate::package_download_plan::QualifiedSource;
use reaboot_reapack::model::{InstalledPackage, LightPackageId, LightVersionId};

#[derive(Default)]
pub struct PackageApplicationPlan<'a> {
    pub package_failures: Vec<TempInstallFailure<'a>>,
    pub package_applications: Vec<PackageApplication<'a>>,
}

pub struct TempInstallFailure<'a> {
    pub version_id: LightVersionId<'a>,
    pub error: anyhow::Error,
}

pub struct PackageApplication<'a> {
    pub version_id: LightVersionId<'a>,
    pub to_be_moved: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
    pub to_be_removed: Option<&'a InstalledPackage>,
}
