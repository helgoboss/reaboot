import {createStore, SetStoreFunction} from "solid-js/store";
import {PageId} from "../model/page.ts";
import {InstallationStatus} from "../../src-lib/bindings/InstallationStatus.ts";
import {Recipe} from "../../src-lib/bindings/Recipe.ts";
import {ResolvedReabootConfig} from "../../src-lib/bindings/ResolvedReabootConfig.ts";

export type MainStoreState = {
    // ID of the currently displayed page.
    currentPageId: PageId,
    // Resolved configuration.
    //
    // If undefined, it means the installer has not been configured yet.
    resolvedConfig?: ResolvedReabootConfig,
    // Current installation status.
    installationStatus: InstallationStatus,
    // Additional recipes to be installed.
    recipes: Recipe[],
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

    set installationStatus(value: InstallationStatus) {
        this.setState("installationStatus", value);
    }

    set recipes(value: Recipe[]) {
        this.setState("recipes", value);
    }
}