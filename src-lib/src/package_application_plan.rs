use crate::multi_downloader::DownloadWithPayload;
use crate::package_download_plan::QualifiedSource;
use reaboot_reapack::model::{InstalledPackage, LightPackageId};

#[derive(Default)]
pub struct PackageApplicationPlan<'a> {
    pub package_applications: Vec<PackageApplication<'a>>,
}

pub struct PackageApplication<'a> {
    pub package_id: LightPackageId<'a>,
    pub to_be_moved: Vec<DownloadWithPayload<QualifiedSource<'a>>>,
    pub to_be_removed: Option<&'a InstalledPackage>,
}