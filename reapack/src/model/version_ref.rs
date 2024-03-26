use crate::model::VersionName;
use serde::Deserialize;
use ts_rs::TS;

/// Reference to a specific package version.
///
/// # Structure
///
/// - **VERSION_REF =** `{VERSION_NAME}` or `latest` or `latest-pre`
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Deserialize, TS)]
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
