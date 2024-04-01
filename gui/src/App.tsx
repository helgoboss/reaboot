import {WelcomePage} from "./pages/WelcomePage.tsx";
import {PageDescriptor} from "./model/page.ts";
import {DonePanel} from "./pages/DonePanel.tsx";
import {Stepper} from "./components/Stepper.tsx";
import {mainService, mainStore} from "./globals.ts";
import {debug} from "tauri-plugin-log-api";
import {onMount, Show} from "solid-js";
import {toast, Toaster} from "solid-toast";
import {PickReaperPage} from "./pages/PickReaperPage.tsx";
import {InstallPage} from "./pages/InstallPage.tsx";
import {PickPackagesPage} from "./pages/PickPackagesPage.tsx";
import {configureInstallation} from "./epics/installation.ts";

export function App() {
    keepSyncingStateFromBackendToStore();
    onMount(() => {
        configureInstallation({portable: false});
    });
    const page = () => pages.find((p) => p.id == mainStore.state.currentPageId)!;
    const resolvedConfig = () => mainStore.state.resolvedConfig;
    return <div class="w-screen h-screen flex flex-col">
        <header class="flex-none p-4">
            <Stepper pages={pages} currentPageId={page().id}/>
        </header>
        <main class="grow flex flex-col">
            {page().content({})}
        </main>
        <Show when={page().showFooter != false}>
            <Show when={resolvedConfig()}>
                {(conf) =>
                    <footer class="p-4 bg-base-300 text-base-content flex flex-row text-xs">
                        <div class="font-bold">REAPER resource path:</div>
                        <div class="grow font-mono text-right">{conf().reaper_resource_dir}</div>
                    </footer>
                }
            </Show>
        </Show>
        <Toaster/>
    </div>
}

function keepSyncingStateFromBackendToStore() {
    debug("Subscribing to ReaBoot events...");
    mainService.getNormalEvents().subscribe((evt) => {
        switch (evt.kind) {
            case "BackendInfoChanged":
                mainStore.backendInfo = evt.info;
                break;
            case "ConfigResolved":
                mainStore.resolvedConfig = evt.config;
                break;
            case "InstallationStageChanged":
                mainStore.installationStage = {
                    label: evt.label,
                    stage: evt.stage,
                };
                if (evt.stage.kind === "Finished" || evt.stage.kind === "Failed") {
                    mainStore.currentPageId = "done";
                }
                break;
            case "InstallationReportReady":
                mainStore.installationReportMarkdown = evt.markdown;
                break;
            case "Error":
                toast.error(evt.display_msg);
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
        showFooter: false,
    },
    {
        id: "pick-reaper",
        title: "Pick REAPER",
        description: "You can install REAPER from scratch or choose an existing installation.",
        content: PickReaperPage,
    },
    {
        id: "pick-packages",
        title: "Pick packages",
        description: "ReaBoot allows you to add initial bundles. Bundles are simply collections of ReaPack packages. You can add packages at a later time, either here or within ReaPack itself.",
        content: PickPackagesPage,
    },
    {
        id: "install",
        title: "Install",
        description: "Now it's time to review your settings and start the installation.",
        content: InstallPage,
    },
    {
        id: "done",
        title: "Done",
        description: "Congratulations! Installation is finished",
        content: DonePanel,
    },
];