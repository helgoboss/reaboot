import {DummyMainService} from "./services/dummy-main-service.ts";
import {MainStore} from "./store/main-store.ts";
import {TauriMainService} from "./services/tauri-main-service.ts";
import {Accessor} from "solid-js";
import {Theme} from "@tauri-apps/api/window";
import {PageDescriptor} from "./model/page.ts";
import {WelcomePage} from "./pages/WelcomePage.tsx";
import {PickReaperPage} from "./pages/PickReaperPage.tsx";
import {CustomizePage} from "./pages/CustomizePage.tsx";
import {InstallPage} from "./pages/InstallPage.tsx";
import {DonePage} from "./pages/DonePage.tsx";

export const useDummyService = false;

export const mainService = useDummyService ? new DummyMainService() : new TauriMainService();

export const themeSignal = createThemeSignal();

export const mainStore = new MainStore({
    currentPageId: "welcome",
    agreedToReaperEula: false,
    installerConfig: {
        custom_reaper_resource_dir: undefined,
        package_urls: [],
        keep_temp_dir: false,
        dry_run: false,
        skip_failed_packages: false,
        update_reaper: false,
        concurrent_downloads: 8,
        selected_features: [""],
    },
    usePortableReaperDir: false,
    installationStage: {
        stage: {
            kind: "NothingInstalled",
        },
        label: "",
    },
    current_tasks: [],
});

export const pages: PageDescriptor[] = [
    {
        id: "welcome",
        title: "Welcome",
        content: WelcomePage,
        showFooter: false,
        requiresReaperEulaAgreement: false,
    },
    {
        id: "pick-reaper",
        title: "Pick REAPER",
        content: PickReaperPage,
        requiresReaperEulaAgreement: false,
    },
    {
        id: "customize",
        title: "Customize",
        content: CustomizePage,
        requiresReaperEulaAgreement: true,
    },
    {
        id: "install",
        title: "Install",
        content: InstallPage,
        requiresReaperEulaAgreement: true,
    },
    {
        id: "done",
        title: "Done",
        content: DonePage,
        isRandomlyAccessible: false,
        requiresReaperEulaAgreement: false,
    },
];

function createThemeSignal(): Accessor<Theme | undefined> {
    return () => "dark";
    //// Use the following only when following the OS dark mode
    // const [theme, setTheme] = createSignal<Theme>();
    // appWindow.theme().then(theme => setTheme(theme ?? undefined));
    // appWindow.onThemeChanged((evt) => {
    //     setTheme(evt.payload);
    // });
    // return theme;
}