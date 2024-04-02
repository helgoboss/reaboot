import {DummyMainService} from "./services/dummy-main-service.ts";
import {MainStore} from "./store/main-store.ts";
import {TauriMainService} from "./services/tauri-main-service.ts";
import {Accessor, createSignal} from "solid-js";
import {appWindow, Theme} from "@tauri-apps/api/window";

export const useDummyService = false;

export const mainService = useDummyService ? new DummyMainService() : new TauriMainService();

export const themeSignal = createThemeSignal();

export const mainStore = new MainStore({
    currentPageId: "welcome",
    resolvedConfig: undefined,
    packageUrls: [],
    packageUrlsExpression: "",
    invalidPackageUrls: [],
    installationStage: {
        stage: {
            kind: "NothingInstalled",
        },
        label: "",
    },
    current_tasks: []
});

function createThemeSignal(): Accessor<Theme | undefined> {
    const [theme, setTheme] = createSignal<Theme>();
    appWindow.theme().then(theme => setTheme(theme ?? undefined));
    appWindow.onThemeChanged((evt) => {
        setTheme(evt.payload);
    });
    return theme;
}