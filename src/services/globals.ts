import {DummyMainService} from "./dummy-main-service.ts";
import {MainStore} from "../store/main-store.ts";

export const mainService = new DummyMainService();

export const mainStore = new MainStore({
    currentPageId: "welcome",
    mainReaperResourceDir: undefined,
});