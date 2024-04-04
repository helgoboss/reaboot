use std::env;

pub fn extract_recipe_id_from_current_exe() -> Option<String> {
    let exe_path = env::current_exe().ok()?;
    #[cfg(target_os = "macos")]
    {
        let output = crate::hdi_util::exec_hdi_util_info().ok()?;
        let dmg_path = crate::hdi_util::extract_path_of_corresponding_dmg(exe_path, &output)?;
        let dmg_file_name = dmg_path.file_name()?;
        let recipe_id = extract(dmg_file_name)?;
        Some(recipe_id.to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let exe_file_name = exe_path.file_name()?.to_str()?;
        let recipe_id = extract(exe_file_name)?;
        Some(recipe_id.to_string())
    }
}

fn extract(file_name: &str) -> Option<&str> {
    let start = file_name.strip_prefix("reaboot-").unwrap_or(file_name);
    let suffix_index = start.find("-installer")?;
    Some(&start[..suffix_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(extract("reaboot-helgobox-installer.exe"), Some("helgobox"));
        assert_eq!(
            extract("blub-helgobox-playtime-installer.exe"),
            Some("blub-helgobox-playtime")
        );
        assert_eq!(extract("helgobox-installer.exe"), Some("helgobox"));
        assert_eq!(
            extract("reaboot-helgobox-playtime-installer.exe"),
            Some("helgobox-playtime")
        );
        assert_eq!(
            extract("helgobox-playtime-installer.exe"),
            Some("helgobox-playtime")
        );
        assert_eq!(
            extract("helgobox-playtime-installer"),
            Some("helgobox-playtime")
        );
        assert_eq!(
            extract("reaboot-helgobox-playtime-installer"),
            Some("helgobox-playtime")
        );
        assert_eq!(
            extract("reaboot-helgobox-playtime-installer (2).exe"),
            Some("helgobox-playtime")
        );
        assert_eq!(
            extract("helgobox-playtime-installer (1).exe"),
            Some("helgobox-playtime")
        );
        assert_eq!(extract("reaboot-helgobox-playtime.exe"), None);
        assert_eq!(extract("helgobox-playtime.exe"), None);
    }
}
