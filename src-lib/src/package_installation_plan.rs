use crate::api::{PackageDescriptor, Recipe, VersionDescriptor};
use crate::reaper_target::ReaperTarget;
use anyhow::{Context, Error};
use reaboot_reapack::index::{Category, Index, IndexPlatform, Package, Source, Version};
use reaboot_reapack::model::VersionName;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
use url::Url;

pub struct PackageInstallationPlan<'a> {
    pub package_descriptors_with_failures: Vec<PackageFailure<'a>>,
    pub version_conflicts: Vec<VersionConflict<'a>>,
    pub empty_versions: Vec<QualifiedVersion<'a>>,
    pub unsupported_sources: Vec<QualifiedSource<'a>>,
    pub file_conflicts: Vec<FileConflict<'a>>,
    pub final_files: Vec<QualifiedFile<'a>>,
}

impl<'a> PackageInstallationPlan<'a> {
    pub fn make(
        recipes: &'a [Recipe],
        indexes: impl IntoIterator<Item = QualifiedIndex<'a>>,
        reaper_target: ReaperTarget,
    ) -> Self {
        let index_by_url: HashMap<_, _> = indexes.into_iter().map(|i| (i.url, i)).collect();
        let package_descriptors = extract_and_deduplicate_package_descriptors(&recipes);
        let (versions, package_descriptors_with_failures) =
            resolve_and_deduplicate_versions(package_descriptors, |url| {
                index_by_url.get(url).copied()
            });
        let (versions, version_conflicts) = weed_out_packages_with_version_conflicts(versions);
        let (sources, empty_versions) =
            resolve_package_sources_weeding_out_platform_incompatible_versions(
                versions,
                reaper_target,
            );
        let (files, unsupported_sources) = resolve_files_weeding_out_unsupported_sources(sources);
        let (files, file_conflicts) = weed_out_files_with_conflicts(files);
        Self {
            package_descriptors_with_failures,
            version_conflicts,
            empty_versions,
            unsupported_sources,
            file_conflicts,
            final_files: files,
        }
    }
}

#[derive(Copy, Clone)]
pub struct QualifiedIndex<'a> {
    pub url: &'a Url,
    pub name: &'a str,
    pub index: &'a Index,
}

pub struct QualifiedFile<'a> {
    pub source: QualifiedSource<'a>,
    pub relative_path: String,
}

pub struct QualifiedSource<'a> {
    version: QualifiedVersion<'a>,
    pub source: &'a Source,
}

struct VersionConflict<'a> {
    package_id: QualifiedPackageId<'a>,
    conflicting_versions: Vec<QualifiedVersion<'a>>,
}

struct FileConflict<'a> {
    relative_path: String,
    conflicting_files: Vec<QualifiedFile<'a>>,
}

struct PackageFailure<'a> {
    desc: QualifiedPackageDescriptor<'a>,
    error: anyhow::Error,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct QualifiedPackageDescriptor<'a> {
    repository_url: &'a Url,
    package: &'a PackageDescriptor,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct QualifiedPackageId<'a> {
    index_name: &'a str,
    category: &'a str,
    package_name: &'a str,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct QualifiedVersionId<'a> {
    package_id: QualifiedPackageId<'a>,
    version_name: &'a VersionName,
}

#[derive(Copy, Clone)]
struct QualifiedVersion<'a> {
    index: QualifiedIndex<'a>,
    category: &'a Category,
    package: &'a Package,
    version: &'a Version,
}

impl<'a> QualifiedVersion<'a> {
    fn id(&self) -> QualifiedVersionId<'a> {
        QualifiedVersionId {
            package_id: QualifiedPackageId {
                index_name: self.index.name,
                category: &self.category.name,
                package_name: &self.package.name,
            },
            version_name: &self.version.name,
        }
    }
}

fn extract_and_deduplicate_package_descriptors(
    recipes: &[Recipe],
) -> HashSet<QualifiedPackageDescriptor> {
    recipes
        .iter()
        .flat_map(|recipe| {
            recipe.package_sets.iter().flat_map(|set| {
                set.packages
                    .iter()
                    .map(move |desc| QualifiedPackageDescriptor {
                        repository_url: &set.repository_url,
                        package: desc,
                    })
            })
        })
        .collect()
}

fn resolve_and_deduplicate_versions<'a>(
    package_descriptors: HashSet<QualifiedPackageDescriptor<'a>>,
    lookup_index: impl Fn(&Url) -> Option<QualifiedIndex<'a>> + Copy,
) -> (Vec<QualifiedVersion<'a>>, Vec<PackageFailure<'a>>) {
    let mut failures = vec![];
    let qualified_versions: HashMap<_, _> = package_descriptors
        .into_iter()
        .filter_map(
            |desc| match lookup_package_version_in_indexes(&desc, lookup_index) {
                Ok(v) => Some((v.id().package_id, v)),
                Err(error) => {
                    failures.push(PackageFailure { desc, error });
                    None
                }
            },
        )
        .collect();
    (qualified_versions.into_values().collect(), failures)
}

fn lookup_package_version_in_indexes<'i>(
    desc: &QualifiedPackageDescriptor,
    lookup_index: impl FnOnce(&Url) -> Option<QualifiedIndex<'i>>,
) -> anyhow::Result<QualifiedVersion<'i>> {
    let index = lookup_index(desc.repository_url).context("Repository index unavailable")?;
    lookup_package_version_in_index(desc, index)
}

fn lookup_package_version_in_index<'i>(
    desc: &QualifiedPackageDescriptor,
    index: QualifiedIndex<'i>,
) -> anyhow::Result<QualifiedVersion<'i>> {
    let category = index
        .index
        .find_category(&desc.package.category)
        .context("Couldn't find package category in repository index")?;
    let package = category
        .find_package(&desc.package.name)
        .context("Couldn't find package in repository index")?;
    let version = match &desc.package.version {
        VersionDescriptor::Latest => package
            .latest_stable_version()
            .context("Package has no stable version yet")?,
        VersionDescriptor::LatestPre => package
            .latest_version_including_pre_releases()
            .context("Package has no versions yet, not even pre-release versions")?,
        VersionDescriptor::Specific(v) => package
            .find_version(&v)
            .context("That specific package version is not available in the repository index")?,
    };
    let qualified_version = QualifiedVersion {
        index,
        category,
        package,
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
                .map(|source| QualifiedSource { version: v, source })
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

fn resolve_files_weeding_out_unsupported_sources(
    sources: Vec<QualifiedSource>,
) -> (Vec<QualifiedFile>, Vec<QualifiedSource>) {
    let mut unsupported_sources = vec![];
    let files = sources
        .into_iter()
        .filter_map(|s| {
            let relative_file = s.source.determine_destination_file(
                s.version.index.name,
                &s.version.category.name,
                s.version.package,
            );
            let Some(relative_file) = relative_file else {
                unsupported_sources.push(s);
                return None;
            };
            let file = QualifiedFile {
                source: s,
                relative_path: relative_file,
            };
            Some(file)
        })
        .collect();
    (files, unsupported_sources)
}

fn weed_out_files_with_conflicts(
    files: Vec<QualifiedFile>,
) -> (Vec<QualifiedFile>, Vec<FileConflict>) {
    weed_out_conflicts(
        files,
        |f| f.relative_path.to_string(),
        |relative_path, conflicting_files| FileConflict {
            relative_path,
            conflicting_files,
        },
    )
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
