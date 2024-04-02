import {ProminentChoice} from "../components/ProminentChoice.tsx";
import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {Page} from "../components/Page.tsx";
import {open} from "@tauri-apps/api/dialog";
import {mainStore} from "../globals.ts";
import {Match, Switch} from "solid-js";
import {configureInstallation} from "../epics/install.ts";
import {MainInstallationIcon, PortableInstallationIcon} from "../components/icons.tsx";
import {WaitingForDataPage} from "./WaitingForDataPage.tsx";

export function PickReaperPage() {
    const backendInfo = mainStore.state.backendInfo;
    const resolvedConfig = mainStore.state.resolvedConfig;
    if (!backendInfo || !resolvedConfig) {
        return <WaitingForDataPage/>;
    }
    return (
        <Page>
            <Switch>
                <Match when={backendInfo.main_reaper_resource_dir_exists}>
                    <p class="text-center font-bold">
                        Which REAPER installation do you want to modify?
                    </p>
                </Match>
                <Match when={true}>
                    <p class="text-center font-bold">
                        Please decide whether you want to create a main or a portable installation!
                    </p>
                </Match>
            </Switch>
            <div class="grow flex flex-col items-center justify-center gap-4">
                <ProminentChoice selected={!resolvedConfig.portable}
                                 icon={<MainInstallationIcon size="24"/>}
                                 topRightIndicator="Good default choice"
                                 onClick={() => configureInstallation({portable: false})}>
                    <h2 class="card-title">
                        Main REAPER installation
                    </h2>
                    <p class="text-base-content/50">
                        <Switch>
                            <Match when={backendInfo.main_reaper_resource_dir_exists}>
                                Add packages to your existing main REAPER installation.
                            </Match>
                            <Match when={true}>
                                Create a new main REAPER installation.
                            </Match>
                        </Switch>
                    </p>
                </ProminentChoice>
                <ProminentChoice selected={resolvedConfig.portable}
                                 icon={<PortableInstallationIcon size="24"/>}
                                 bottomRightIndicator={<button class="btn btn-accent" onClick={(e) => {
                                     e.stopPropagation();
                                     return configurePortable(true);
                                 }}>
                                     Choose directory...
                                 </button>}
                                 onClick={() => configurePortable(false)}>
                    <div class="flex flex-row items-center gap-4">
                        <div class="flex flex-col">
                            <h2 class="card-title">
                                Portable REAPER installation
                            </h2>
                            <div>
                                <p class="text-base-content/50">
                                    Add packages to an existing portable REAPER installation or
                                    create a new one.
                                </p>
                            </div>
                        </div>
                    </div>
                </ProminentChoice>
            </div>
            <ButtonRow>
                <NavButton onClick={() => mainStore.currentPageId = "pick-packages"}>Continue</NavButton>
            </ButtonRow>
        </Page>
    )
        ;
}

async function configurePortable(forcePick: boolean) {
    if (forcePick || !mainStore.state.portableReaperDir) {
        await pickPortableReaperDir();
    }
    if (mainStore.state.portableReaperDir) {
        configureInstallation({portable: true});
    }
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