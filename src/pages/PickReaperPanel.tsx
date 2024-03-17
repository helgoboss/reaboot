import {MainButton} from "../components/MainButton.tsx";
import {ButtonList} from "../components/ButtonList.tsx";
import {SecondaryButton} from "../components/SecondaryButton.tsx";
import {onMount} from "solid-js";
import {mainService, mainStore} from "../globals.ts";
import {open} from '@tauri-apps/api/dialog';

export function PickReaperPanel() {
    onMount(async () => {
        mainStore.mainReaperResourceDir = await mainService.getMainReaperResourceDir();
    });
    const mainReaperResourcePath = mainStore.state.mainReaperResourceDir;
    if (mainReaperResourcePath == null) {
        return (
            <>
                <div>
                    We didn't detect any existing REAPER main installation.
                </div>
                <ButtonList>
                    <MainButton>Install REAPER</MainButton>
                    <SecondaryButton onClick={pickPortableReaperInstallation}>Pick portable REAPER
                        installation</SecondaryButton>
                </ButtonList>
            </>
        );
    } else {
        return (
            <>
                <div>
                    We detected an existing main installation of REAPER with resource path: <span
                    class="font-mono">{mainReaperResourcePath}</span>
                </div>
                <ButtonList>
                    <MainButton>Use main REAPER installation</MainButton>
                    <SecondaryButton onClick={pickPortableReaperInstallation}>Pick portable REAPER
                        installation</SecondaryButton>
                    <SecondaryButton>Create new portable REAPER installation</SecondaryButton>
                </ButtonList>
            </>
        );
    }
}

async function pickPortableReaperInstallation() {
    const selected = await open({
        title: "Pick the root directory of your portable REAPER installation!",
        multiple: false,
        directory: true,
    });
    if (selected == null || Array.isArray(selected)) {
        return;
    }
    mainStore.chosenReaperResourceDir = selected;
}