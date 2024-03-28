mod license_agreement;
mod listener;
mod report;

use crate::commands::install::license_agreement::confirm_license;
use crate::commands::install::listener::CliInstallerListener;
use crate::commands::install::report::print_report;
use anyhow::{bail, Context};
use clap::Args;
use reaboot_core::api::ReabootConfig;
use reaboot_core::downloader::Downloader;
use reaboot_core::installer::{InstallError, Installer, InstallerConfig};
use reaboot_core::reaboot_util::resolve_config;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Args)]
#[command(author, version, about, long_about = None)]
pub struct InstallArgs {
    /// Custom REAPER resource directory to be used as destination.
    ///
    /// When provided, ReaBoot automatically assumes that you intend to create or modify
    /// a **portable** REAPER installation.
    ///
    /// If not provided, ReaBoot uses the main REAPER installation.
    #[arg(long)]
    reaper_resource_dir: Option<String>,
    /// Creates the temporary directory for downloads within the given custom directory.
    ///
    /// If not provided, ReaBoot creates the temporary directory in `REAPER_RESOURCE_DIR/ReaBoot`.
    #[arg(long)]
    temp_parent_dir: Option<PathBuf>,
    /// If set, doesn't delete the temporary directory when the installation is finished.
    #[arg(long, default_value_t = false)]
    keep_temp_dir: bool,
    /// Determines the maximum number of concurrent downloads.
    #[arg(long, default_value_t = 5)]
    concurrent_downloads: u32,
    /// If set, skips the last step of actually moving everything to the destination directory.
    #[arg(long, default_value_t = false)]
    dry_run: bool,
    /// If set, skips all license prompts.
    #[arg(long, default_value_t = false)]
    accept_licenses: bool,
    /// If set, ReaBoot won't prompt you for anything. At the moment, this only means automatically
    /// accepting licenses. In the future, there might be more interactivity that this flag will
    /// suppress.
    #[arg(long, default_value_t = false)]
    non_interactive: bool,
    /// If set, packages that couldn't be downloaded or are not installable for other reasons will
    /// be skipped and not considered as failure.
    ///
    /// By default, a failing package would cause ReaBoot to not install anything and exit with
    /// a non-zero exit code.
    #[arg(long, default_value_t = false)]
    skip_failed_packages: bool,
    /// REAPER version to install if REAPER is not yet installed at the destination.
    ///
    /// You can either provide a specific version number (pre-releases are supported as well)
    /// or use `latest` or `latest-pre`.
    #[arg(long, default_value = "latest")]
    reaper_version: String,
    /// URLs of ReaPack packages to be installed.
    #[arg(short, long)]
    package_url: Option<Vec<Url>>,
}

pub async fn install(args: InstallArgs) -> anyhow::Result<()> {
    let config = ReabootConfig {
        custom_reaper_resource_dir: args.reaper_resource_dir.map(PathBuf::from),
        package_urls: args.package_url.unwrap_or_default(),
        custom_reaper_target: None,
    };
    let resolved_config = resolve_config(&config)?;
    let downloader = Downloader::new(args.concurrent_downloads);
    let reaper_version = args
        .reaper_version
        .parse()
        .context("You didn't provide a valid REAPER version string.")?;
    let config = InstallerConfig {
        resolved_config,
        package_urls: config.package_urls,
        downloader,
        temp_parent_dir: args.temp_parent_dir,
        concurrent_downloads: args.concurrent_downloads,
        dry_run: args.dry_run,
        listener: CliInstallerListener::new(),
        keep_temp_dir: args.keep_temp_dir,
        skip_failed_packages: args.skip_failed_packages,
        reaper_version,
    };
    let installer = Installer::new(config).await?;
    // Show REAPER EULA if necessary
    let skip_license_prompts = args.non_interactive || args.accept_licenses;
    if !skip_license_prompts && installer.reaper_is_installable() {
        let initial_stage = installer.determine_initial_installation_stage().await?;
        if initial_stage.is_nothing_installed() {
            if !confirm_license().await? {
                println!("You haven't agreed to the license terms. Exiting.");
                return Ok(());
            }
        }
    }
    // Install everything
    match installer.install().await {
        Ok(r) => {
            print_report(r, !args.dry_run);
        }
        Err(e) => match e {
            InstallError::SomePackagesFailed(r) => {
                print_report(r, false);
                bail!("Installing nothing because not all desired packages can be installed");
            }
            InstallError::Other(e) => {
                Err(e.context("Installation failed"))?;
            }
        },
    }
    Ok(())
}