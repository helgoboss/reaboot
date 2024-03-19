use crate::VersionName;
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer};
use time::OffsetDateTime;

/// This is the root element of any ReaPack index file.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Index {
    /// Must be "1".
    version: String,
    /// Display name of the repository (must contain filename-friendly characters only).
    ///
    /// Required for import.
    name: Option<String>,
    #[serde(default)]
    #[serde(rename = "$value")]
    entries: Vec<IndexEntry>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum IndexEntry {
    /// Many
    Category(Category),
    /// Zero or one
    Metadata(Metadata),
}

/// Used both for organization in the package list and for specifying subdirectories of script and
/// effect packages.
///
/// The target directory can also be changed using the `file` attribute of the [`source`](Source)
/// element.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Category {
    /// Path divided with slashes
    name: String,
    #[serde(default)]
    #[serde(rename = "$value")]
    reapacks: Vec<Reapack>,
}

/// Represents a single package, made of multiple versions, each consisting of one
/// or more files.
///
/// Packages meeting any of these conditions are silently discarded:
///
/// - The package type is invalid/unsupported
/// - The package contains no version
/// - None of the versions have files (`source` elements) compatible with the current platform
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Reapack {
    /// Default filename of the package
    name: String,
    r#type: ReapackType,
    /// (Added in v1.1) Display name of the package (the value of `name` is used if omitted)
    desc: Option<String>,
    #[serde(default)]
    #[serde(rename = "$value")]
    entries: Vec<ReapackEntry>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ReapackEntry {
    /// Many
    Version(Version),
    /// Zero or one
    Metadata(Metadata),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Version {
    name: VersionName,
    /// Author(s) of the release (no specific format)
    author: Option<String>,
    /// Release datetime in ISO 8601 format (UTC timezone)
    #[serde(default)]
    #[serde(with = "time::serde::iso8601::option")]
    time: Option<OffsetDateTime>,
    #[serde(default)]
    #[serde(rename = "$value")]
    entries: Vec<VersionEntry>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum VersionEntry {
    /// Zero or one
    Changelog(Changelog),
    /// Many
    Source(Source),
}

/// Sets the plain text changelog of the [`version`](Version) containing this element.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Changelog {
    #[serde(rename = "$value")]
    content: String,
}

/// This element represent a single file in a version.
///
/// The content of this node must be the downloadˆ URL.
///
/// Use a version-specific URL and keep previous version available when possible.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Source {
    /// File name/path (relative to the category name). Defaults to the package name.
    file: Option<String>,
    platform: Option<Platform>,
    /// Overrides the [package type](Reapack).
    r#type: Option<ReapackType>,
    /// List of Action List sections.
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_sections")]
    main: Vec<Section>,
    /// [Multihash](https://multiformats.io/multihash/) checksum of the file in hexadecimal form
    /// (added in v1.2.2). Supports SHA-256 (`1220` prefix).
    hash: Option<String>,
    /// Download URL.
    #[serde(rename = "$value")]
    content: String,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
enum ReapackType {
    Known(KnownReapackType),
    Unknown(String),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum KnownReapackType {
    Script,
    Effect,
    Extension,
    Data,
    Theme,
    LangPack,
    WebInterface,
    #[serde(rename = "projectpl")]
    ProjectTemplate,
    #[serde(rename = "tracktpl")]
    TrackTemplate,
    MiiNoteNames,
    AutoItem,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
enum Platform {
    Known(KnownPlatform),
    Unknown(String),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum KnownPlatform {
    #[default]
    All,
    Darwin,
    Darwin32,
    Darwin64,
    DarwinArm64,
    Linux,
    Linux32,
    Linux64,
    LinuxArmv7l,
    LinuxAarch64,
    Windows,
    Win32,
    Win64,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
enum Section {
    Known(KnownSection),
    Unknown(String),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum KnownSection {
    #[serde(rename = "main")]
    Main,
    #[serde(rename = "midi_editor")]
    MidiEditor,
    #[serde(rename = "midi_inlineeditor")]
    MidiInlineEditor,
    #[serde(rename = "midi_eventlisteditor")]
    MidiEventListEditor,
    #[serde(rename = "mediaexplorer")]
    MediaExplorer,
    /// For compatibility with v1.0, a special value `true` is also supported. This uses the
    /// category name to determine the section.
    #[serde(rename = "true")]
    True,
}

/// Fills the about dialog of the repository or of a package.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Metadata {
    #[serde(default)]
    #[serde(rename = "$value")]
    entries: Vec<MetadataEntry>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MetadataEntry {
    /// Zero or one
    Description(String),
    /// Zero or one
    Link(Link),
}

/// If the `href` argument is present, the content of the element becomes the display name of the
/// link.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Link {
    /// Path divided with slashes
    rel: Option<Rel>,
    /// If present, the content of the element becomes the display name of the link.
    ///
    /// If omitted, the content of the element becomes the URL.
    ///
    /// Must start with `http://` or `https://`.
    href: Option<String>,
    #[serde(rename = "$value")]
    content: String,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
enum Rel {
    Known(KnownRel),
    Unknown(String),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum KnownRel {
    #[default]
    Website,
    Donation,
    Screenshot,
}

fn deserialize_sections<'de, D>(deserializer: D) -> Result<Vec<Section>, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    text.split(' ')
        .map(|item| Section::deserialize(item.into_deserializer()))
        .collect()
}

impl Index {
    pub fn find_category(&self, name: &str) -> Option<&Category> {
        self.categories().find(|cat| cat.name == name)
    }

    pub fn categories(&self) -> impl Iterator<Item = &Category> {
        self.entries.iter().filter_map(|entry| match entry {
            IndexEntry::Category(c) => Some(c),
            IndexEntry::Metadata(_) => None,
        })
    }

    pub fn metadata(&self) -> Option<&Metadata> {
        self.entries.iter().find_map(|entry| match entry {
            IndexEntry::Category(_) => None,
            IndexEntry::Metadata(m) => Some(m),
        })
    }
}

impl Category {
    pub fn find_package(&self, name: &str) -> Option<&Reapack> {
        self.reapacks.iter().find(|p| p.name == name)
    }
}

impl Reapack {
    pub fn latest_version(&self) -> Option<&Version> {
        self.versions().max_by_key(|e| &e.name)
    }

    pub fn latest_stable_version(&self) -> Option<&Version> {
        self.stable_versions().max_by_key(|e| &e.name)
    }

    pub fn stable_versions(&self) -> impl Iterator<Item = &Version> {
        self.versions().filter(|v| v.name.is_stable())
    }

    pub fn versions(&self) -> impl Iterator<Item = &Version> {
        self.entries.iter().filter_map(|entry| match entry {
            ReapackEntry::Version(v) => Some(v),
            ReapackEntry::Metadata(_) => None,
        })
    }

    pub fn metadata(&self) -> Option<&Metadata> {
        self.entries.iter().find_map(|entry| match entry {
            ReapackEntry::Version(_) => None,
            ReapackEntry::Metadata(m) => Some(m),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_example() {
        let src = include_str!("test/Simple Example.xml");
        let index: Index = serde_xml_rs::from_str(src).unwrap();
    }

    #[test]
    fn advanced() {
        let src = include_str!("test/Helgoboss Projects.xml");
        let index: Index = serde_xml_rs::from_str(src).unwrap();
        let extensions = index.find_category("Extensions").unwrap();
        let realearn = extensions.find_package("ReaLearn-x64").unwrap();
        let stable_versions: Vec<_> = realearn.versions().map(|v| v.name.to_string()).collect();
        assert_eq!(realearn.name, "ReaLearn-x64");
        assert_eq!(
            realearn.desc,
            Some(
                "ReaLearn: Sophisticated MIDI/OSC-learn tool for controlling REAPER with feedback"
                    .to_string()
            )
        );
        assert_eq!(
            realearn.r#type,
            ReapackType::Known(KnownReapackType::Extension)
        );
        let latest_version = realearn.latest_version().unwrap();
        assert_eq!(latest_version.name, "2.16.0-pre.13".parse().unwrap());
        let latest_stable_version = realearn.latest_stable_version().unwrap();
        assert_eq!(latest_stable_version.name, "2.15.0".parse().unwrap());
    }
}
