import {MainButton} from "../components/MainButton.tsx";
import {ButtonList} from "../components/ButtonList.tsx";
import {SecondaryButton} from "../components/SecondaryButton.tsx";
import {mainService, mainStore} from "../globals.ts";
import {open} from '@tauri-apps/api/dialog';

export function PickReaperPanel() {
    const resolvedConfig = mainStore.state.resolvedConfig;
    if (!resolvedConfig) {
        return (
            <div>Waiting for data...</div>
        );
    }
    const resourceDirLabel = <div>
        <span
            class="font-mono">{resolvedConfig.reaper_resource_dir}
        </span>
    </div>;
    const pickPortableButton = () => {
        return <SecondaryButton onClick={pickPortableReaperInstallation}>
            Pick portable REAPER installation
        </SecondaryButton>;
    };
    const resetPortableButton = () => {
        return <SecondaryButton onClick={resetCustomReaperResourceDir}>
            Go back to main REAPER installation
        </SecondaryButton>;
    };
    const installReaperButton = () => {
        return <MainButton>Install REAPER</MainButton>;
    };
    const installationStatus = mainStore.state.installationStage;
    if (installationStatus.kind == "NothingInstalled") {
        // REAPER is not installed according to the current configuration
        if (resolvedConfig.portable) {
            // User must have picked a custom directory already, but it doesn't look like a portable REAPER installation.
            return <>
                <div>
                    The chosen directory doesn't look like a portable REAPER installation: {resourceDirLabel}.
                </div>
                <ButtonList>
                    {installReaperButton()}
                    {pickPortableButton()}
                    {resetPortableButton()}
                </ButtonList>
            </>;
        } else {
            // User hasn't picked a custom directory. Main installation doesn't seem to exist.
            return <>
                <div>
                    We didn't detect any existing REAPER main installation.
                </div>
                <ButtonList>
                    {installReaperButton()}
                    {pickPortableButton()}
                </ButtonList>
            </>;
        }
    } else {
        // REAPER is already installed according to the current configuration
        if (resolvedConfig.portable) {
            // User has picked a valid portable REAPER installation
            return <>
                <div>
                    You chose the following portable REAPER installation: {resourceDirLabel}.
                </div>
                <ButtonList>
                    <SecondaryButton onClick={pickPortableReaperInstallation}>
                        Pick another portable REAPER installation
                    </SecondaryButton>;
                    {installReaperButton()}
                    {resetPortableButton()}
                </ButtonList>
            </>;
        } else {
            // User hasn't picked a custom directory, and we have a valid main REAPER installation
            return <>
                <div>
                    We detected an existing main installation of REAPER with resource path: {resourceDirLabel}
                </div>
                <ButtonList>
                    <MainButton>Use main REAPER installation</MainButton>
                    {pickPortableButton()}
                </ButtonList>
            </>;
        }
    }
}

async function pickPortableReaperInstallation() {
    const chosenDir = await open({
        title: "Pick the root directory of your portable REAPER installation!",
        multiple: false,
        directory: true,
    });
    if (chosenDir == null || Array.isArray(chosenDir)) {
        return;
    }
    mainService.configure({
        custom_reaper_resource_dir: chosenDir,
        package_urls: mainStore.state.packageUrls,
    });
}

function resetCustomReaperResourceDir() {
    mainService.configure({
        custom_reaper_resource_dir: undefined,
        package_urls: mainStore.state.packageUrls,
    });
}