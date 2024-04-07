import {Stepper} from "./components/Stepper.tsx";
import {mainService, mainStore, pages} from "./globals.ts";
import {debug} from "tauri-plugin-log-api";
import {Match, Show, Switch} from "solid-js";
import {Toaster} from "solid-toast";
import {configureInstaller} from "./epics/install.ts";
import {MainInstallationIcon, PortableInstallationIcon} from "./components/icons.tsx";
import {navigateTo, showError, showWarning} from "./epics/common.tsx";
import {GlobalDialog} from "./components/GlobalDialog.tsx";
import {clipboard} from "@tauri-apps/api";
import {Recipe} from "../../core/bindings/Recipe.ts";

export function App() {
    keepSyncingStateFromBackendToStore();
    // Right at the beginning, we configure the installer exactly once with default values.
    // This makes the backend give us all necessary data.
    void configureInstaller({});
    void detectInitialRecipe();
    const resolvedConfig = () => mainStore.state.resolvedConfig;
    return <div class="w-screen h-screen flex flex-col min-h-0">
        <header class="flex-none p-4">
            <Stepper pages={pages} currentPageId={mainStore.currentPage().id}/>
        </header>
        <main class="grow flex flex-col min-h-0">
            {mainStore.currentPage().content({})}
        </main>
        <Show when={mainStore.currentPage().showFooter != false}>
            <Show when={resolvedConfig()}>
                {(conf) =>
                    <footer class="p-4 bg-base-300 text-base-content flex flex-row text-xs gap-3">
                        <div class="text-left font-bold tooltip tooltip-right"
                             data-tip="Folder in which REAPER saves user data, e.g. preferences and scripts.">
                            REAPER resource path:
                        </div>
                        <div class="grow"></div>
                        <div>
                            <Switch>
                                <Match when={conf().portable}><PortableInstallationIcon size={14}/></Match>
                                <Match when={true}><MainInstallationIcon size={14}/></Match>
                            </Switch>
                        </div>
                        <div class="font-mono">
                            {conf().reaper_resource_dir}
                        </div>
                        <Show when={!conf().reaper_exe_exists}>
                            <div class="badge badge-secondary badge-sm">new</div>
                        </Show>
                    </footer>
                }
            </Show>
        </Show>
        <GlobalDialog/>
        <Toaster/>
    </div>
}

function keepSyncingStateFromBackendToStore() {
    debug("Subscribing to ReaBoot events...");
    mainService.getNormalEvents().subscribe((evt) => {
        switch (evt.kind) {
            case "BackendInfoChanged":
                mainStore.setBackendInfo(evt.info);
                break;
            case "ConfigResolved":
                mainStore.setResolvedConfig(evt.config);
                break;
            case "InstallationStageChanged":
                mainStore.setInstallationStage(evt);
                if (mainStore.state.currentPageId === "install") {
                    if (evt.stage.kind === "Finished" || evt.stage.kind === "Failed") {
                        navigateTo("done");
                    }
                }
                break;
            case "InstallationDone":
                mainStore.setInstallationReportHtml(evt.preparation_report_html);
                mainStore.setManualReaperInstallPath(evt.manual_reaper_install_path);
                break;
            case "TaskStarted":
                mainStore.addTask(evt.task_id, evt.label);
                break;
            case "TaskProgressed":
                mainStore.updateTaskProgress(evt.task_id, evt.progress);
                break;
            case "TaskFinished":
                mainStore.removeTask(evt.task_id);
                break;
            case "Info":
                // showInfo(evt.display_msg);
                break;
            case "Warn":
                showWarning(evt.display_msg);
                break;
            case "Error":
                showError(evt.display_msg);
                break;
        }
    });
}

async function detectInitialRecipe() {
    const text = await clipboard.readText();
    if (text == null) {
        return;
    }
    const recipe = await extractRecipe(text);
    if (!recipe) {
        return;
    }
    await configureInstaller({recipe});
}

async function extractRecipe(text: string): Promise<Recipe | undefined> {
    // At first, check if the text itself represents a recipe (= is JSON)
    const recipe = parseRecipe(text);
    if (recipe) {
        return recipe;
    }
    // If not, check if it's a URL and fetch it as recipe
    const recipeUrl = extractUrl(text);
    if (recipeUrl == null) {
        return;
    }
    return fetchRecipe(recipeUrl);
}

async function fetchRecipe(url: URL): Promise<Recipe | undefined> {
    // Send request
    const response = await fetch(url);
    if (!response.ok) {
        return;
    }
    // Don't continue if response contains too much data
    const maxContentLength = 100 * 1024;
    const contentLength = response.headers.get("Content-Length");
    if (contentLength === null || parseInt(contentLength) > maxContentLength) {
        return;
    }
    // Try to read response as text
    const text = await response.text().catch(() => undefined);
    if (!text) {
        return;
    }
    // Parse response text as recipe
    return parseRecipe(text);
}

function parseRecipe(text: string): Recipe | undefined {
    try {
        return JSON.parse(text);
    } catch {
        return;
    }
}

function extractUrl(text: string): URL | undefined {
    const lines = text.split("\n");
    if (lines.length !== 1) {
        return;
    }
    return parseUrl(lines[0]);
}

function parseUrl(text: string): URL | undefined {
    try {
        return new URL(text);
    } catch {
        return;
    }
}