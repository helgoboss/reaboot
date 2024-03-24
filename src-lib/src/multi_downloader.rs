use crate::api::{InstallationStatus, MultiDownloadInfo};
use crate::downloader::{Download, Downloader};
use crate::task_tracker::TaskTracker;
use futures::{stream, StreamExt};
use reaboot_reapack::index::Index;
use std::io::BufReader;
use std::path::PathBuf;
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
        status_listener: impl Fn(MultiDownloadInfo),
        progress_listener: impl Fn(f64),
    ) -> Vec<DownloadResult<P>> {
        let (task_tracker, tasks) = TaskTracker::new(downloads);
        // Keep reporting progress while we are downloading
        let progress_reporting_future = async move {
            loop {
                let summary = task_tracker.summary();
                let multi_download_info = MultiDownloadInfo {
                    in_progress_count: summary.in_progress_count,
                    success_count: summary.success_count,
                    error_count: summary.error_count,
                    total_count: summary.total_count,
                };
                status_listener(multi_download_info);
                progress_listener(summary.total_progress);
                if summary.done() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        };
        // Do the actual downloading in parallel
        let download_future = async {
            let download_results: Vec<_> = stream::iter(tasks)
                .enumerate()
                .map(|(i, task)| async move {
                    task.record.start();
                    let download_result = self
                        .downloader
                        .download(task.payload.download.clone(), |progress| {
                            task.record.set_progress(progress.to_simple_progress());
                        })
                        .await;
                    match download_result {
                        Ok(_) => {
                            task.record.finish();
                            Ok(task.payload)
                        }
                        Err(e) => {
                            task.record.fail();
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
        };
        let (_, download_results) = futures::join!(progress_reporting_future, download_future);
        download_results
    }
}
