import {ProminentChoice} from "../components/ProminentChoice.tsx";
import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {Page} from "../components/Page.tsx";
import {FaRegularFolderOpen} from "solid-icons/fa";
import {open} from "@tauri-apps/api/dialog";
import {mainStore} from "../globals.ts";
import {Match, Show, Switch} from "solid-js";
import {configureInstallation} from "../epics/installation.ts";

export function PickReaperPage() {
    const backendInfo = mainStore.state.backendInfo;
    const resolvedConfig = mainStore.state.resolvedConfig;
    if (!backendInfo || !resolvedConfig) {
        return (
            <div>Waiting for data...</div>
        );
    }
    return (
        <Page>
            <p class="text-center font-bold">
                Please choose the REAPER installation to which you want to add packages.
            </p>
            <p class="text-center">
                <Switch>
                    <Match when={backendInfo.main_reaper_resource_dir_exists}>
                        ReaBoot has detected an existing main REAPER installation on your machine.
                    </Match>
                    <Match when={true}>
                        ReaBoot hasn't found any main REAPER installation on your machine.
                    </Match>
                </Switch>
            </p>
            <div class="grow flex flex-col items-center justify-center gap-4">
                <ProminentChoice selected={!resolvedConfig.portable}
                                 onClick={() => configureInstallation({portable: false})}>
                    <h2 class="card-title">Main REAPER installation</h2>
                    <p class="text-base-content/50">
                        <Show when={!resolvedConfig.portable}>
                            <Switch>
                                <Match when={backendInfo.main_reaper_resource_dir_exists}>
                                    Add packages to your existing main REAPER installation.
                                </Match>
                                <Match when={true}>
                                    Create a new main REAPER installation.
                                </Match>
                            </Switch>
                        </Show>
                    </p>
                </ProminentChoice>
                <ProminentChoice selected={resolvedConfig.portable}
                                 onClick={() => configurePortable(false)}>
                    <div class="flex flex-row items-center gap-4">
                        <div class="flex flex-col">
                            <h2 class="card-title">Portable REAPER installation</h2>
                            <p class="text-base-content/50">
                                Add packages to an existing portable REAPER installation or
                                create a new one.
                            </p>
                        </div>
                        <button class="btn btn-circle">
                            <FaRegularFolderOpen size={24} onClick={(e) => {
                                e.stopPropagation();
                                return configurePortable(true);
                            }}/>
                        </button>
                    </div>
                </ProminentChoice>
            </div>
            <ButtonRow>
                <NavButton onClick={() => mainStore.currentPageId = "pick-packages"}>Continue</NavButton>
            </ButtonRow>
        </Page>
    );
}

async function configurePortable(forcePick: boolean) {
    if (forcePick || !mainStore.state.portableReaperDir) {
        await pickPortableReaperDir();
    }
    configureInstallation({portable: true});
}

async function pickPortableReaperDir() {
    const chosenDir = await open({
        title: "Pick the root directory of your portable REAPER installation!",
        multiple: false,
        directory: true,
    });
    if (chosenDir == null || Array.isArray(chosenDir)) {
        return;
    }
    mainStore.portableReaperDir = chosenDir;
}