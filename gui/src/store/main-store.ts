import {createStore, SetStoreFunction} from "solid-js/store";
import {PageId} from "../model/page.ts";
import {InstallationStage} from "../../../core/bindings/InstallationStage.ts";
import {ReabootBackendInfo} from "../../../core/bindings/ReabootBackendInfo.ts";
import {ResolvedInstallerConfig} from "../../src-tauri/bindings/ResolvedInstallerConfig.ts";

export type MainStoreState = {
    // ID of the currently displayed page.
    currentPageId: PageId,
    // Basic info from the backend.
    //
    // If undefined, it means the backend hasn't sent its info yet.
    backendInfo?: ReabootBackendInfo,
    // Resolved configuration.
    //
    // If undefined, it means the installer has not been configured yet.
    resolvedConfig?: ResolvedInstallerConfig,
    // Current installation stage.
    installationStage: InstallationStageContainer,
    // Package URLs to be installed.
    packageUrls: string[],
    // Last-picked portable REAPER directory
    portableReaperDir?: string,
    // Installation report, markdown-formatted
    installationReportMarkdown?: string,
}

export type InstallationStageContainer = {
    label: string,
    stage: InstallationStage,
}

export class MainStore {
    readonly state: MainStoreState;
    private readonly setState: SetStoreFunction<MainStoreState>;

    constructor(initialState: MainStoreState) {
        const [state, setState] = createStore<MainStoreState>(initialState);
        this.state = state;
        this.setState = setState;
    }

    set currentPageId(pageId: PageId) {
        this.setState("currentPageId", pageId);
    }

    set backendInfo(value: ReabootBackendInfo) {
        this.setState("backendInfo", value);
    }

    set resolvedConfig(value: ResolvedInstallerConfig) {
        this.setState("resolvedConfig", value);
    }

    set installationStage(value: InstallationStageContainer) {
        this.setState("installationStage", value);
    }

    set packageUrls(value: string[]) {
        this.setState("packageUrls", value);
    }

    set portableReaperDir(value: string) {
        this.setState("portableReaperDir", value);
    }

    set installationReportMarkdown(value: string) {
        this.setState("installationReportMarkdown", value);
    }
}