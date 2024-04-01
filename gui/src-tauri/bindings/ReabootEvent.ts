// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { InstallationStage } from "./InstallationStage";
import type { ReabootBackendInfo } from "./ReabootBackendInfo";
import type { ResolvedInstallerConfig } from "./ResolvedInstallerConfig";

export type ReabootEvent = { "kind": "Error", display_msg: string, } | { "kind": "BackendInfoChanged", info: ReabootBackendInfo, } | { "kind": "ConfigResolved", config: ResolvedInstallerConfig, } | { "kind": "InstallationStageChanged", label: string, stage: InstallationStage, } | { "kind": "TaskStarted", task_id: number, label: string, } | { "kind": "TaskProgressed", task_id: number, progress: number, } | { "kind": "TaskFinished", task_id: number, } | { "kind": "InstallationReportReady", markdown: string, };