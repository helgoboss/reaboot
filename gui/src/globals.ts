import {DummyMainService} from "./services/dummy-main-service.ts";
import {MainStore} from "./store/main-store.ts";
import {TauriMainService} from "./services/tauri-main-service.ts";
import {PageDescriptor} from "./model/page.ts";
import {WelcomePage} from "./pages/WelcomePage.tsx";
import {PickReaperPage} from "./pages/PickReaperPage.tsx";
import {CustomizePage} from "./pages/CustomizePage.tsx";
import {InstallPage} from "./pages/InstallPage.tsx";
import {DonePage} from "./pages/DonePage.tsx";

export const useDummyService = false;

export const mainService = useDummyService ? new DummyMainService() : new TauriMainService();

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
    expertMode: false,
    installationStage: {
        stage: {
            kind: "NothingInstalled",
        },
        label: "",
    },
    currentTasks: [],
});

export const pages: PageDescriptor[] = [
    {
        id: "welcome",
        title: "Welcome",
        content: WelcomePage,
        showFooter: false,
    },
    {
        id: "pick-reaper",
        title: "Pick REAPER",
        content: PickReaperPage,
    },
    {
        id: "customize",
        title: "Customize",
        content: CustomizePage,
    },
    {
        id: "install",
        title: "Install",
        content: InstallPage,
    },
    {
        id: "done",
        title: "Done",
        content: DonePage,
    },
];
