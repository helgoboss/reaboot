use serde::Deserialize;

/// Section.
///
/// The `#[serde(rename = "...")]` attributes must not change in order to stay compatible with
/// ReaPack's [Index Format](https://github.com/cfillion/reapack/wiki/Index-Format).
///
/// The numbers must not change in order to stay compatible with ReaPack's database schema.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum Section {
    #[serde(rename = "main")]
    Main = 1 << 0,
    #[serde(rename = "midi_editor")]
    MidiEditor = 1 << 1,
    #[serde(rename = "midi_inlineeditor")]
    MidiInlineEditor = 1 << 2,
    #[serde(rename = "midi_eventlisteditor")]
    MidiEventListEditor = 1 << 3,
    #[serde(rename = "mediaexplorer")]
    MediaExplorer = 1 << 4,
    /// For compatibility with v1.0, a special value `true` is also supported. This uses the
    /// category name to determine the section.
    #[serde(rename = "true")]
    ImplicitSection = -1,
}
