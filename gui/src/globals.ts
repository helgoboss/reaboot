import {DummyMainService} from "./services/dummy-main-service.ts";
import {MainStore} from "./store/main-store.ts";
import {TauriMainService} from "./services/tauri-main-service.ts";

export const useDummyService = false;

export const mainService = useDummyService ? new DummyMainService() : new TauriMainService();

export const mainStore = new MainStore({
    currentPageId: "welcome",
    resolvedConfig: undefined,
    packageUrls: [],
    installationStage: {
        stage: {
            kind: "NothingInstalled",
        },
        label: "",
    }
});