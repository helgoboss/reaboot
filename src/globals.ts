import {DummyMainService} from "./services/dummy-main-service.ts";
import {MainStore} from "./store/main-store.ts";
import {TauriMainService} from "./services/tauri-main-service.ts";

export const mainService = new TauriMainService();

export const mainStore = new MainStore({
    currentPageId: "welcome",
    mainReaperResourceDir: undefined,
});