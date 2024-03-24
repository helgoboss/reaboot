use enumset::EnumSetType;
use num_enum::TryFromPrimitive;
use serde::Deserialize;

/// Section.
///
/// The `#[serde(rename = "...")]` attributes must not change in order to stay compatible with
/// ReaPack's [Index Format](https://github.com/cfillion/reapack/wiki/Index-Format).
///
/// The numbers must not change in order to stay compatible with ReaPack's database schema.
#[derive(Ord, PartialOrd, Hash, Debug, Deserialize, TryFromPrimitive, EnumSetType)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum Section {
    #[serde(rename = "main")]
    Main = 0,
    #[serde(rename = "midi_editor")]
    MidiEditor = 1,
    #[serde(rename = "midi_inlineeditor")]
    MidiInlineEditor = 2,
    #[serde(rename = "midi_eventlisteditor")]
    MidiEventListEditor = 3,
    #[serde(rename = "mediaexplorer")]
    MediaExplorer = 4,
}
