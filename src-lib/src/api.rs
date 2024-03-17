use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum WorkerCommand {
    Install,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
#[serde(tag = "kind")]
pub enum InstallationStatusEvent {
    Idle,
    DownloadingReaper { file: RemoteFile },
    DownloadingReaPack { file: RemoteFile },
    InitializingReaPack,
    DownloadingRepositoryIndex { file: RemoteFile },
    DownloadingPackageFile { file: RemoteFile, package: Package },
    InstallingPackage { package: Package },
    Done,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct RemoteFile {
    pub label: String,
    pub url: String,
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct Package {
    pub name: String,
    pub desc: String,
    pub version: String,
    pub author: String,
}
