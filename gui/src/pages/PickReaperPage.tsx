import {ProminentChoice} from "../components/ProminentChoice.tsx";
import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {Page} from "../components/Page.tsx";
import {open} from "@tauri-apps/api/dialog";
import {mainStore} from "../globals.ts";
import {Match, Show, Switch} from "solid-js";
import {configureInstaller} from "../epics/install.ts";
import {MainInstallationIcon, PortableInstallationIcon} from "../components/icons.tsx";
import {WaitingForDataPage} from "./WaitingForDataPage.tsx";
import {navigateTo} from "../epics/common.tsx";
import {Switch as KSwitch} from "@kobalte/core";

export function PickReaperPage() {
    const backendInfo = mainStore.state.backendInfo;
    const resolvedConfig = mainStore.state.resolvedConfig;
    if (!backendInfo || !resolvedConfig) {
        return <WaitingForDataPage/>;
    }
    return (
        <Page>
            <p class="text-center font-bold">
                <Switch>
                    <Match when={backendInfo.main_reaper_exe_exists}>
                        Which REAPER installation do you want to modify?
                    </Match>
                    <Match when={true}>
                        Do you want to create a new main REAPER installation or pick a portable one?
                    </Match>
                </Switch>
            </p>
            <div class="grow my-4 flex flex-col items-center justify-center gap-4">
                <ProminentChoice selected={!resolvedConfig.portable}
                                 icon={<MainInstallationIcon size="24"/>}
                                 topRightIndicator="Good default choice"
                                 onClick={() => configureInstaller({customReaperResourceDir: null})}>
                    <h2 class="card-title">
                        Main REAPER installation
                    </h2>
                    <p class="text-base-content/50">
                        <Switch>
                            <Match when={backendInfo.main_reaper_exe_exists}>
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
                                 bottomRightIndicator={mainStore.state.lastPickedPortableReaperDir &&
                                     <button class="btn btn-accent" onClick={(e) => {
                                         e.stopPropagation();
                                         return configurePortable(true);
                                     }}>
                                         Change directory...
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
            <Show when={resolvedConfig.reaper_exe_exists}>
                <KSwitch.Root class="self-center mb-4 flex flex-row gap-2"
                              checked={mainStore.state.installerConfig.update_reaper}
                              onChange={on => configureInstaller({updateReaper: on})}>
                    <KSwitch.Label>Update REAPER if new version available</KSwitch.Label>
                    <KSwitch.Input/>
                    <KSwitch.Control>
                        <KSwitch.Thumb class="toggle toggle-primary"
                                       aria-checked={mainStore.state.installerConfig.update_reaper}/>
                    </KSwitch.Control>
                </KSwitch.Root>
            </Show>
            <ButtonRow>
                <NavButton onClick={() => navigateTo("customize")}>Continue</NavButton>
            </ButtonRow>
        </Page>
    );
}

async function configurePortable(forcePick: boolean) {
    if (forcePick || !mainStore.state.lastPickedPortableReaperDir) {
        await pickPortableReaperDir();
    }
    configureInstaller({customReaperResourceDir: mainStore.state.lastPickedPortableReaperDir});
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
    mainStore.setLastPickedPortableReaperDir(chosenDir);
}