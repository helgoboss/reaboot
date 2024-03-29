use crate::reaper_resource_dir::ReaperResourceDir;
use crate::reaper_target::ReaperTarget;
use anyhow::Context;
use octocrab::models::repos::{Asset, Release};
use octocrab::Octocrab;

use std::path::PathBuf;

/// Returns the expected OS/architecture-specific location of the ReaPack shared library file.
pub fn get_default_reapack_shared_lib_file(
    reaper_resource_dir: &ReaperResourceDir,
    reaper_target: ReaperTarget,
) -> PathBuf {
    let file_name = get_os_specific_reapack_file_name(reaper_target);
    reaper_resource_dir.user_plugins_dir().join(file_name)
}

/// Returns the location of the first existing ReaPack shared library, no matter OS or architecture.
pub fn find_reapack_shared_lib_file(reaper_resource_dir: &ReaperResourceDir) -> Option<PathBuf> {
    let user_plugins_dir = reaper_resource_dir.user_plugins_dir();
    if !user_plugins_dir.exists() {
        return None;
    }
    user_plugins_dir.read_dir().ok()?.find_map(|entry| {
        let entry = entry.ok()?;
        let file_name = entry.file_name();
        let file_name = file_name.to_str()?;
        if !file_name.starts_with("reaper_reapack") {
            return None;
        }
        Some(entry.path())
    })
}

pub async fn get_latest_reapack_release() -> anyhow::Result<Release> {
    let octocrab = Octocrab::builder().build().unwrap();
    octocrab
        .repos("cfillion", "reapack")
        .releases()
        .get_latest()
        .await
        .context("Couldn't find latest ReaPack release")
}

pub async fn get_correct_reapack_asset(
    release: Release,
    reaper_target: ReaperTarget,
) -> anyhow::Result<Asset> {
    let os_specific_file_name = get_os_specific_reapack_file_name(reaper_target);
    release
        .assets
        .into_iter()
        .find(|asset| asset.name == os_specific_file_name)
        .context("Couldn't find correct download file within GitHub ReaPack asset")
}

pub fn get_os_specific_reapack_file_name(reaper_target: ReaperTarget) -> &'static str {
    match reaper_target {
        ReaperTarget::MacOsAarch64 => "reaper_reapack-arm64.dylib",
        ReaperTarget::MacOsX86 => "reaper_reapack-i386.dylib",
        ReaperTarget::MacOsX86_64 => "reaper_reapack-x86_64.dylib",
        ReaperTarget::WindowsX86 => "reaper_reapack-x86.dll",
        ReaperTarget::WindowsX64 => "reaper_reapack-x64.dll",
        ReaperTarget::LinuxAarch64 => "reaper_reapack-aarch64.so",
        ReaperTarget::LinuxArmv7l => "reaper_reapack-armv7l.so",
        ReaperTarget::LinuxI686 => "reaper_reapack-i686.so",
        ReaperTarget::LinuxX86_64 => "reaper_reapack-x86_64.so",
    }
}
