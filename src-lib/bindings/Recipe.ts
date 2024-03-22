// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { PackageSet } from "./PackageSet";

/**
 * A collection of repositories and packages to be installed.
 */
export type Recipe = { 
/**
 * Short display name.
 */
name: string, 
/**
 * List of repositories along with packages to install from them.
 */
package_sets: Array<PackageSet>, };