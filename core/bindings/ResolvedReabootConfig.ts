// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ReaperTarget } from "./ReaperTarget";

/**
 * State that should only change on configuration changes and a REAPER install, not during
 * the further installation process.
 */
export type ResolvedReabootConfig = { 
/**
 * The resolved REAPER resource directory which is going to be used for the installation.
 *
 * [`InstallationStatus`] indicates, whether this directory exists and is a valid
 * REAPER resource directory.
 */
reaper_resource_dir: string, 
/**
 * `true` if the resource directory is part of a portable REAPER installation (not the one of
 * the main REAPER installation).
 */
portable: boolean, 
/**
 * Resolved REAPER target.
 */
reaper_target: ReaperTarget, };