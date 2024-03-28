use crate::installer::DownloadedIndex;
use crate::reaper_target::ReaperTarget;
use anyhow::{Context, Error};
use reaboot_reapack::index::{
    Category, Index, IndexPackageType, IndexPlatform, Package, Source, Version,
};
use reaboot_reapack::model::{
    InstalledPackage, LightPackageId, LightVersionId, PackageType, PackageUrl, VersionName,
    VersionRef,
};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
use thiserror::Error;
use url::Url;

pub struct PackageDownloadPlan<'a> {
    /// Packages that were mentioned in the recipes but are not in the repository index.
    pub package_descriptors_with_failures: Vec<PackageDescFailure<'a>>,
    /// Packages for which it's unclear which version to install.
    pub version_conflicts: Vec<VersionConflict<'a>>,
    /// Package versions that have an unsupported package type override or don't have any files to
    /// install and therefore can be considered as not supported on this operating system.
    pub incompatible_versions: Vec<QualifiedVersion<'a>>,
    /// Package files that clash with other package files of packages mentioned in the recipes,
    /// because they would be installed to exactly the same destination (directory and name).
    pub recipe_file_conflicts: Vec<RecipeFileConflict<'a>>,
    /// Package files that clash with other package files of already installed packages,
    /// because they would be installed to exactly the same destination (directory and name).
    pub files_conflicting_with_already_installed_files: Vec<QualifiedSource<'a>>,
}

#[derive(Copy, Clone, Error, Debug)]
pub enum PackageDescError {
    #[error("Repository is unavailable")]
    RepositoryIndexUnavailable,
    #[error("Desired package category was not found")]
    PackageCategoryNotFound,
    #[error("Desired package was not found")]
    PackageNotFound,
    #[error("Package has unknown type")]
    PackageHasUnknownType,
    #[error("Desired package has no stable version yet")]
    PackageHasNoStableVersion,
    #[error("Desired package has no versions at all")]
    PackageHasNoVersionsAtAll,
    #[error("Desired package version was not found")]
    PackageVersionNotFound,
}

impl<'a> PackageDownloadPlan<'a> {
    /// Returns all remaining files to be installed, with files belonging to incomplete packages
    /// removed.
    pub fn make(
        package_urls: &'a [PackageUrl],
        indexes: &'a HashMap<Url, DownloadedIndex>,
        installed_packages_to_keep: &[InstalledPackage],
        reaper_target: ReaperTarget,
    ) -> (Vec<QualifiedSource<'a>>, Self) {
        let deduplicated_package_urls = HashSet::from_iter(package_urls);
        let (versions, package_descriptors_with_failures) =
            resolve_and_deduplicate_versions(deduplicated_package_urls, indexes);
        let (versions, version_conflicts) = weed_out_packages_with_version_conflicts(versions);
        let (sources, incompatible_versions) =
            resolve_package_sources_weeding_out_platform_incompatible_versions(
                versions,
                reaper_target,
            );
        let (sources, recipe_file_conflicts) = weed_out_conflicting_files_within_recipes(sources);
        let (mut files, files_conflicting_with_already_installed_files) =
            weed_out_conflicting_files_with_already_installed_packages(
                sources,
                installed_packages_to_keep,
            );
        remove_incomplete_versions(&mut files, &recipe_file_conflicts);
        let plan = Self {
            package_descriptors_with_failures,
            version_conflicts,
            incompatible_versions,
            recipe_file_conflicts,
            files_conflicting_with_already_installed_files,
        };
        (files, plan)
    }
}

pub struct QualifiedSource<'a> {
    pub version: QualifiedVersion<'a>,
    pub source: &'a Source,
    /// If the source has a package type override, it must be a supported type, otherwise we weed
    /// it out early in the process. We save the supported type here.
    pub typ: Option<PackageType>,
    pub relative_path: String,
}

pub struct VersionConflict<'a> {
    pub package_id: LightPackageId<'a>,
    pub conflicting_versions: Vec<QualifiedVersion<'a>>,
}

pub struct RecipeFileConflict<'a> {
    pub relative_path: String,
    pub conflicting_files: Vec<QualifiedSource<'a>>,
}

pub struct PackageDescFailure<'a> {
    pub remote: Option<&'a str>,
    pub package_url: &'a PackageUrl,
    pub error: PackageDescError,
}

#[derive(Copy, Clone)]
pub struct QualifiedPackage<'a> {
    pub index: &'a DownloadedIndex,
    pub category: &'a Category,
    pub package: &'a Package,
    /// Packages with unknown type are sorted out early, so we save the known type here.
    pub typ: PackageType,
}

impl<'a> QualifiedPackage<'a> {
    pub fn id(&self) -> LightPackageId<'a> {
        LightPackageId {
            remote: &self.index.name,
            category: &self.category.name,
            package: &self.package.name,
        }
    }
}

#[derive(Copy, Clone)]
pub struct QualifiedVersion<'a> {
    pub package: QualifiedPackage<'a>,
    pub version: &'a Version,
}

impl<'a> QualifiedVersion<'a> {
    pub fn id(&self) -> LightVersionId<'a> {
        LightVersionId {
            package_id: self.package.id(),
            version: &self.version.name,
        }
    }
}

impl<'a> QualifiedSource<'a> {
    pub fn package_id(&self) -> LightPackageId<'a> {
        self.version.id().package_id
    }
}

fn resolve_and_deduplicate_versions<'a>(
    package_urls: HashSet<&'a PackageUrl>,
    indexes: &'a HashMap<Url, DownloadedIndex>,
) -> (Vec<QualifiedVersion<'a>>, Vec<PackageDescFailure<'a>>) {
    let mut failures = vec![];
    let qualified_versions: HashMap<_, _> = package_urls
        .into_iter()
        .filter_map(|desc| {
            let Some(index) = indexes.get(desc.repository_url()) else {
                failures.push(PackageDescFailure {
                    remote: None,
                    package_url: desc,
                    error: PackageDescError::RepositoryIndexUnavailable,
                });
                return None;
            };
            match lookup_package_version_in_index(&desc, index) {
                Ok(v) => Some((v.id().package_id, v)),
                Err(error) => {
                    failures.push(PackageDescFailure {
                        remote: Some(&index.name),
                        package_url: desc,
                        error,
                    });
                    None
                }
            }
        })
        .collect();
    (qualified_versions.into_values().collect(), failures)
}

fn lookup_package_version_in_index<'i>(
    package_url: &PackageUrl,
    index: &'i DownloadedIndex,
) -> Result<QualifiedVersion<'i>, PackageDescError> {
    let category = index
        .index
        .find_category(&package_url.category())
        .ok_or(PackageDescError::PackageCategoryNotFound)?;
    let package = category
        .find_package(&package_url.package_name())
        .ok_or(PackageDescError::PackageNotFound)?;
    let IndexPackageType::Known(typ) = &package.typ else {
        return Err(PackageDescError::PackageHasUnknownType);
    };
    let qualified_package = QualifiedPackage {
        index,
        category,
        package,
        typ: *typ,
    };
    let version = match &package_url.version_ref() {
        VersionRef::Latest => package
            .latest_stable_version()
            .ok_or(PackageDescError::PackageHasNoStableVersion)?,
        VersionRef::LatestPre => package
            .latest_version_including_pre_releases()
            .ok_or(PackageDescError::PackageHasNoVersionsAtAll)?,
        VersionRef::Specific(v) => package
            .find_version(&v)
            .ok_or(PackageDescError::PackageVersionNotFound)?,
    };
    let qualified_version = QualifiedVersion {
        package: qualified_package,
        version,
    };
    Ok(qualified_version)
}

fn weed_out_packages_with_version_conflicts(
    versions: Vec<QualifiedVersion>,
) -> (Vec<QualifiedVersion>, Vec<VersionConflict>) {
    weed_out_conflicts(
        versions,
        |v| v.id().package_id,
        |package_id, conflicting_versions| VersionConflict {
            package_id,
            conflicting_versions,
        },
    )
}

fn resolve_package_sources_weeding_out_platform_incompatible_versions(
    versions: Vec<QualifiedVersion>,
    reaper_target: ReaperTarget,
) -> (Vec<QualifiedSource>, Vec<QualifiedVersion>) {
    let mut incompatible_versions = vec![];
    let sources: Vec<_> = versions
        .into_iter()
        .flat_map(|v| {
            let sources: Vec<_> = get_platform_compatible_sources(v.version, reaper_target)
                .filter_map(|source| {
                    let typ = match source.typ.as_ref() {
                        None => {
                            // No package type override. Cool.
                            None
                        }
                        Some(IndexPackageType::Known(t)) => {
                            // Override with known package type. Cool.
                            Some(*t)
                        }
                        Some(IndexPackageType::Unknown(_)) => {
                            // Override with unknown package type. Not cool.
                            return None;
                        }
                    };
                    let resolved_typ = typ.unwrap_or(v.package.typ);
                    let relative_path = source.determine_destination_file(
                        &v.package.index.name,
                        &v.package.category.name,
                        &v.package.package,
                        resolved_typ,
                    );
                    let qualified_source = QualifiedSource {
                        version: v,
                        source,
                        typ,
                        relative_path,
                    };
                    Some(qualified_source)
                })
                .collect();
            if sources.is_empty() {
                // Versions with no sources that match the current platform are silently discarded
                // by ReaPack and therefore reported as incompatible by ReaBoot
                incompatible_versions.push(v);
            }
            sources.into_iter()
        })
        .collect();
    (sources, incompatible_versions)
}

fn get_platform_compatible_sources(
    version: &Version,
    reaper_target: ReaperTarget,
) -> impl Iterator<Item = &Source> {
    version.sources().filter(move |source| {
        let IndexPlatform::Known(platform) = &source.platform else {
            // Unknown platforms are never compatible
            return false;
        };
        reaper_target.is_compatible_with_reapack_platform(*platform)
    })
}

fn weed_out_conflicting_files_within_recipes(
    files: Vec<QualifiedSource>,
) -> (Vec<QualifiedSource>, Vec<RecipeFileConflict>) {
    weed_out_conflicts(
        files,
        |f| f.relative_path.to_string(),
        |relative_path, conflicting_files| RecipeFileConflict {
            relative_path,
            conflicting_files,
        },
    )
}

fn weed_out_conflicting_files_with_already_installed_packages<'a>(
    files: Vec<QualifiedSource<'a>>,
    already_installed_non_replaced_packages: &[InstalledPackage],
) -> (Vec<QualifiedSource<'a>>, Vec<QualifiedSource<'a>>) {
    let already_installed_paths: HashSet<_> = already_installed_non_replaced_packages
        .iter()
        .flat_map(|p| p.files.iter())
        .map(|f| &f.path)
        .collect();
    let (files, conflicting_files): (Vec<_>, Vec<_>) = files
        .into_iter()
        .partition(|f| !already_installed_paths.contains(&f.relative_path));
    (files, conflicting_files)
}

fn weed_out_conflicts<T, Key, Conflict>(
    items: Vec<T>,
    get_key: impl Fn(&T) -> Key,
    build_conflict: impl Fn(Key, Vec<T>) -> Conflict,
) -> (Vec<T>, Vec<Conflict>)
where
    Key: Hash + Eq,
{
    // Group by key so that we can easily detect conflicts
    let mut items_by_key: HashMap<Key, Vec<T>> = HashMap::new();
    for item in items {
        items_by_key.entry(get_key(&item)).or_default().push(item);
    }
    // Put items with conflicts aside
    let mut conflicts = vec![];
    let cool_items = items_by_key
        .into_iter()
        .filter_map(|(key, items)| {
            if items.len() == 1 {
                Some(items.into_iter().next()?)
            } else {
                // We have conflicting items
                let conflict = build_conflict(key, items);
                conflicts.push(conflict);
                None
            }
        })
        .collect();
    (cool_items, conflicts)
}

fn remove_incomplete_versions(
    sources: &mut Vec<QualifiedSource>,
    file_conflicts: &Vec<RecipeFileConflict>,
) {
    let incomplete_versions = identify_incomplete_versions(&file_conflicts);
    sources.retain(|source| !incomplete_versions.contains(&source.version.id()));
}

fn identify_incomplete_versions<'a>(
    file_conflicts: &'a Vec<RecipeFileConflict>,
) -> HashSet<LightVersionId<'a>> {
    file_conflicts
        .iter()
        .flat_map(|c| c.conflicting_files.iter())
        .map(|s| s.version.id())
        .collect()
}
