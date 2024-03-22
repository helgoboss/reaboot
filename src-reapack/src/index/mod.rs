use crate::model::PackageType::{
    AutomationItem, Data, Effect, Extension, LangPack, MidiNoteNames, ProjectTemplate, Script,
    Theme, TrackTemplate, WebInterface,
};
use crate::model::{PackageType, Platform, Section, VersionName};
use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;
use time::OffsetDateTime;
use url::Url;

/// This is the root element of any ReaPack index file.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Index {
    /// Must be "1".
    pub version: String,
    /// Display name of the repository (must contain filename-friendly characters only).
    ///
    /// Required for import.
    pub name: Option<String>,
    #[serde(default)]
    #[serde(rename = "$value")]
    entries: Vec<IndexEntry>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndexEntry {
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
    pub name: String,
    #[serde(default)]
    #[serde(rename = "$value")]
    pub packages: Vec<Package>,
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
pub struct Package {
    /// Default filename of the package
    pub name: String,
    #[serde(rename = "type")]
    pub typ: IndexPackageType,
    /// (Added in v1.1) Display name of the package (the value of `name` is used if omitted)
    pub desc: Option<String>,
    #[serde(default)]
    #[serde(rename = "$value")]
    entries: Vec<PackageEntry>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageEntry {
    /// Many
    Version(Version),
    /// Zero or one
    Metadata(Metadata),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Version {
    pub name: VersionName,
    /// Author(s) of the release (no specific format)
    pub author: Option<String>,
    /// Release datetime in ISO 8601 format (UTC timezone)
    #[serde(default)]
    #[serde(with = "time::serde::iso8601::option")]
    pub time: Option<OffsetDateTime>,
    #[serde(default)]
    #[serde(rename = "$value")]
    entries: Vec<VersionEntry>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionEntry {
    /// Zero or one
    Changelog(Changelog),
    /// Many
    Source(Source),
}

/// Sets the plain text changelog of the [`version`](Version) containing this element.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Changelog {
    #[serde(rename = "$value")]
    pub content: String,
}

/// This element represent a single file in a version.
///
/// The content of this node must be the downloadˆ URL.
///
/// Use a version-specific URL and keep previous version available when possible.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct Source {
    /// File name/path (relative to the category name). Defaults to the package name.
    pub file: Option<String>,
    #[serde(default)]
    pub platform: IndexPlatform,
    /// Overrides the [package type](Package).
    pub typ: Option<IndexPackageType>,
    /// List of Action List sections.
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_sections")]
    pub main: Vec<IndexSection>,
    /// [Multihash](https://multiformats.io/multihash/) checksum of the file in hexadecimal form
    /// (added in v1.2.2). Supports SHA-256 (`1220` prefix).
    pub hash: Option<String>,
    /// Download URL.
    #[serde(rename = "$value")]
    pub content: Url,
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
pub enum MetadataEntry {
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
    pub rel: Rel,
    /// If present, the content of the element becomes the display name of the link.
    ///
    /// If omitted, the content of the element becomes the URL.
    ///
    /// Must start with `http://` or `https://`.
    pub href: Option<String>,
    #[serde(rename = "$value")]
    pub content: String,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
pub enum Rel {
    Known(KnownRel),
    Unknown(String),
}

impl Default for Rel {
    fn default() -> Self {
        Self::Known(KnownRel::default())
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnownRel {
    #[default]
    Website,
    Donation,
    Screenshot,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
pub enum IndexPackageType {
    Known(PackageType),
    Unknown(String),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
pub enum IndexSection {
    Known(Section),
    Unknown(String),
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(untagged)]
pub enum IndexPlatform {
    Known(Platform),
    Unknown(String),
}

impl Default for IndexPlatform {
    fn default() -> Self {
        Self::Known(Platform::default())
    }
}

fn deserialize_sections<'de, D>(deserializer: D) -> Result<Vec<IndexSection>, D::Error>
where
    D: Deserializer<'de>,
{
    let text = String::deserialize(deserializer)?;
    text.split(' ')
        .map(|item| IndexSection::deserialize(item.into_deserializer()))
        .collect()
}

impl Index {
    pub fn parse(reader: impl Read) -> Result<Self, serde_xml_rs::Error> {
        serde_xml_rs::from_reader(reader)
    }

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
    pub fn find_package(&self, name: &str) -> Option<&Package> {
        self.packages.iter().find(|p| p.name == name)
    }
}

impl Package {
    pub fn find_version(&self, name: &VersionName) -> Option<&Version> {
        self.all_versions().find(|v| &v.name == name)
    }

    pub fn latest_version_including_pre_releases(&self) -> Option<&Version> {
        self.all_versions().max_by_key(|e| &e.name)
    }

    pub fn latest_stable_version(&self) -> Option<&Version> {
        self.stable_versions().max_by_key(|e| &e.name)
    }

    pub fn stable_versions(&self) -> impl Iterator<Item = &Version> {
        self.all_versions().filter(|v| v.name.is_stable())
    }

    pub fn all_versions(&self) -> impl Iterator<Item = &Version> {
        self.entries.iter().filter_map(|entry| match entry {
            PackageEntry::Version(v) => Some(v),
            PackageEntry::Metadata(_) => None,
        })
    }

    pub fn metadata(&self) -> Option<&Metadata> {
        self.entries.iter().find_map(|entry| match entry {
            PackageEntry::Version(_) => None,
            PackageEntry::Metadata(m) => Some(m),
        })
    }
}

impl Version {
    pub fn sources(&self) -> impl Iterator<Item = &Source> {
        self.entries.iter().filter_map(|entry| match entry {
            VersionEntry::Changelog(_) => None,
            VersionEntry::Source(s) => Some(s),
        })
    }
}

impl Source {
    /// Returns the destination file relative to the REAPER resource directory.
    ///
    /// Returns `None` if the package type is unknown.
    pub fn determine_destination_file(
        &self,
        index_name: &str,
        category: &str,
        package: &Package,
    ) -> Option<String> {
        use PackageType::*;
        let typ = self.typ.as_ref().unwrap_or(&package.typ);
        let IndexPackageType::Known(typ) = typ else {
            return None;
        };
        let file = self.file.as_ref().unwrap_or(&package.name);
        let dest_file = match typ {
            Script => format!("Scripts/{index_name}/{category}/{file}"),
            Extension => format!("UserPlugins/{file}"),
            Effect => format!("Effects/{index_name}/{category}/{file}"),
            Data => format!("Data/{file}"),
            Theme => format!("ColorThemes/{file}"),
            LangPack => format!("LangPack/{file}"),
            WebInterface => format!("reaper_www_root/{file}"),
            ProjectTemplate => format!("ProjectTemplates/{file}"),
            TrackTemplate => format!("TrackTemplates/{file}"),
            MidiNoteNames => format!("MIDINoteNames/{file}"),
            AutomationItem => format!("AutomationItems/{index_name}/{category}/{file}"),
        };
        Some(dest_file)
    }
}

#[derive(Error, Debug)]
pub enum ParseVersionError {
    #[error("invalid version name '{0}'")]
    InvalidVersionName(String),
    #[error("version segment overflow in '{0}'")]
    VersionSegmentOverflow(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PackageType;

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
        let stable_versions: Vec<_> = realearn
            .all_versions()
            .map(|v| v.name.to_string())
            .collect();
        assert_eq!(realearn.name, "ReaLearn-x64");
        assert_eq!(
            realearn.desc,
            Some(
                "ReaLearn: Sophisticated MIDI/OSC-learn tool for controlling REAPER with feedback"
                    .to_string()
            )
        );
        assert_eq!(
            realearn.typ,
            IndexPackageType::Known(PackageType::Extension)
        );
        let latest_version = realearn.latest_version_including_pre_releases().unwrap();
        assert_eq!(latest_version.name, "2.16.0-pre.13".parse().unwrap());
        let latest_stable_version = realearn.latest_stable_version().unwrap();
        assert_eq!(latest_stable_version.name, "2.15.0".parse().unwrap());
    }
}
