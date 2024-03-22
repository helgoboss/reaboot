use serde::Deserialize;
use std::borrow::Cow;

/// Package type.
///
/// The `#[serde(rename = "...")]` attributes must not change in order to stay compatible with
/// ReaPack's [Index Format](https://github.com/cfillion/reapack/wiki/Index-Format).
///
/// The numbers must not change in order to stay compatible with ReaPack's database schema.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum PackageType {
    #[serde(rename = "script")]
    Script = 1,
    #[serde(rename = "extension")]
    Extension = 2,
    #[serde(rename = "effect")]
    Effect = 3,
    #[serde(rename = "data")]
    Data = 4,
    #[serde(rename = "theme")]
    Theme = 5,
    #[serde(rename = "langpack")]
    LangPack = 6,
    #[serde(rename = "webinterface")]
    WebInterface = 7,
    #[serde(rename = "projecttpl")]
    ProjectTemplate = 8,
    #[serde(rename = "tracktpl")]
    TrackTemplate = 9,
    #[serde(rename = "midinotenames")]
    MidiNoteNames = 10,
    #[serde(rename = "autoitem")]
    AutomationItem = 11,
}
