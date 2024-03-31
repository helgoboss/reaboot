import {createStore, SetStoreFunction} from "solid-js/store";
import {PageId} from "../model/page.ts";
import {ResolvedReabootConfig} from "../../../core/bindings/ResolvedReabootConfig.ts";
import {InstallationStage} from "../../../core/bindings/InstallationStage.ts";

export type MainStoreState = {
    // ID of the currently displayed page.
    currentPageId: PageId,
    // Resolved configuration.
    //
    // If undefined, it means the installer has not been configured yet.
    resolvedConfig?: ResolvedReabootConfig,
    // Current installation stage.
    installationStage: InstallationStage,
    // Package URLs to be installed.
    packageUrls: string[],
    // Last-picked portable REAPER directory
    portableReaperDir?: string,
    // Whether a main REAPER installation exists on this machine
    mainReaperInstallationExists: boolean,
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

    set resolvedConfig(value: ResolvedReabootConfig) {
        this.setState("resolvedConfig", value);
    }

    set installationStage(value: InstallationStage) {
        this.setState("installationStage", value);
    }

    set packageUrls(value: string[]) {
        this.setState("packageUrls", value);
    }

    set portableReaperDir(value: string) {
        this.setState("portableReaperDir", value);
    }
}