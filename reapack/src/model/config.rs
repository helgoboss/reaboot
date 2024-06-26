use anyhow::Context;
use indexmap::IndexMap;
use ini::{EscapePolicy, Ini, LineSeparator, ParseOption, Properties, WriteOption};
use std::fs;

use encoding_rs::Encoding;
use std::path::Path;

use url::Url;

/// This is the currently supported ReaPack config version.
///
/// For ReaBoot, that means:
///
/// - If the INI file doesn't exist yet, it will create a new one with that version and
///   a *minimal* INI schema corresponding to that version. If the latest ReaPack version is
///   made for a newer config version, it's still okay. ReaPack will carry out the necessary
///   migration. ReaBoot will still continue to operate.
/// - If the INI file exists already and its version is *lower* than the one defined here,
///   ReaBoot will apply the same INI file migrations as ReaPack would do (restoring repos) and
///   raise the version to the one defined here.
/// - If the INI file exists already and its user version is *greater* than the one defined here,
///   ReaBoot will continue to operate because breaking INI schema changes don't
///   seem to be part of ReaPack's plan, it's more about stuff like restoring repositories etc.
pub const REAPACK_CONFIG_VERSION: u32 = 4;

/// ReaPack configuration that's typically saved in the "reapack.ini" file.
///
/// At the moment, this contains just those properties that are relevant for ReaBoot.
/// Other properties will not be touched.
pub struct Config {
    pub encoding: &'static Encoding,
    pub general_version: u32,
    pub remote_by_name: IndexMap<String, Remote>,
}

pub struct Remote {
    pub name: String,
    pub url: Url,
    pub enabled: bool,
    /// `None` means it uses the global setting.
    pub auto_install: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            encoding: get_os_encoding(),
            general_version: REAPACK_CONFIG_VERSION,
            remote_by_name: create_default_remotes()
                .map(|r| (r.name.clone(), r))
                .collect(),
        }
    }
}

/// This must correspond to ReaPack's default remotes aligned with
/// [`REAPACK_CONFIG_VERSION`]. In ReaPack's code, they are usually
/// [here](https://github.com/cfillion/reapack/blob/master/src/config.cpp#L100).
const DEFAULT_REMOTES: &[(&str, &str)] = &[
    // Don't forget this one!
    ("ReaPack", "https://reapack.com/index.xml"),
    (
        "ReaTeam Scripts",
        "https://github.com/ReaTeam/ReaScripts/raw/master/index.xml",
    ),
    (
        "ReaTeam JSFX",
        "https://github.com/ReaTeam/JSFX/raw/master/index.xml",
    ),
    (
        "ReaTeam Themes",
        "https://github.com/ReaTeam/Themes/raw/master/index.xml",
    ),
    (
        "ReaTeam LangPacks",
        "https://github.com/ReaTeam/LangPacks/raw/master/index.xml",
    ),
    (
        "ReaTeam Extensions",
        "https://github.com/ReaTeam/Extensions/raw/master/index.xml",
    ),
    (
        "MPL Scripts",
        "https://github.com/MichaelPilyavskiy/ReaScripts/raw/master/index.xml",
    ),
    (
        "X-Raym Scripts",
        "https://github.com/X-Raym/REAPER-ReaScripts/raw/master/index.xml",
    ),
];

impl Config {
    pub fn load_from_ini_file(path: &Path) -> anyhow::Result<Self> {
        let (ini, encoding) = load_ini(path)?;
        Ok(Self::from_ini(&ini, encoding))
    }

    fn from_ini(ini: &Ini, encoding: &'static Encoding) -> Self {
        let general_version = ini
            .get_from(GENERAL_INI_SECTION, "version")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let remotes = ini
            .section(REMOTES_INI_SECTION)
            .map(get_remotes_from_props)
            .unwrap_or_default();
        Self {
            encoding,
            general_version,
            remote_by_name: remotes,
        }
    }

    pub fn apply_to_ini_file(&self, path: &Path) -> anyhow::Result<()> {
        let (mut ini, encoding) =
            load_ini(path).unwrap_or_else(|_| (Ini::new(), get_os_encoding()));
        self.apply_to_ini(&mut ini);
        // This could be done more efficient, but for ReaBoot's purpose it's totally okay
        let mut ini_utf8_bytes = Vec::new();
        ini.write_to_opt(
            &mut ini_utf8_bytes,
            WriteOption {
                escape_policy: EscapePolicy::Basics,
                line_separator: LineSeparator::SystemDefault,
                kv_separator: "=",
            },
        )?;
        let ini_text = String::from_utf8(ini_utf8_bytes)?;
        let (bytes, _, _) = encoding.encode(&ini_text);
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn apply_to_ini(&self, ini: &mut Ini) {
        ini.set_to(
            GENERAL_INI_SECTION,
            "version".to_string(),
            self.general_version.to_string(),
        );
        ini.delete(REMOTES_INI_SECTION);
        for (i, remote) in self.remote_by_name.values().enumerate() {
            ini.set_to(
                REMOTES_INI_SECTION,
                format!("remote{i}"),
                remote.to_ini_value(),
            );
        }
        ini.set_to(
            REMOTES_INI_SECTION,
            "size".to_string(),
            self.remote_by_name.len().to_string(),
        );
    }

    /// Migrates configuration if necessary and returns `true` if it did.
    ///
    /// See https://github.com/cfillion/reapack/blob/master/src/config.cpp, method `migrate`.
    pub fn migrate(&mut self) -> bool {
        if self.general_version <= 3 {
            self.restore_default_remotes();
            self.general_version = REAPACK_CONFIG_VERSION;
            true
        } else {
            false
        }
    }

    pub fn add_remote(&mut self, remote: Remote) {
        self.remote_by_name.insert(remote.name.clone(), remote);
    }

    fn restore_default_remotes(&mut self) {
        for remote in create_default_remotes() {
            self.add_remote(remote);
        }
    }
}

fn load_ini(path: &Path) -> anyhow::Result<(Ini, &'static Encoding)> {
    let encoding = get_os_encoding();
    let bytes = fs::read(path).context("couldn't read reapack.ini file")?;
    let (ini_text, encoding, _) = encoding.decode(&bytes);
    let ini = Ini::load_from_str_opt(
        &ini_text,
        ParseOption {
            enabled_quote: false,
            enabled_escape: false,
        },
    )?;
    Ok((ini, encoding))
}

impl Remote {
    pub fn from_ini_value(line: &str) -> anyhow::Result<Self> {
        let mut split = line.split('|');
        let name = split.next().context("remote should have a name")?;
        let url = split
            .next()
            .context("remote should have a URL")?
            .parse()
            .context("remote URL is invalid")?;
        let enabled = split.next().map(|s| s == "1").unwrap_or(true);
        let auto_install = split.next().and_then(|s| match s {
            "0" => Some(false),
            "1" => Some(true),
            _ => None,
        });
        let remote = Self {
            name: name.to_string(),
            url,
            enabled,
            auto_install,
        };
        Ok(remote)
    }

    pub fn to_ini_value(&self) -> String {
        let enabled_int: u32 = self.enabled.into();
        let auto_install_int: u32 = self.auto_install.map(|v| v.into()).unwrap_or(2);
        format!(
            "{}|{}|{enabled_int}|{auto_install_int}",
            self.name, self.url
        )
    }
}

fn get_remotes_from_props(props: &Properties) -> IndexMap<String, Remote> {
    let size: u32 = props.get("size").and_then(|s| s.parse().ok()).unwrap_or(0);
    (0..size)
        .filter_map(|i| {
            let value = props.get(format!("remote{i}"))?;
            let remote = Remote::from_ini_value(value).ok()?;
            Some((remote.name.clone(), remote))
        })
        .collect()
}

fn create_default_remotes() -> impl Iterator<Item = Remote> {
    DEFAULT_REMOTES.iter().map(|(name, url)| Remote {
        name: name.to_string(),
        url: Url::parse(url).unwrap(),
        enabled: true,
        auto_install: None,
    })
}

const GENERAL_INI_SECTION: Option<&str> = Some("general");
const REMOTES_INI_SECTION: Option<&str> = Some("remotes");

fn get_os_encoding() -> &'static Encoding {
    #[cfg(windows)]
    {
        let page = unsafe { windows::Win32::Globalization::GetACP() };
        // https://learn.microsoft.com/en-us/windows/win32/intl/code-page-identifiers
        let encoding = match page {
            874 => encoding_rs::WINDOWS_874,
            1250 => encoding_rs::WINDOWS_1250,
            1251 => encoding_rs::WINDOWS_1251,
            1252 => encoding_rs::WINDOWS_1252,
            1253 => encoding_rs::WINDOWS_1253,
            1254 => encoding_rs::WINDOWS_1254,
            1255 => encoding_rs::WINDOWS_1255,
            1256 => encoding_rs::WINDOWS_1256,
            1257 => encoding_rs::WINDOWS_1257,
            1258 => encoding_rs::WINDOWS_1258,
            // Not sure if the following ISO code pages can even be returned, but better be prepared
            28591 => encoding_rs::WINDOWS_1252,
            28592 => encoding_rs::ISO_8859_2,
            28593 => encoding_rs::ISO_8859_3,
            28594 => encoding_rs::ISO_8859_4,
            28595 => encoding_rs::ISO_8859_5,
            28596 => encoding_rs::ISO_8859_6,
            28597 => encoding_rs::ISO_8859_7,
            28598 => encoding_rs::ISO_8859_8,
            28599 => encoding_rs::WINDOWS_1254,
            28603 => encoding_rs::ISO_8859_13,
            28605 => encoding_rs::ISO_8859_15,
            38598 => encoding_rs::ISO_8859_8,
            50220 | 50221 | 50222 => encoding_rs::ISO_2022_JP,
            51932 => encoding_rs::EUC_JP,
            51949 => encoding_rs::EUC_KR,
            20866 => encoding_rs::KOI8_R,
            _ => encoding_rs::UTF_8,
        };
        println!(
            "Identified Windows code page {page}. Using encoding {} for loading/saving INI file.",
            encoding.name()
        );
        encoding
    }
    #[cfg(not(windows))]
    {
        encoding_rs::UTF_8
    }
}
