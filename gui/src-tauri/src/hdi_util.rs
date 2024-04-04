use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

pub fn exec_hdi_util_info() -> anyhow::Result<Vec<u8>> {
    let output = Command::new("hdiutil").arg("info").arg("-plist").output()?;
    Ok(output.stdout)
}

pub fn extract_path_of_corresponding_dmg(
    path_to_exe: impl AsRef<Path>,
    hdi_util_info_plist_output: &[u8],
) -> Option<Utf8PathBuf> {
    let path_to_exe = Utf8Path::from_path(path_to_exe.as_ref())?;
    let info: Info = plist::from_bytes(hdi_util_info_plist_output).ok()?;
    let image = info
        .images
        .into_iter()
        .find(|image| image.provides_mount_point_for(path_to_exe))?;
    image.image_path
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Info {
    images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Image {
    image_path: Option<Utf8PathBuf>,
    system_entities: Vec<SystemEntity>,
}

impl Image {
    fn provides_mount_point_for(&self, file_path: &Utf8Path) -> bool {
        self.system_entities
            .iter()
            .flat_map(|e| &e.mount_point)
            .any(|mp| file_path.starts_with(mp))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SystemEntity {
    mount_point: Option<Utf8PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let dmg_path = extract_path_of_corresponding_dmg(
            "/Volumes/reaboot/reaboot.app/Contents/MacOS/reaboot",
            include_bytes!("tests/hdi-util-info.plist"),
        );
        assert_eq!(
            dmg_path.unwrap().as_str(),
            "/Users/helgoboss/Documents/projects/dev/reaboot/target/release/bundle/dmg/reaboot-helgobox-installer.dmg"
        );
    }
}
