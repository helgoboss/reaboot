// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ReaperPlatform } from "./ReaperPlatform";
import type { Recipe } from "./Recipe";
import type { VersionRef } from "./VersionRef";

/**
 * Data structure for configuring the installer from the frontend.
 */
export type InstallerConfig = { 
/**
 * Custom REAPER resource directory to which to install ReaPack and packages.
 *
 * If not provided, it will take the main resource directory for the currently logged-in user.
 *
 * If this directory doesn't contain `reaper.ini`, ReaBoot assumes that REAPER is
 * not installed yet. In this case, it will attempt to install REAPER. It will
 * make a main installation if this corresponds to the typical REAPER resource path of
 * a main installation, otherwise it will make a portable install **into** this
 * directory.
 */
custom_reaper_resource_dir?: string, 
/**
 * OS and architecture for which to install things.
 *
 * Influences the choice of binaries (REAPER, ReaPack and packages).
 *
 * If not provided, this will be derived from the OS and architecture for which ReaBoot itself
 * was compiled.
 */
custom_platform?: ReaperPlatform, 
/**
 * A list of package URLs pointing to packages to be installed.
 *
 * These recipes will be installed *in addition* to those that are going to be installed
 * anyway (if the installer is branded).
 */
package_urls: Array<string>, 
/**
 * Maximum number of retries if a download fails.
 */
num_download_retries?: number, 
/**
 * The directory in which the installer creates a randomly-named temporary directory for
 * download purposes.
 *
 * If not provided, this will be `{REAPER_RESOURCE_DIR}/ReaBoot`.
 */
temp_parent_dir?: string, 
/**
 * Whether to keep the temporary directory or not.
 */
keep_temp_dir: boolean, 
/**
 * Maximum number of concurrent downloads.
 */
concurrent_downloads?: number, 
/**
 * If `true`, nothing will be installed.
 *
 * A good way to check if the installation would (most likely) succeed.
 */
dry_run: boolean, 
/**
 * Which REAPER version to install if it doesn't exist already.
 */
reaper_version?: VersionRef, 
/**
 * If `true`, the installer will succeed even if there are failed packages.
 */
skip_failed_packages: boolean, 
/**
 * An optional recipe.
 */
recipe?: Recipe, 
/**
 * The set of recipe features to be installed.
 *
 * Features not contained in the recipe will be ignored.
 */
selected_features: Array<string>, 
/**
 * Update REAPER if there's a new version available.
 */
update_reaper: boolean, };