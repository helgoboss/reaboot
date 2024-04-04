import {createStore, produce, SetStoreFunction} from "solid-js/store";
import {PageDescriptor, PageId} from "../model/page.ts";
import {InstallationStage} from "../../../core/bindings/InstallationStage.ts";
import {ReabootBackendInfo} from "../../../core/bindings/ReabootBackendInfo.ts";
import {ResolvedInstallerConfig} from "../../src-tauri/bindings/ResolvedInstallerConfig.ts";
import {InstallerConfig} from "../../../core/bindings/InstallerConfig.ts";
import {Accessor, createMemo} from "solid-js";
import {getPage} from "../epics/common.tsx";

export type MainStoreState = {
    // ID of the currently displayed page
    // Dictated by frontend.
    currentPageId: PageId,
    // Last-picked portable REAPER directory
    // Dictated by frontend.
    lastPickedPortableReaperDir?: string,
    // Whether to use the portable REAPER installation.
    // Dictated by frontend.
    usePortableReaperDir: boolean,
    // Whether user has agreed to the REAPER EULA already.
    agreedToReaperEula: boolean,
    // Installer config.
    // Dictated by frontend.
    installerConfig: InstallerConfig,
    // Basic info from the backend.
    // If undefined, it means the backend hasn't sent its info yet.
    // Set in response to event from backend.
    backendInfo?: ReabootBackendInfo,
    // Resolved configuration.
    // If undefined, it means the installer has not been configured yet
    // Set in response to event from backend.
    resolvedConfig?: ResolvedInstallerConfig,
    // Current installation stage.
    // Set in response to event from backend.
    installationStage: InstallationStageContainer,
    // Installation report, HTML-formatted.
    // Set in response to event from backend.
    installationReportHtml?: string,
    // If set, it means that the installer couldn't install REAPER automatically and this contains the path of
    // the installer.
    // Set in response to event from backend.
    manualReaperInstallPath?: string,
    // Currently running tasks
    // Set in response to event from backend.
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

    setCurrentPageId(pageId: PageId) {
        this.setState("currentPageId", pageId);
    }

    setBackendInfo(value: ReabootBackendInfo) {
        this.setState("backendInfo", value);
    }

    setInstallerConfig(value: InstallerConfig) {
        this.setState("installerConfig", value);
    }

    setResolvedConfig(value: ResolvedInstallerConfig) {
        this.setState("resolvedConfig", value);
    }

    setInstallationStage(value: InstallationStageContainer) {
        this.setState("installationStage", value);
    }

    setLastPickedPortableReaperDir(value: string | undefined) {
        this.setState("lastPickedPortableReaperDir", value);
    }

    setInstallationReportHtml(value: string | undefined) {
        this.setState("installationReportHtml", value);
    }

    setManualReaperInstallPath(value: string | undefined) {
        this.setState("manualReaperInstallPath", value);
    }

    agreeToEula() {
        this.setState("agreedToReaperEula", true);
    }

    get currentPage(): Accessor<PageDescriptor> {
        return createMemo(() => getPage(this.state.currentPageId));
    }

    get installationIsRunning(): boolean {
        switch (this.state.installationStage.stage.kind) {
            case "NothingInstalled":
            case "InstalledReaper":
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