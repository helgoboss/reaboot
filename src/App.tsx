import {WelcomePanel} from "./pages/WelcomePanel.tsx";
import {Page} from "./components/Page.tsx";
import {PageDescriptor} from "./model/page.ts";
import {PickReaperPanel} from "./pages/PickReaperPanel.tsx";
import {InstallPanel} from "./pages/InstallPanel.tsx";
import {DonePanel} from "./pages/DonePanel.tsx";
import {Stepper} from "./components/Stepper.tsx";
import {mainService, mainStore} from "./globals.ts";
import {debug} from "tauri-plugin-log-api";
import {onMount} from "solid-js";
import {toast, Toaster} from "solid-toast";

export function App() {
    keepSyncingStateFromBackendToStore();
    onMount(() => {
        mainService.configure({
            recipes: mainStore.state.recipes,
            custom_reaper_resource_dir: undefined,
        });
    });
    const currentPageDescriptor = () => pages.find((p) => p.id == mainStore.state.currentPageId)!;
    return <div class="w-screen h-screen flex flex-col">
        <header class="flex-none bg-gray-100"><Stepper pages={pages} currentPageId={currentPageDescriptor().id}/>
        </header>
        <main class="flex-grow bg-white">
            <Page title={currentPageDescriptor().title} description={currentPageDescriptor().description}>
                {currentPageDescriptor().content({})}
            </Page>
        </main>
        <Toaster/>
    </div>
}

function keepSyncingStateFromBackendToStore() {
    debug("Subscribing to ReaBoot events...");
    mainService.getNormalEvents().subscribe((evt) => {
        switch (evt.kind) {
            case "ConfigResolved":
                mainStore.resolvedConfig = evt.state;
                break;
            case "InstallationStatusChanged":
                mainStore.installationStatus = evt.status;
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
        description: "This installer provides an easy and clean way to set up REAPER, ReaPack and add-ons of your choice.",
        content: WelcomePanel,
    },
    {
        id: "pick-reaper",
        title: "Pick REAPER",
        description: "You can install REAPER from scratch or choose an existing installation.",
        content: PickReaperPanel,
    },
    {
        id: "pick-bundles",
        title: "Pick bundles",
        description: "ReaBoot allows you to add initial bundles. Bundles are simply collections of ReaPack packages. You can add packages at a later time, either here or within ReaPack itself.",
        content: PickReaperPanel,
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