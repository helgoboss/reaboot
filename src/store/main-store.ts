import {createStore, SetStoreFunction} from "solid-js/store";
import {PageId} from "../model/page.ts";

export type MainStoreState = {
    currentPageId: PageId,
    mainReaperResourceDir?: string,
    chosenReaperResourceDir?: string,
}

export class MainStore {
    readonly state: MainStoreState;
    private readonly setState: SetStoreFunction<MainStoreState>;

    constructor(initialState: MainStoreState) {
        const [state, setState] = createStore<MainStoreState>(initialState);
        this.state = state;
        this.setState = setState;
    }

    openPage(pageId: PageId) {
        this.setState("currentPageId", pageId);
    }

    set mainReaperResourceDir(value: string | undefined) {
        this.setState("mainReaperResourceDir", value);
    }

    set chosenReaperResourceDir(value: string | undefined) {
        this.setState("chosenReaperResourceDir", value);
    }
}