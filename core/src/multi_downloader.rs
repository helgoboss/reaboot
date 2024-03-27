use crate::api::{InstallationStage, MultiDownloadInfo};
use crate::downloader::{Download, Downloader};
use crate::installer::InstallerTask;
use crate::task_tracker::{track_tasks, TaskTrackerListener};
use futures::{stream, StreamExt};
use reaboot_reapack::index::Index;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub struct MultiDownloader {
    downloader: Downloader,
    concurrent_downloads: u32,
}

pub struct DownloadWithPayload<P> {
    pub download: Download,
    pub payload: P,
}

impl<P> DownloadWithPayload<P> {
    pub fn new(download: Download, payload: P) -> Self {
        Self { download, payload }
    }
}

pub struct DownloadError<P> {
    pub download: DownloadWithPayload<P>,
    pub error: anyhow::Error,
}

pub type DownloadResult<P> = Result<DownloadWithPayload<P>, DownloadError<P>>;

impl MultiDownloader {
    pub fn new(downloader: Downloader, concurrent_downloads: u32) -> Self {
        Self {
            downloader,
            concurrent_downloads,
        }
    }

    pub async fn download_multiple<P>(
        &self,
        downloads: impl IntoIterator<Item = DownloadWithPayload<P>>,
        listener: impl TaskTrackerListener<Payload = DownloadWithPayload<P>>,
    ) -> Vec<DownloadResult<P>> {
        let tasks = track_tasks(downloads, listener);
        let download_results: Vec<_> = stream::iter(tasks)
            .enumerate()
            .map(|(i, task)| async move {
                task.start();
                let download_result = self
                    .downloader
                    .download(task.payload.download.clone(), |progress| {
                        let p = progress.to_simple_progress();
                        task.set_progress(p);
                    })
                    .await;
                match download_result {
                    Ok(_) => {
                        task.finish();
                        Ok(task.payload)
                    }
                    Err(e) => {
                        task.fail();
                        let download_error = DownloadError {
                            download: task.payload,
                            error: e,
                        };
                        Err(download_error)
                    }
                }
            })
            .buffer_unordered(self.concurrent_downloads as usize)
            .collect()
            .await;
        download_results
    }
}
