use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use reaboot_core::api::{InstallationStatus, ReabootConfig, ResolvedReabootConfig};
use reaboot_core::downloader::Downloader;
use reaboot_core::installer::{Installer, InstallerConfig, InstallerListener};
use reaboot_core::reaboot_util::resolve_config;
use std::path::PathBuf;
use tempdir::TempDir;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = init_tracing();
    let app = App::parse();
    match app.command {
        Command::Install(args) => install(args).await,
    }
}

fn init_tracing() -> anyhow::Result<()> {
    let env_var = std::env::var("REABOOT_LOG")?;
    let env_filter = EnvFilter::new(env_var);
    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_thread_ids(true)
        .with_thread_names(true)
        // .compact()
        .with_env_filter(env_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber).context("setting default subscriber failed")?;
    Ok(())
}

#[derive(Debug, Parser)]
#[clap(version)]
pub struct App {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Install(InstallArgs),
}

#[derive(Debug, Args)]
#[command(author, version, about, long_about = None)]
struct InstallArgs {
    /// Directory of a portable REAPER installation to be used as destination.
    ///
    /// If not provided, ReaBoot uses the main REAPER installation.
    #[arg(long)]
    portable: Option<String>,
    // TODO-medium
    // /// Increases logging output.
    // #[arg(short, long, default_value_t = false)]
    // verbose: bool,
    // /// Prints a detailed report at the end.
    // // TODO-medium
    // #[arg(long, default_value_t = false)]
    // report: bool,
    /// Creates the temporary directory for downloads within the given custom directory.
    ///
    /// If provided, ReaBoot doesn't clean up the downloaded files, so you can freely inspect
    /// what it downloaded.
    ///
    /// If not provided, ReaBoot uses an operating-system-specific temporary parent directory
    /// and clean it up right after using it.
    #[arg(long)]
    temp_parent_dir: Option<String>,
    /// Determines the maximum number of concurrent downloads.
    #[arg(long, default_value_t = 5)]
    concurrent_downloads: u32,
    /// Doesn't actually install anything to the destination directory, but validates and downloads
    /// stuff.
    #[arg(long, default_value_t = false)]
    dry_run: bool,
    /// REAPER version to install if REAPER is not yet installed at the destination.
    ///
    /// You can either provide a specific version number (pre-releases are supported as well)
    /// or use `latest` or `latest-pre`.
    #[arg(long, default_value = "latest")]
    reaper_version: String,
    #[arg(short, long)]
    package_url: Option<Vec<Url>>,
}

async fn install(args: InstallArgs) -> anyhow::Result<()> {
    let config = ReabootConfig {
        custom_reaper_resource_dir: args.portable.map(PathBuf::from),
        package_urls: args.package_url.unwrap_or_default(),
        // TODO-low
        custom_reaper_target: None,
    };
    let resolved_config = resolve_config(&config)?;
    let downloader = Downloader::new(args.concurrent_downloads);
    let temp_dir_prefix = "reaboot-";
    let temp_dir = if let Some(p) = &args.temp_parent_dir {
        tokio::fs::create_dir_all(p).await?;
        TempDir::new_in(p, temp_dir_prefix)
    } else {
        TempDir::new(temp_dir_prefix)
    };
    let temp_dir = temp_dir.context("couldn't create temp directory")?;
    let temp_dir_path = if args.temp_parent_dir.is_some() {
        temp_dir.into_path()
    } else {
        temp_dir.path().to_path_buf()
    };
    let reaper_version = args
        .reaper_version
        .parse()
        .context("You didn't provide a valid REAPER version string.")?;
    let config = InstallerConfig {
        resolved_config,
        package_urls: config.package_urls,
        downloader,
        temp_download_dir: temp_dir_path,
        concurrent_downloads: args.concurrent_downloads,
        dry_run: args.dry_run,
        listener: CliInstallerListener::new(),
        reaper_version,
    };
    let installer = Installer::new(config)?;
    installer.install().await.context("installation failed")?;
    Ok(())
}

struct CliInstallerListener {
    main_progress_bar: ProgressBar,
}

impl CliInstallerListener {
    pub fn new() -> Self {
        let main_progress_bar = ProgressBar::new(100);
        main_progress_bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        Self { main_progress_bar }
    }
}

impl InstallerListener for CliInstallerListener {
    fn emit_installation_status(&self, event: InstallationStatus) {
        self.main_progress_bar.reset();
        self.main_progress_bar.set_message(event.to_string());
    }

    fn emit_progress(&self, progress: f64) {
        self.main_progress_bar
            .set_position((progress * 100.0).round() as u64)
    }

    fn log_write_activity(&self, activity: String) {
        println!("{activity}");
    }
}
