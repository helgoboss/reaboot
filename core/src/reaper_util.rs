use crate::file_util::move_dir_contents;
use crate::reaper_platform::ReaperPlatform;
use anyhow::{anyhow, bail, ensure, Context};

use reaboot_reapack::model::{VersionName, VersionRef};

use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::reaper_resource_dir::ReaperResourceDir;
use url::Url;

const LATEST_STABLE_VERSION_URL: &str = "https://www.cockos.com/reaper/latestversion/";
const LATEST_UNSTABLE_VERSION_URL: &str = "https://www.landoleet.org/whatsnew.txt";
const EULA_URL: &str = "https://www.reaper.fm/license.txt";

/// Returns the expected location of the REAPER main resource directory, even if it doesn't exist.
pub fn get_default_main_reaper_resource_dir() -> anyhow::Result<ReaperResourceDir> {
    let config_dir = dirs::config_dir().context("couldn't identify home directory")?;
    Ok(config_dir.join("REAPER").into())
}

pub struct ReaperInstallerAsset {
    pub version: VersionName,
    pub url: Url,
    pub file_name: String,
}

pub async fn get_latest_reaper_installer_asset(
    reaper_target: ReaperPlatform,
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

/// This extracts REAPER into `dest_dir`.
///
/// For the final (portable) installation step, the **contents** of this directory need to be copied to the REAPER
/// resource directory.
///
/// - On macOS, `dest_dir` will contain just one entry: The `REAPER.app` application bundle directory.
/// - On Windows, `dest_dir` will contain `reaper.exe` and other files and directories.
/// - On Linux, `dest_dir` will contain `reaper.so` and other files and directories.
pub fn extract_reaper_to_portable_dir(
    installer_asset: &Path,
    dest_dir: &Path,
    temp_dir: &Path,
) -> anyhow::Result<()> {
    let extension = installer_asset
        .extension()
        .context("REAPER installer asset doesn't have extension")?
        .to_str()
        .context("REAPER installer asset extension not UTF-8 compatible")?;
    match extension {
        "dmg" => extract_reaper_for_macos_to_dir(installer_asset, dest_dir, temp_dir)?,
        "exe" => extract_reaper_for_windows_to_dir(installer_asset, dest_dir)?,
        "xz" => extract_reaper_for_linux_to_dir(installer_asset, dest_dir, temp_dir)?,
        e => bail!("REAPER installer asset has unsupported file extension {e}"),
    };
    Ok(())
}

fn extract_reaper_for_macos_to_dir(
    dmg_path: &Path,
    dest_dir: &Path,
    temp_dir: &Path,
) -> anyhow::Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        bail!("It's not possible on a non-macOS system to extract REAPER for macOS");
    }
    #[cfg(target_os = "macos")]
    {
        fn convert_dmg_to_img(dmg_path: &Path, img_path: &PathBuf) -> anyhow::Result<()> {
            let dmg_file = File::open(dmg_path).context("couldn't open REAPER DMG file")?;
            let mut dmg_wiz = dmgwiz::DmgWiz::from_reader(dmg_file, dmgwiz::Verbosity::None)
                .map_err(|e| anyhow!("could not read REAPER dmg file: {e}"))?;
            let img_file = File::create(img_path).context("could not create REAPER img file")?;
            dmg_wiz
                .extract_all(BufWriter::new(img_file))
                .map_err(|e| anyhow!("could not extract files from REAPER dmg file: {e}"))?;
            Ok(())
        }

        // Simply attaching the DMG file won't work (maybe because of the license?), so we need
        // to convert it to IMG first.
        let img_path = dmg_path.with_extension("img");
        convert_dmg_to_img(dmg_path, &img_path)?;
        // Now we attach the IMG file
        let mount_dir = temp_dir.join("extracted-dmg");
        let _info = dmg::Attach::new(img_path)
            .mount_root(&mount_dir)
            .hidden()
            .with()
            .context("could not attach REAPER img file")?;
        // And copy all files out of it
        let mounted_reaper_app_dir = mount_dir.join("REAPER_INSTALL_UNIVERSAL/REAPER.app");
        std::fs::create_dir_all(dest_dir)?;
        let dest_reaper_app_dir = dest_dir.join("REAPER.app");
        crate::file_util::copy_dir_recursively(mounted_reaper_app_dir, dest_reaper_app_dir)?;
        Ok(())
    }
}

fn extract_reaper_for_windows_to_dir(
    reaper_installer_exe: &Path,
    dest_dir: &Path,
) -> anyhow::Result<()> {
    ensure!(
        cfg!(target_os = "windows"),
        "It's not possible on a non-Windows system to extract REAPER for Windows"
    );
    let dest_dir_string = dest_dir
        .to_str()
        .context("destination directory for extracting REAPER is not valid UTF-8")?;
    let exit_status = Command::new(reaper_installer_exe)
        .arg("/S")
        .arg("/PORTABLE")
        .arg(format!("/D={dest_dir_string}"))
        .output()
        .context("Error while executing the REAPER installer")?
        .status;
    ensure!(
        exit_status.success(),
        "REAPER installer returned with a non-zero exit code"
    );
    Ok(())
}

fn extract_reaper_for_linux_to_dir(
    reaper_tar_xz: &Path,
    dest_dir: &Path,
    tmp_dir: &Path,
) -> anyhow::Result<()> {
    fs::create_dir_all(tmp_dir)?;
    let exit_status = Command::new("tar")
        .arg("-xf")
        .arg(reaper_tar_xz)
        .arg("-C")
        .arg(tmp_dir)
        .output()
        .context("Error while unpacking the REAPER archive via tar")?
        .status;
    ensure!(exit_status.success(), "tar returned a non-zero exit code");
    let relevant_sub_dir = tmp_dir.join("reaper_linux_x86_64/REAPER");
    move_dir_contents(relevant_sub_dir, dest_dir)?;
    Ok(())
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
                .strip_prefix('v')
                .context("whatsnew.txt should return version starting with letter v")?
                .split_once(' ')
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

pub async fn get_reaper_eula() -> anyhow::Result<String> {
    let response = reqwest::get(EULA_URL).await?;
    let body = response.text().await?;
    Ok(body)
}

fn get_os_specific_reaper_installer_file_name(
    reaper_target: ReaperPlatform,
    version: &VersionName,
) -> String {
    let version = version.to_string().replace('.', "");
    match reaper_target {
        // TODO-medium What about the "macOS 10.5-10.14" download ("reaper711_x86_64.dmg")?
        ReaperPlatform::MacOsAarch64 | ReaperPlatform::MacOsX86_64 => {
            format!("reaper{version}_universal.dmg")
        }
        ReaperPlatform::MacOsX86 => format!("reaper{version}_i386.dmg"),
        ReaperPlatform::WindowsX86 => format!("reaper{version}-install.exe"),
        ReaperPlatform::WindowsX64 => format!("reaper{version}_x64-install.exe"),
        ReaperPlatform::LinuxAarch64 => format!("reaper{version}_linux_aarch64.tar.xz"),
        ReaperPlatform::LinuxArmv7l => format!("reaper{version}_linux_armv7l.tar.xz"),
        ReaperPlatform::LinuxI686 => format!("reaper{version}_linux_i686.tar.xz"),
        ReaperPlatform::LinuxX86_64 => format!("reaper{version}_linux_x86_64.tar.xz"),
    }
}
