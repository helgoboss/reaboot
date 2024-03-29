import {ProminentChoice} from "../components/ProminentChoice.tsx";
import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/nav-button.tsx";
import {Page} from "../components/Page.tsx";
import {FaRegularFolderOpen} from "solid-icons/fa";
import {open} from "@tauri-apps/api/dialog";
import {mainService, mainStore} from "../globals.ts";

export function PickReaperPage() {
    return (
        <Page>
            <div class="grow flex flex-col items-center justify-center gap-4">
                <ProminentChoice selected={true}>
                    <h2 class="card-title">Main REAPER installation</h2>
                    <p class="text-neutral-content/50">Add packages to your existing main REAPER installation.</p>
                </ProminentChoice>
                <ProminentChoice selected={false}>
                    <div class="flex flex-row items-center gap-4">
                        <div class="flex flex-col">
                            <h2 class="card-title">Portable REAPER installation</h2>
                            <p class="text-neutral-content/50">
                                Add packages to an existing portable REAPER installation or
                                create a new one.
                            </p>
                        </div>
                        <button class="btn btn-circle">
                            <FaRegularFolderOpen size={24} onClick={pickPortableReaperInstallation}/>
                        </button>
                    </div>
                </ProminentChoice>
            </div>
            <ButtonRow>
                <NavButton>Continue</NavButton>
            </ButtonRow>
        </Page>
    );
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