use camino::Utf8Path;
use enumset::EnumSetType;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Section.
///
/// The `#[serde(rename = "...")]` attributes must not change in order to stay compatible with
/// ReaPack's [Index Format](https://github.com/cfillion/reapack/wiki/Index-Format).
///
/// The numbers must not change in order to stay compatible with ReaPack's database schema.
#[derive(Ord, PartialOrd, Hash, Debug, Serialize, Deserialize, TryFromPrimitive, EnumSetType)]
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

impl Display for Section {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let section_string = serde_plain::to_string(self).map_err(|_| std::fmt::Error)?;
        section_string.fmt(f)
    }
}

impl Section {
    /// This is for compatibility with indexes made for ReaPack v1.0.
    ///
    /// https://github.com/cfillion/reapack/blob/1727aa34f3420a3996092d137fe69b9585ce809b/src/source.cpp#L44
    pub fn detect_from_category_legacy(category: &Utf8Path) -> Self {
        let first_component = category
            .components()
            .next()
            .map(|s| s.as_str())
            .unwrap_or("");
        let top_category = first_component.to_lowercase();
        if top_category == "midi editor" {
            Self::MidiEditor
        } else {
            Self::Main
        }
    }
}
