import MainStore from "../store/main-store.ts";
import MainButton from "./MainButton.tsx";
import {ButtonList} from "./ButtonList.tsx";
import SecondaryButton from "./SecondaryButton.tsx";

export function PickReaperPanel() {
    const mainReaperResourcePath = MainStore.state.mainReaperResourcePath;
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