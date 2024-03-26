use serde::Deserialize;
use std::cmp;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::OnceLock;
use thiserror::Error;
use tinyvec::{tiny_vec, TinyVec};
use ts_rs::TS;

/// ReaPack treats versions names as segments of whole numbers and letters optionally separated by
/// one or more non-alphanumeric character (such as dots).
///
/// - A valid version must start with a digit.
/// - Individual number segments must fit in an unsigned 16-bit integer (0 to 65535).
/// - A version is treated as a pre-release if it contains one or more letters.
/// - Versions are handled as if they end by an infinity of 0 segments (1 = 1.0 = 1.0.0 etc).
#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct VersionName {
    string: String,
    segments: TinyVec<[Segment; 3]>,
    stable: bool,
}

// With this, we instruct ts-rs to treat it just like a string when generating TypeScript.
// I followed the example of "Url" (its impl_primitives! macro generates the same).
impl TS for VersionName {
    type WithoutGenerics = Self;

    fn name() -> String {
        "string".to_string()
    }
    fn inline() -> String {
        <Self as TS>::name()
    }
    fn inline_flattened() -> String {
        panic!("{} cannot be flattened", <Self as TS>::name())
    }
    fn decl() -> String {
        panic!("{} cannot be declared", <Self as TS>::name())
    }
    fn decl_concrete() -> String {
        panic!("{} cannot be declared", <Self as TS>::name())
    }
}

impl Hash for VersionName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.string.hash(state)
    }
}

impl VersionName {
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_stable(&self) -> bool {
        self.stable
    }

    /// Returns the major version number.
    pub fn major(&self) -> Numeric {
        let first_segment = self
            .segments
            .first()
            .expect("versions without any segment shouldn't exist, this is a bug in VersionName");
        match first_segment {
            Segment::Numeric(i) => *i,
            Segment::String(_) => panic!("versions where the first segment is a string should not exist, this is a bug in VersionName")
        }
    }
}

impl AsRef<str> for VersionName {
    fn as_ref(&self) -> &str {
        &self.string
    }
}

impl Display for VersionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.string.fmt(f)
    }
}

impl Eq for VersionName {}

impl PartialEq for VersionName {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}

impl PartialOrd for VersionName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionName {
    fn cmp(&self, other: &Self) -> Ordering {
        use Segment::*;
        let biggest = cmp::max(self.len(), other.len());
        let default = Numeric(0);
        for i in 0..biggest {
            let lseg = self.segments.get(i).unwrap_or(&default);
            let rseg = other.segments.get(i).unwrap_or(&default);
            if let (Numeric(l), Numeric(r)) = (lseg, rseg) {
                let ord = l.cmp(r);
                if !ord.is_eq() {
                    return ord;
                }
            }
            if let (String(l), String(r)) = (lseg, rseg) {
                let ord = l.cmp(r);
                if !ord.is_eq() {
                    return ord;
                }
            }
            if let (Numeric(_), String(_)) = (lseg, rseg) {
                return Ordering::Greater;
            }
            if let (String(_), Numeric(_)) = (lseg, rseg) {
                return Ordering::Less;
            }
        }
        Ordering::Equal
    }
}

#[derive(Clone, Debug)]
enum Segment {
    Numeric(Numeric),
    String(String),
}

impl Default for Segment {
    fn default() -> Self {
        Self::Numeric(0)
    }
}

type Numeric = u16;

impl FromStr for VersionName {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut segments = tiny_vec!([Segment; 3]);
        let mut letters = 0usize;
        let string = s.to_string();
        for m in search_segments_regex().find_iter(s) {
            let is_alphabetic = m
                .as_str()
                .chars()
                .next()
                .is_some_and(|ch| ch.is_alphabetic());
            if is_alphabetic {
                if segments.is_empty() {
                    // Got leading letters
                    return Err(ParseVersionError::InvalidVersionName(string));
                }
                segments.push(Segment::String(m.as_str().to_string()));
                letters += 1;
            } else {
                let Ok(number) = m.as_str().parse() else {
                    return Err(ParseVersionError::VersionSegmentOverflow(string));
                };
                segments.push(Segment::Numeric(number));
            }
        }
        if segments.is_empty() {
            return Err(ParseVersionError::InvalidVersionName(string));
        }
        let version = Self {
            string,
            segments,
            stable: letters == 0,
        };
        Ok(version)
    }
}

impl TryFrom<String> for VersionName {
    type Error = ParseVersionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

#[derive(Error, Debug)]
pub enum ParseVersionError {
    #[error("invalid version name '{0}'")]
    InvalidVersionName(String),
    #[error("version segment overflow in '{0}'")]
    VersionSegmentOverflow(String),
}

fn search_segments_regex() -> &'static regex::Regex {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"\d+|[a-zA-Z]+").unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare() {
        assert!(
            VersionName::from_str("2.16.0-pre.7").unwrap()
                < VersionName::from_str("2.16.0-pre.10").unwrap()
        );
    }
}
