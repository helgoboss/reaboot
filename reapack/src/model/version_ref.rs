use crate::model::{ParsePackageUrlError, VersionName};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use ts_rs::TS;

/// Reference to a specific package version.
///
/// # Structure
///
/// - **VERSION_REF =** `{VERSION_NAME}` or `latest` or `latest-pre`
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum VersionRef {
    /// Refers to the latest available version of a package, excluding pre-releases.
    #[default]
    Latest,
    /// Refers to the latest available version of a package, including pre-releases.
    LatestPre,
    /// Refers to a specific version of a package.
    #[serde(untagged)]
    Specific(VersionName),
}

impl From<VersionName> for VersionRef {
    fn from(value: VersionName) -> Self {
        Self::Specific(value)
    }
}

impl Display for VersionRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = serde_plain::to_string(self).map_err(|_| std::fmt::Error)?;
        s.fmt(f)
    }
}

impl FromStr for VersionRef {
    type Err = ParsePackageUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_plain::from_str(s).map_err(|_| ParsePackageUrlError::InvalidVersionRef)
    }
}
