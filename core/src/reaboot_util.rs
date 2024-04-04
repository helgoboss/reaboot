use std::env;
use std::path::PathBuf;

use anyhow::{bail, Context};
use url::Url;

use reaboot_reapack::database::{CompatibilityInfo, Database};
use reaboot_reapack::model::{
    PackagePath, PackageUrl, PackageVersionRef, ParsePackageUrlError, VersionRef,
};

use crate::api::{InstallationStage, InstallerConfig, ReabootBackendInfo, ResolvedInstallerConfig};
use crate::file_util::file_or_dir_is_writable_or_creatable;
use crate::reaper_platform::ReaperPlatform;
use crate::reaper_resource_dir::ReaperResourceDir;
use crate::reaper_util;
use crate::recipe::Recipe;

pub fn collect_backend_info() -> ReabootBackendInfo {
    let (main_reaper_resource_dir, main_reaper_ini_exists) =
        if let Ok(dir) = reaper_util::get_default_main_reaper_resource_dir() {
            let ini_exists = dir.contains_reaper_ini();
            (Some(dir.into_inner()), ini_exists)
        } else {
            (None, false)
        };
    let inherent_reaper_platform = ReaperPlatform::from_reaboot_build();
    let main_reaper_exe: PathBuf =
        reaper_util::get_os_specific_main_reaper_exe_path(inherent_reaper_platform).into();

    ReabootBackendInfo {
        main_reaper_resource_dir,
        main_reaper_ini_exists,
        main_reaper_exe_exists: main_reaper_exe.exists(),
        main_reaper_exe,
        inherent_reaper_platform,
    }
}

pub async fn resolve_config(
    config: InstallerConfig,
    recipe: Option<Recipe>,
) -> anyhow::Result<ResolvedInstallerConfig> {
    // Check if this is the main REAPER resource directory
    let main_reaper_resource_dir = reaper_util::get_default_main_reaper_resource_dir()?;
    let (reaper_resource_dir, portable) = if let Some(d) = config.custom_reaper_resource_dir {
        let d = ReaperResourceDir::new(d)?;
        let portable = d != main_reaper_resource_dir;
        (d, portable)
    } else {
        (main_reaper_resource_dir, false)
    };
    // Determine platform
    let reaper_platform = config
        .custom_platform
        .unwrap_or(ReaperPlatform::from_reaboot_build());
    // Determine corresponding REAPER executable
    let reaper_exe = if portable {
        let exe_file_name = reaper_util::get_os_specific_reaper_exe_file_name(reaper_platform);
        reaper_resource_dir.join(exe_file_name)
    } else {
        reaper_util::get_os_specific_main_reaper_exe_path(reaper_platform).into()
    };
    // Check ReaPack DB
    complain_if_reapack_db_too_new(&reaper_resource_dir).await?;
    // Prefer a sub dir of the destination REAPER resource dir as location for all temporary
    // files, e.g. downloads. Rationale: Eventually, all downloaded files need to be moved
    // to their final destination. This move is very fast if it's just a rename, not a copy
    // operation. A rename is only possible if source file and destination directory are on
    // the same mount ... and this is much more likely with this approach than to choose
    // a random OS temp directory.
    let temp_parent_dir = if let Some(d) = config.temp_parent_dir {
        if file_or_dir_is_writable_or_creatable(&d) {
            d
        } else {
            bail!("The custom temp parent dir you have provided is not writable");
        }
    } else {
        let temp_reaboot_dir = reaper_resource_dir.temp_reaboot_dir();
        if file_or_dir_is_writable_or_creatable(&temp_reaboot_dir) {
            temp_reaboot_dir
        } else {
            // Fall back to OS temp dir as parent. This can be useful in dry mode.
            env::temp_dir()
        }
    };
    // Parse user-defined package URLs
    let mut package_urls = parse_package_urls(&config.package_urls)
        .context("couldn't parse user-provided package URLs")?;
    // Add ReaPack package (this is good to have for updates within REAPER and also necessary
    // for scripts being registered at runtime)
    package_urls.push(create_reapack_package_url());
    // Add recipe package URLs
    if let Some(r) = recipe.as_ref() {
        let recipe_package_urls = parse_package_urls(r.package_urls.iter())
            .context("couldn't parse recipe package URls")?;
        package_urls.extend(recipe_package_urls);
    }
    // Create config value
    let resolved = ResolvedInstallerConfig {
        reaper_ini_exists: reaper_resource_dir.contains_reaper_ini(),
        reaper_exe_exists: reaper_exe.exists(),
        reaper_resource_dir,
        reaper_exe,
        portable,
        platform: reaper_platform,
        package_urls,
        num_download_retries: config.num_download_retries.unwrap_or(3),
        temp_parent_dir,
        keep_temp_dir: config.keep_temp_dir,
        concurrent_downloads: config.concurrent_downloads.unwrap_or(5),
        dry_run: config.dry_run,
        reaper_version: config.reaper_version.unwrap_or_default(),
        skip_failed_packages: config.skip_failed_packages,
        recipe,
    };
    Ok(resolved)
}

fn parse_package_urls(
    urls: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<Vec<PackageUrl>, ParsePackageUrlError> {
    urls.into_iter().map(PackageUrl::parse).collect()
}

pub async fn complain_if_reapack_db_too_new(
    reaper_resource_dir: &ReaperResourceDir,
) -> anyhow::Result<()> {
    let reapack_db_file = reaper_resource_dir.reapack_registry_db_file();
    if !reapack_db_file.exists() {
        // ReaPack DB file doesn't exist yet. Good for us.
        return Ok(());
    }
    let Ok(mut reapack_db) = Database::open(reapack_db_file).await else {
        // ReaPack DB exists, but it can't be read ... unsure.
        return Ok(());
    };
    if reapack_db.compatibility_info().await? == CompatibilityInfo::DbTooNew {
        bail!("This ReaBoot version is too old to handle your installed ReaPack database. Please download the latest ReaBoot version! If this already is the latest ReaBoot version, please send a quick message to info@helgoboss.org!");
    }
    Ok(())
}

pub async fn determine_initial_installation_stage(
    resolved_config: &ResolvedInstallerConfig,
) -> anyhow::Result<InstallationStage> {
    let reaper_installed = resolved_config.reaper_exe_exists;
    if !reaper_installed {
        return Ok(InstallationStage::NothingInstalled);
    };
    // At this point, we can be sure that REAPER is installed
    if !resolved_config.package_urls.is_empty() {
        return Ok(InstallationStage::InstalledReaper);
    }
    // No packages to be installed
    Ok(InstallationStage::Finished)
}

fn create_reapack_package_url() -> PackageUrl {
    PackageUrl {
        repository_url: Url::parse("https://reapack.com/index.xml").unwrap(),
        package_version_ref: PackageVersionRef {
            package_path: PackagePath {
                category: "Extensions".to_string(),
                package_name: "ReaPack.ext".to_string(),
            },
            version_ref: VersionRef::Latest,
        },
    }
}
