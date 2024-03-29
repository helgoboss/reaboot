import {WelcomePage} from "./pages/WelcomePage.tsx";
import {PageDescriptor} from "./model/page.ts";
import {InstallPanel} from "./pages/InstallPanel.tsx";
import {DonePanel} from "./pages/DonePanel.tsx";
import {Stepper} from "./components/Stepper.tsx";
import {mainService, mainStore} from "./globals.ts";
import {debug} from "tauri-plugin-log-api";
import {onMount} from "solid-js";
import {toast, Toaster} from "solid-toast";
import {PickReaperPage} from "./pages/PickReaperPage.tsx";
import {InstallPage} from "./pages/InstallPage.tsx";

export function App() {
    keepSyncingStateFromBackendToStore();
    onMount(() => {
        mainService.configure({
            package_urls: mainStore.state.packageUrls,
            custom_reaper_resource_dir: undefined,
            custom_reaper_target: undefined,
        });
    });
    const page = () => pages.find((p) => p.id == mainStore.state.currentPageId)!;
    return <div class="w-screen h-screen flex flex-col px-8">
        <header class="flex-none p-4">
            <Stepper pages={pages} currentPageId={page().id}/>
        </header>
        <main class="grow flex flex-col">
            {page().content({})}
        </main>
        <Toaster/>
    </div>
}

function keepSyncingStateFromBackendToStore() {
    debug("Subscribing to ReaBoot events...");
    mainService.getNormalEvents().subscribe((evt) => {
        switch (evt.kind) {
            case "ConfigResolved":
                mainStore.resolvedConfig = evt.config;
                break;
            case "InstallationStageChanged":
                mainStore.installationStage = evt.stage;
                break;
            case "Error":
                toast.error(evt.error.display_msg);
                break;
        }
    });
}

const pages: PageDescriptor[] = [
    {
        id: "welcome",
        title: "Welcome!",
        description: "This installer provides an easy and clean way to set up REAPER, ReaPack and packages of your choice.",
        content: WelcomePage,
        // showNav: false,
    },
    {
        id: "pick-reaper",
        title: "Pick REAPER",
        description: "You can install REAPER from scratch or choose an existing installation.",
        content: PickReaperPage,
    },
    {
        id: "pick-bundles",
        title: "Pick bundles",
        description: "ReaBoot allows you to add initial bundles. Bundles are simply collections of ReaPack packages. You can add packages at a later time, either here or within ReaPack itself.",
        content: InstallPage,
    },
    {
        id: "install",
        title: "Install",
        description: "Now it's time to review your settings and start the installation.",
        content: InstallPanel,
    },
    {
        id: "done",
        title: "Done",
        description: "Congratulations! Installation is finished",
        content: DonePanel,
    },
];