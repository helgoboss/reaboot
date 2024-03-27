use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use reaboot_core::api::{InstallationStage, ReabootConfig, ResolvedReabootConfig};
use reaboot_core::downloader::Downloader;
use reaboot_core::installer::{Installer, InstallerConfig, InstallerListener, InstallerTask};
use reaboot_core::reaboot_util::resolve_config;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::RwLock;
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
    tracing::subscriber::set_global_default(subscriber)
        .context("setting default subscriber failed")?;
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
    multi_progress: MultiProgress,
    main_progress_bar: ProgressBar,
    task_progress_bars: RwLock<HashMap<u32, ProgressBar>>,
}

impl CliInstallerListener {
    pub fn new() -> Self {
        let multi_progress = MultiProgress::new();
        let main_progress_bar = multi_progress.add(create_main_progress_bar());
        main_progress_bar.tick();
        Self {
            multi_progress,
            main_progress_bar,
            task_progress_bars: Default::default(),
        }
    }

    fn log(&self, msg: impl Display) {
        // let _ = self.multi_progress.println(msg.to_string());
    }
}

impl InstallerListener for CliInstallerListener {
    fn installation_stage_changed(&self, event: InstallationStage) {
        // self.main_progress_bar.finish();
        self.main_progress_bar.set_message(event.to_string());
        // for (_, pb) in self.task_progress_bars.write().unwrap().drain() {
        //     pb.finish();
        // }
    }

    fn installation_stage_progressed(&self, progress: f64) {
        self.main_progress_bar
            .set_position(convert_progress(progress));
    }

    fn task_started(&self, task_id: u32, task: InstallerTask) {
        let pb = self
            .multi_progress
            .add(create_task_progress_bar(task_id, task));
        pb.tick();
        self.task_progress_bars.write().unwrap().insert(task_id, pb);
    }

    fn task_progressed(&self, task_id: u32, progress: f64) {
        let map = self.task_progress_bars.read().unwrap();
        if let Some(pb) = map.get(&task_id) {
            pb.set_position(convert_progress(progress));
        }
    }

    fn task_finished(&self, task_id: u32) {
        if let Some(pb) = self.task_progress_bars.write().unwrap().get(&task_id) {
            pb.finish();
            // self.multi_progress.remove(&pb);
        }
    }

    fn warn(&self, message: impl Display) {
        self.log(message);
    }

    fn info(&self, message: impl Display) {
        self.log(message);
    }

    fn debug(&self, message: impl Display) {
        self.log(message);
    }
}

fn create_main_progress_bar() -> ProgressBar {
    let main_progress_bar = ProgressBar::new(100);
    main_progress_bar.set_draw_target(ProgressDrawTarget::hidden());
    main_progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    main_progress_bar
}

fn create_task_progress_bar(task_id: u32, task: InstallerTask) -> ProgressBar {
    let pb = ProgressBar::new(100);
    pb.set_draw_target(ProgressDrawTarget::hidden());
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.red/green} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    pb.set_message(format!("{}. {}", task_id + 1, task.label));
    pb
}

fn convert_progress(progress: f64) -> u64 {
    (progress * 100.0).round() as u64
}
