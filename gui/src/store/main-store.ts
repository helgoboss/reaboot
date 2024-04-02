import {createStore, produce, SetStoreFunction} from "solid-js/store";
import {PageId} from "../model/page.ts";
import {InstallationStage} from "../../../core/bindings/InstallationStage.ts";
import {ReabootBackendInfo} from "../../../core/bindings/ReabootBackendInfo.ts";
import {ResolvedInstallerConfig} from "../../src-tauri/bindings/ResolvedInstallerConfig.ts";

export type MainStoreState = {
    // ID of the currently displayed page
    currentPageId: PageId,
    // Basic info from the backend.
    //
    // If undefined, it means the backend hasn't sent its info yet.
    backendInfo?: ReabootBackendInfo,
    // Resolved configuration.
    //
    // If undefined, it means the installer has not been configured yet
    resolvedConfig?: ResolvedInstallerConfig,
    // Current installation stage.
    installationStage: InstallationStageContainer,
    // Plain text from which package URLs are extracted
    packageUrlsExpression: string,
    // Package URLs to be installed
    packageUrls: string[],
    // Invalid package URLs
    invalidPackageUrls: string[],
    // Last-picked portable REAPER directory
    portableReaperDir?: string,
    // Installation report, HTML-formatted
    installationReportHtml?: string,
    // Currently running tasks
    current_tasks: ReabootTask[],
}

export type ReabootTask = {
    id: number,
    label: string,
    progress: number,
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

    set packageUrlsExpression(value: string) {
        this.setState("packageUrlsExpression", value);
    }

    set packageUrls(value: string[]) {
        this.setState("packageUrls", value);
    }

    set invalidPackageUrls(value: string[]) {
        this.setState("invalidPackageUrls", value);
    }

    set portableReaperDir(value: string | undefined) {
        this.setState("portableReaperDir", value);
    }

    set installationReportHtml(value: string) {
        this.setState("installationReportHtml", value);
    }

    get installationIsRunning(): boolean {
        switch (this.state.installationStage.stage.kind) {
            case "NothingInstalled":
            case "InstalledReaper":
            case "InstalledReaPack":
            case "Failed":
            case "Finished":
                return false;
            default:
                return true;
        }
    }

    addTask(id: number, label: string) {
        this.setState(
            produce((state) => {
                state.current_tasks.push({
                    id,
                    label,
                    progress: 0.0,
                });
            })
        );
    }

    updateTaskProgress(id: number, progress: number) {
        this.setState(
            produce((state) => {
                const task = state.current_tasks.find(t => t.id === id);
                if (task) {
                    task.progress = progress;
                }
            })
        );
    }

    removeTask(id: number) {
        this.setState("current_tasks", tasks => tasks.filter(t => t.id !== id));
    }
}