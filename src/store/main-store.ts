import {createStore} from "solid-js/store";
import {PageId} from "../model/page.ts";

export type MainStore = {
    currentPageId: PageId,
    mainReaperResourcePath?: string,
}

function createMainStore() {
    const [state, setState] = createStore<MainStore>(initialState);
    return {
        state,
        openPage: (pageId: PageId) => {
            setState("currentPageId", pageId);
        }
    };
}


const initialState: MainStore = {
    currentPageId: "welcome",
    mainReaperResourcePath: undefined,
};

export default createMainStore();