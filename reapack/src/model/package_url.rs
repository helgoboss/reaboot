use crate::model::VersionRef;

use serde::Serialize;
use std::str::FromStr;
use thiserror::Error;
use ts_rs::TS;

use url::{form_urlencoded, Url};

/// The Package URL is a URL that points to a specific repository index and uniquely identifies a
/// specific version of a package within that index.
///
/// # Examples
///
/// - `https://github.com/ReaTeam/ReaScripts/raw/master/index.xml#p=Various/rodilab_Color%20palette.lua&v=1.2.3-pre`
/// - `https://github.com/helgoboss/reaper-packages/raw/master/index.xml#p=Extensions/Helgobox-x64`
///
/// # Structure
///
/// The URL follows this schema:
///
/// - **PACKAGE_URL =** `{REPOSITORY_INDEX_URL}#{PACKAGE_VERSION_REF}`
/// - **PACKAGE_VERSION_REF =** `p={PACKAGE_PATH}&v={VERSION_REF}`
/// - **PACKAGE_PATH =** `{CATEGORY}/{PACKAGE_NAME}`
/// - **VERSION_REF =** `{VERSION_NAME}` or `latest` or `latest-pre`
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, TS)]
#[ts(export)]
pub struct PackageUrl {
    /// Repository index URL.
    ///
    /// - Must not contain any `#` character.
    repository_url: Url,
    /// Package version reference.
    ///
    /// - This part is technically the [fragment ID](https://www.w3.org/Addressing/URL/4_2_Fragments.html)
    ///   of the URL.
    /// - See [`PackageVersionRef`]
    package_version_ref: PackageVersionRef,
}

/// A package version reference uniquely identifies a specific version of a package within
/// a repository index.
///
/// # Examples
///
/// - `p=Various/rodilab_Color%20palette.lua&v=1.2.3-pre`
///
/// # Structure
///
/// A version reference follows this schema:
///
/// - **PACKAGE_VERSION_REF =** `p={PACKAGE_PATH}&v={VERSION_REF}`
/// - **PACKAGE_PATH =** `{CATEGORY}/{PACKAGE_NAME}`
/// - **VERSION_REF =** `{VERSION_NAME}` or `latest` or `latest-pre`
///
/// - It's built like a typical URL query string, so it's made of `=`-delimited key-value pairs
///   that are separated by `&` characters.
/// - Keys and values are encoded with the `application/x-www-form-urlencoded` encoding.
/// - The order of the key-value pairs is irrelevant.
/// - `p` is required.
/// - `v` is optional and defaults to `latest`.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, TS)]
#[ts(export)]
pub struct PackageVersionRef {
    package_path: PackagePath,
    version_ref: VersionRef,
}

/// A package path uniquely identifies a package within a repository index without referring to
/// a specific version of that package.
///
/// # Examples
///
/// # Structure
///
/// A package path follows this schema:
///
/// - **PACKAGE_PATH =** `{CATEGORY}/{PACKAGE_NAME}`
/// - **VERSION_REF =** `{VERSION_NAME}` or `latest` or `latest-pre`
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, TS)]
#[ts(export)]
pub struct PackagePath {
    /// May contain `/` characters.
    category: String,
    /// The package name is a file name, so it must not contain any `/` characters.
    package_name: String,
}

impl PackageUrl {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, ParsePackageUrlError> {
        let mut url = Url::parse(input.as_ref()).map_err(ParsePackageUrlError::InvalidUrl)?;
        let fragment = url
            .fragment()
            .ok_or(ParsePackageUrlError::MissingFragmentIdentifier)?;
        let package_version_ref = fragment.parse()?;
        url.set_fragment(None);
        let package_url = Self {
            repository_url: url,
            package_version_ref,
        };
        Ok(package_url)
    }

    pub fn repository_url(&self) -> &Url {
        &self.repository_url
    }

    pub fn package_version_ref(&self) -> &PackageVersionRef {
        &self.package_version_ref
    }

    pub fn package_path(&self) -> &PackagePath {
        &self.package_version_ref().package_path
    }

    pub fn version_ref(&self) -> &VersionRef {
        &self.package_version_ref().version_ref
    }

    pub fn category(&self) -> &str {
        &self.package_path().category
    }

    pub fn package_name(&self) -> &str {
        &self.package_path().package_name
    }
}

impl PackageVersionRef {
    pub fn package_path(&self) -> &PackagePath {
        &self.package_path
    }

    pub fn version_ref(&self) -> &VersionRef {
        &self.version_ref
    }

    pub fn category(&self) -> &str {
        &self.package_path().category
    }

    pub fn package_name(&self) -> &str {
        &self.package_path().package_name
    }
}

impl PackagePath {
    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }
}

impl FromStr for PackageVersionRef {
    type Err = ParsePackageUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut package_path: Option<PackagePath> = None;
        let mut version_ref = VersionRef::Latest;
        for (key, value) in form_urlencoded::parse(s.as_bytes()) {
            match key.as_ref() {
                "p" => {
                    package_path = Some(value.parse()?);
                }
                "v" => {
                    version_ref = value.parse()?;
                }
                _ => {
                    // Ignore unknown attributes
                }
            }
        }
        let package_version_ref = Self {
            package_path: package_path.ok_or(ParsePackageUrlError::MissingPackagePath)?,
            version_ref,
        };
        Ok(package_version_ref)
    }
}

impl FromStr for PackagePath {
    type Err = ParsePackageUrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (category, package_name) = s
            .rsplit_once('/')
            .ok_or(ParsePackageUrlError::InvalidPackagePath)?;
        let package_path = Self {
            category: category.to_string(),
            package_name: package_name.to_string(),
        };
        Ok(package_path)
    }
}

#[derive(Error, Debug)]
pub enum ParsePackageUrlError {
    #[error("Invalid URL")]
    InvalidUrl(url::ParseError),
    #[error("Fragment identifier is missing")]
    MissingFragmentIdentifier,
    #[error("Package path is missing")]
    MissingPackagePath,
    #[error("Package path is invalid")]
    InvalidPackagePath,
    #[error("Version reference is invalid")]
    InvalidVersionRef,
}
