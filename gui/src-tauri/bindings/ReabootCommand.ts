// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { InstallerConfig } from "./InstallerConfig";

export type ReabootCommand = { "kind": "ConfigureInstallation", config: InstallerConfig, } | { "kind": "StartInstallation" } | { "kind": "CancelInstallation" };