import {MainButton} from "../components/MainButton.tsx";
import {ButtonList} from "../components/ButtonList.tsx";
import {SecondaryButton} from "../components/SecondaryButton.tsx";
import {onMount} from "solid-js";
import {mainService, mainStore} from "../services/globals.ts";

export function PickReaperPanel() {
    onMount(async () => {
        mainStore.setMainReaperResourceDir(await mainService.getMainReaperResourceDir());
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
                    <SecondaryButton>Pick portable REAPER installation</SecondaryButton>
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
                    <SecondaryButton>Pick portable REAPER installation</SecondaryButton>
                    <SecondaryButton>Create new portable REAPER installation</SecondaryButton>
                </ButtonList>
            </>
        );
    }
}