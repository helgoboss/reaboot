use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use tempdir::TempDir;

use reaboot_core::api::InstallerConfig;
use reaboot_core::installer::{InstallError, Installer, InstallerNewArgs};

use crate::commands::install::license_agreement::confirm_license;
use crate::commands::install::listener::CliInstallerListener;
use crate::commands::install::report::print_report;

mod license_agreement;
mod listener;
mod report;

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
    reaper_resource_dir: Option<PathBuf>,
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
    /// If set, REAPER will be updated to the latest version.
    #[arg(long, default_value_t = false)]
    update_reaper: bool,
    /// URLs of ReaPack packages to be installed.
    #[arg(short, long)]
    package_url: Option<Vec<String>>,
}

pub async fn install(args: InstallArgs) -> anyhow::Result<()> {
    let reaper_version = args
        .reaper_version
        .parse()
        .context("You didn't provide a valid REAPER version string.")?;
    let config = InstallerConfig {
        custom_reaper_resource_dir: args.reaper_resource_dir,
        custom_platform: None,
        package_urls: args.package_url.unwrap_or_default(),
        num_download_retries: None,
        temp_parent_dir: args.temp_parent_dir,
        installation_id: None,
        keep_temp_dir: args.keep_temp_dir,
        concurrent_downloads: Some(args.concurrent_downloads),
        dry_run: args.dry_run,
        reaper_version: Some(reaper_version),
        update_reaper: args.update_reaper,
        skip_failed_packages: args.skip_failed_packages,
        recipe: None,
        selected_features: Default::default(),
        install_reapack: None,
    };
    let (interaction_sender, interaction_receiver) = tokio::sync::broadcast::channel(10);
    let listener = CliInstallerListener::new(interaction_sender);
    let temp_dir_for_reaper_download = TempDir::new("reaboot-")
        .context("couldn't create temporary directory for REAPER download")?;
    let installer_new_args = InstallerNewArgs {
        config,
        temp_dir_for_reaper_download: temp_dir_for_reaper_download.path().to_path_buf(),
        interactions: interaction_receiver,
        listener,
    };
    let installer = Installer::new(installer_new_args).await?;
    // Show REAPER EULA if necessary
    let skip_license_prompts = args.non_interactive || args.accept_licenses;
    if !skip_license_prompts
        && installer.reaper_is_installable()
        && !installer.resolved_config().reaper_exe_exists
        && !confirm_license().await?
    {
        println!("You haven't agreed to the license terms. Exiting.");
        return Ok(());
    }
    // Install everything
    println!("Starting installation process...\n");
    match installer.install().await {
        Ok(outcome) => {
            print_report(
                &outcome.preparation_report,
                outcome.actually_installed_things,
            );
            if let Some(installer) = outcome.manual_reaper_install_path {
                // This makes the REAPER download temp directory survive.
                temp_dir_for_reaper_download.into_path();
                eprintln!("\nReaBoot couldn't install REAPER automatically. Please do it manually instead! The installer is located here:\n{installer:?}");
            }
        }
        Err(e) => match e {
            InstallError::SomePackagesFailed(r) => {
                print_report(&r, false);
                Err(InstallError::SomePackagesFailed(r))?;
            }
            InstallError::Other(e) => {
                Err(e.context("Installation failed"))?;
            }
        },
    }
    Ok(())
}
