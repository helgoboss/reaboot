pub struct MultiDownloader {
    concurrent_downloads: usize,
}

impl MultiDownloader {
    pub async fn download_multiple(&self) -> anyhow::Result<()> {
        unimplemented!()
    }
}
