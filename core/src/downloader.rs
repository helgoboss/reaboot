use crate::hash_util;
use crate::hash_util::ReabootHashVerifier;
use anyhow::Context;
use futures::stream::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use url::Url;

#[derive(Clone)]
pub struct Downloader {
    client: ClientWithMiddleware,
}

#[derive(Clone, Debug)]
pub struct Download {
    pub url: Url,
    pub file: PathBuf,
    pub expected_multihash: Option<String>,
}

impl Download {
    pub fn new(url: Url, file: PathBuf, expected_multihash: Option<String>) -> Self {
        Self {
            url,
            file,
            expected_multihash,
        }
    }
}

impl Downloader {
    pub fn new(retries: u32) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(retries);
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", HeaderValue::from_static("reaboot"));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        let client = reqwest_middleware::ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        Self { client }
    }

    pub async fn download(
        &self,
        download: Download,
        progress_listener: impl Fn(DownloadProgress),
    ) -> anyhow::Result<()> {
        progress_listener(DownloadProgress::Connecting);
        let mut req = self.client.get(download.url.clone());
        let res = req.send().await?;
        progress_listener(DownloadProgress::CreatingDestFile);
        res.error_for_status_ref()?;
        let content_length = get_content_length(res.headers());
        if let Some(dir) = download.file.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut dest_file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&download.file)
            .await?;
        let mut stream = res.bytes_stream();
        let mut bytes_already_downloaded = 0;
        progress_listener(DownloadProgress::Downloading(0.0));
        let mut verifier = if let Some(m) = download.expected_multihash.as_ref() {
            Some(
                ReabootHashVerifier::try_from_hash(m)
                    .context("Download came with a checksum but we have no way to verify it. Discarding download.")?,
            )
        } else {
            None
        };
        while let Some(item) = stream.next().await {
            let mut chunk = item?;
            let chunk_size = chunk.len() as u64;
            if let Some(l) = content_length {
                if l != 0 {
                    let progress = bytes_already_downloaded as f64 / l as f64;
                    progress_listener(DownloadProgress::Downloading(progress));
                }
            }
            bytes_already_downloaded += chunk_size;
            dest_file.write_all_buf(&mut chunk).await?;
            if let Some(verifier) = &mut verifier {
                verifier.update(&chunk);
            }
        }
        if let Some(verifier) = verifier {
            verifier.verify()
                .context("Download came with a checksum but downloaded file has another checksum. Discarding download.")?;
        }
        progress_listener(DownloadProgress::Finished);
        Ok(())
    }
}

fn get_content_length(headers: &HeaderMap) -> Option<u64> {
    headers.get("Content-Length")?.to_str().ok()?.parse().ok()
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DownloadProgress {
    Connecting,
    CreatingDestFile,
    Downloading(f64),
    Finished,
}

impl DownloadProgress {
    pub fn to_simple_progress(&self) -> f64 {
        match self {
            DownloadProgress::Connecting => 0.01,
            DownloadProgress::CreatingDestFile => 0.02,
            DownloadProgress::Downloading(progress) => 0.03 + progress * 0.97,
            DownloadProgress::Finished => 1.0,
        }
    }
}
