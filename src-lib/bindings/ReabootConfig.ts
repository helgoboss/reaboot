// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Recipe } from "./Recipe";

/**
 * Command for configuring the installation process.
 */
export type ReabootConfig = { 
/**
 * Custom REAPER resource directory, most likely the one of a portable REAPER installation.
 *
 * If `None`, we will use the resource directory of the main installation.
 */
custom_reaper_resource_dir?: string, 
/**
 * A list of recipes with packages to install.
 *
 * These recipes will be installed *in addition* to the packages that are going to be installed
 * anyway (if the installer is branded).
 */
recipes: Array<Recipe>, };