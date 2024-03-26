// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { VersionDescriptor } from "./VersionDescriptor";

/**
 * Uniquely identifies a specific version of a package, within the context of a repository.
 */
export type PackageDescriptor = { 
/**
 * Category of the package.
 */
category: string, 
/**
 * Package name.
 */
name: string, 
/**
 * Describes the version to be installed.
 */
version: VersionDescriptor, };