import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {Page} from "../components/Page.tsx";
import {mainStore} from "../globals.ts";
import {configureInstaller} from "../epics/install.ts";
import {WaitingForDataPage} from "./WaitingForDataPage.tsx";
import {PackageTable} from "../components/PackageTable.tsx";
import {clipboard} from "@tauri-apps/api";
import {FaSolidCheck, FaSolidCirclePlus, FaSolidX} from "solid-icons/fa";
import {navigateTo, showError} from "../epics/common.tsx";
import {createSignal, For, Show} from "solid-js";

export function CustomizePage() {
    const resolvedConfig = mainStore.state.resolvedConfig;
    if (!resolvedConfig) {
        return <WaitingForDataPage/>;
    }
    const showFeaturePane = () => mainStore.parsedRecipeFeatures().length > 0;
    const [featureHelp, setFeatureHelp] = createSignal<string | null>(null);
    return (
        <Page>
            <p class="text-center font-bold">
                Optional: Customize installation
            </p>
            <div class="grow flex flex-row items-stretch justify-stretch gap-5 min-h-0 min-w-0 pt-3">
                <Show when={showFeaturePane()}>
                    <div class="basis-1/2 card card-compact bg-base-200 min-h-0 min-w-0">
                        <div class="card-body min-h-0 overflow-x-hidden">
                            <h2 class="card-title text-base">Toggle features on/off</h2>
                            <div class="basis-3/4 overflow-y-auto">
                                <ul class="flex flex-wrap gap-2">
                                    <For each={mainStore.parsedRecipeFeatures()}>
                                        {([id, feature]) => {
                                            return <li>
                                                <button class="badge flex flex-row gap-1"
                                                        classList={{"badge-accent": mainStore.featureIsSelected(id)}}
                                                        onClick={() => toggleFeature(id)}
                                                        onMouseEnter={() => setFeatureHelp(feature.raw.description ?? null)}
                                                        onMouseLeave={() => setFeatureHelp(null)}>
                                                    {feature.raw.name}
                                                    {mainStore.featureIsSelected(id) ? <FaSolidCheck/> : <FaSolidX/>}
                                                </button>
                                            </li>;
                                        }
                                        }
                                    </For>
                                </ul>
                            </div>
                            <div class="basis-1/4 overflow-y-auto">
                                {featureHelp()}
                            </div>
                        </div>
                    </div>
                </Show>
                <div class="grow flex flex-col min-h-0" classList={{"basis-1/2": showFeaturePane()}}>
                    <Show when={mainStore.showAddCustomPackagesButton}>
                        <ButtonRow class="pt-0">
                            <button class="btn m-0" onClick={() => addPackageUrlsFromClipboard()}
                                    title="This expects a list of package URLs in your clipboard.">
                                <FaSolidCirclePlus size={14}/> Add custom packages from clipboard
                            </button>
                        </ButtonRow>
                    </Show>
                    <div class="grow card card-compact bg-base-200 min-h-0">
                        <div class="card-body min-h-0">
                            <h2 class="card-title text-base">Final package list</h2>
                            <div class="overflow-y-auto">
                                <PackageTable packages={resolvedConfig.package_urls}/>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            <p class="text-center text-sm px-8 mt-3">
                Packages which are already installed will be
                replaced with the new version.
                You can add more packages later at any time, either
                by starting ReaBoot again or by using ReaPack in REAPER
                (Extensions&nbsp;→&nbsp;ReaPack&nbsp;→&nbsp;Browse packages).
            </p>
            <ButtonRow>
                <NavButton onClick={() => navigateTo("install")}>
                    {resolvedConfig.package_urls.length == 0 ? "Continue without packages" : "Continue"}
                </NavButton>
            </ButtonRow>
        </Page>
    );
}

async function addPackageUrlsFromClipboard() {
    const text = await clipboard.readText();
    if (text == null) {
        showError("Clipboard is empty");
        return;
    }
    await addPackageUrls(text);
}


async function addPackageUrls(text: string) {
    // Parse lines
    const newPackageUrls = text.split("\n").map(line => line.trim()).filter(line => line.length !== 0);
    // Merge with existing package URLs and deduplicate
    const allPackages = new Set([...mainStore.state.installerConfig.package_urls, ...newPackageUrls]);
    // Configure
    await configureInstaller({packageUrls: [...allPackages]});
}

function toggleFeature(featureId: string) {
    configureInstaller({
        selectedFeatures: mainStore.featureIsSelected(featureId)
            ? mainStore.state.installerConfig.selected_features.filter(id => id !== featureId)
            : [...mainStore.state.installerConfig.selected_features, featureId]
    });
}