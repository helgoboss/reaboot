use crate::reaper_target::ReaperTarget;
use anyhow::Context;
use octocrab::models::repos::{Asset, Release};
use octocrab::Octocrab;
use reaboot_reapack::model::{VersionName, VersionRef};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::Url;

/// Returns the expected location of the REAPER main resource directory, even if it doesn't exist.
///
/// Returns `None` if the home directory couldn't be identified.
pub fn get_default_main_reaper_resource_dir() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("REAPER"))
}

/// Returns whether the given directory is a valid REAPER resource directory.
pub fn is_valid_reaper_resource_dir(dir: &Path) -> bool {
    dir.join("reaper.ini").exists()
}

pub struct ReaperInstallerAsset {
    pub version: VersionName,
    pub url: Url,
    pub file_name: String,
}

pub async fn get_latest_reaper_installer_asset(
    reaper_target: ReaperTarget,
    version_ref: &VersionRef,
) -> anyhow::Result<ReaperInstallerAsset> {
    let version = resolve_reaper_version(version_ref).await?;
    let major_version = version.major();
    let file_name = get_os_specific_reaper_installer_file_name(reaper_target, &version);
    let url = if version.is_stable() {
        format!("https://www.reaper.fm/files/{major_version}.x/{file_name}")
    } else {
        format!("https://www.landoleet.org/{file_name}")
    };
    let asset = ReaperInstallerAsset {
        version,
        url: Url::parse(&url)?,
        file_name,
    };
    Ok(asset)
}

/// REAPER versions seem to be similar to ReaPack versions in nature.
async fn resolve_reaper_version(version_ref: &VersionRef) -> anyhow::Result<VersionName> {
    let include_pre_releases = match version_ref {
        VersionRef::Latest => false,
        VersionRef::LatestPre => true,
        VersionRef::Specific(v) => return Ok(v.clone()),
    };
    let stable_version =
        get_latest_reaper_version_from_url(LATEST_STABLE_VERSION_URL, |line| Ok(line)).await?;
    if !include_pre_releases {
        return Ok(stable_version);
    }
    let unstable_version =
        get_latest_reaper_version_from_url(LATEST_UNSTABLE_VERSION_URL, |line| {
            let (version, _) = line
                .strip_prefix("v")
                .context("whatsnew.txt should return version starting with letter v")?
                .split_once(" ")
                .context("whatsnew.txt should contain space after version string")?;
            Ok(version)
        })
        .await?;
    let final_version = if unstable_version > stable_version {
        unstable_version
    } else {
        stable_version
    };
    Ok(final_version)
}

async fn get_latest_reaper_version_from_url(
    url: &str,
    narrow_down_version_line: impl FnOnce(&str) -> anyhow::Result<&str>,
) -> anyhow::Result<VersionName> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let version_line = body.lines().next().with_context(|| {
        format!("{url} should return version number somewhere contained in first line")
    })?;
    let raw_version = narrow_down_version_line(version_line)?;
    let version = raw_version
        .parse()
        .with_context(|| format!("{url} has not returned a valid REAPER version"))?;
    Ok(version)
}

fn get_os_specific_reaper_installer_file_name(
    reaper_target: ReaperTarget,
    version: &VersionName,
) -> String {
    let version = version.to_string().replace(".", "");
    match reaper_target {
        // TODO What about the "macOS 10.5-10.14" download ("reaper711_x86_64.dmg")?
        ReaperTarget::MacOsAarch64 | ReaperTarget::MacOsX86_64 => {
            format!("reaper{version}_universal.dmg")
        }
        ReaperTarget::MacOsX86 => format!("reaper{version}_i386.dmg"),
        ReaperTarget::WindowsX86 => format!("reaper{version}-install.exe"),
        ReaperTarget::WindowsX64 => format!("reaper{version}_x64-install.exe"),
        ReaperTarget::LinuxAarch64 => format!("reaper{version}_linux_aarch64.tar.xz"),
        ReaperTarget::LinuxArmv7l => format!("reaper{version}_linux_armv7l.tar.xz"),
        ReaperTarget::LinuxI686 => format!("reaper{version}_linux_i686.tar.xz"),
        ReaperTarget::LinuxX86_64 => format!("reaper{version}_linux_x86_64.tar.xz"),
    }
}

const LATEST_STABLE_VERSION_URL: &str = "https://www.cockos.com/reaper/latestversion/";
const LATEST_UNSTABLE_VERSION_URL: &str = "https://www.landoleet.org/whatsnew.txt";
