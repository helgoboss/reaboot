import {PageId} from "../model/page.ts";
import {mainService, mainStore, pages} from "../globals.ts";
import {showDialog} from "../components/GlobalDialog.tsx";
import {createResource, Match, Switch} from "solid-js";
import {Toast, toaster} from "@kobalte/core";
import {FaSolidX} from "solid-icons/fa";
import {ResolvedInstallerConfig} from "../../src-tauri/bindings/ResolvedInstallerConfig.ts";
import {configureInstaller} from "./install.ts";

export function showError(message: any) {
    showDialog<boolean>({
        title: "Error",
        content: <p class="text-center">{message}</p>,
        buildButtons: (close) => {
            return <>
                <button class="btn" onClick={() => close(false)}>Close</button>
            </>;
        }
    });
}

export function showInfo(message: any) {
    showToast("alert-info", message);
}

export function showSuccess(message: any) {
    showToast("alert-success", message);
}

export function showWarning(message: any) {
    showToast("alert-warning", message);
}

function showToast(clazz: string, message: string) {
    toaster.show(props => (
        <Toast.Root toastId={props.toastId} class={`alert ${clazz}`}>
            <div class="flex flex-row justify-between">
                <div>
                    <Toast.Description>
                        {message}
                    </Toast.Description>
                </div>
                <Toast.CloseButton class="ml-5">
                    <FaSolidX/>
                </Toast.CloseButton>
            </div>
        </Toast.Root>
    ));
}

async function confirmReaperInstall(resolvedConfig: ResolvedInstallerConfig): Promise<boolean> {
    const yes = await showDialog<boolean>({
        title: "Install REAPER?",
        fullScreen: false,
        content: <div class="text-center">
            <p>
                We found REAPER configuration files on your disk, but we couldn't detect an existing installation at:
            </p>
            <p class="mt-2 font-mono">{resolvedConfig.reaper_exe}</p>
            <p class="mt-2">
                Would you like to install REAPER at this default location?
            </p>
        </div>,
        buildButtons: (close) => {
            return <>
                <button class="btn" onClick={() => close(true)}>Yes, please install it!</button>
                <button class="btn btn-primary" onClick={() => close(false)}>
                    No, it's already installed!
                </button>
            </>;
        }
    });
    return yes || false
}

async function confirmReaperEula(): Promise<boolean> {
    const [eulaResource] = createResource(mainService.getReaperEula);
    const yes = await showDialog<boolean>({
        title: "REAPER license agreement",
        fullScreen: true,
        content: <div class="flex flex-col min-h-0">
            <div class="text-center">
                ReaBoot is going to download and install REAPER because it's not yet installed at the location of
                your choice. In order to continue, you need to accept the REAPER license agreement.
            </div>
            <div class="grow card min-h-0 bg-base-300 mt-4">
                <div class="card-body min-h-0">
                    <Switch>
                        <Match when={eulaResource.loading}>
                            <span class="loading loading-ball loading-md"/>
                        </Match>
                        <Match when={true}>
                            <div class="overflow-y-auto whitespace-pre text-xs select-all">
                                {eulaResource()}
                            </div>
                        </Match>
                    </Switch>
                </div>
            </div>
        </div>,
        buildButtons: (close) => {
            return <>
                <button class="btn btn-primary" onClick={() => close(true)}>Agree</button>
                <button class="btn" onClick={() => close(false)}>Disagree</button>
            </>;
        }
    });
    return yes || false
}

export async function navigateTo(pageId: PageId) {
    // Check if a recipe has been selected
    if (!mainStore.state.parsedRecipe) {
        showToast("alert-warning", "You can continue as soon as you give ReaBoot an installation recipe!");
        return;
    }
    // Check on a page-by-page basis
    switch (pageId) {
        case "customize":
            // Check if customize page should be skipped
            if (!await ensureUserAgreedToEula()) {
                return;
            }
            if (pageId == "customize" && !mainStore.shouldShowCustomizePage) {
                mainStore.setCurrentPageId("install");
                return;
            }
            break;
        case "install":
            // Check feature requirements
            if (!await ensureUserAgreedToEula()) {
                return;
            }
            if (mainStore.shouldShowCustomizePage && !mainStore.featureConfigIsValid) {
                showToast("alert-warning", "Please select at least one feature!");
                mainStore.setCurrentPageId("customize");
                return;
            }
            break;
        case "done":
            const kind = mainStore.state.installationStage.stage.kind;
            if (kind !== "Finished" && kind !== "Failed") {
                return;
            }
            break;
    }
    // Finally change page
    mainStore.setCurrentPageId(pageId);
}

export function getPage(pageId: PageId) {
    return pages.find((p) => p.id == pageId)!;
}

async function ensureUserAgreedToEula(): Promise<boolean> {
    const resolvedConfig = mainStore.state.resolvedConfig;
    if (!resolvedConfig) {
        return false;
    }
    if (resolvedConfig.reaper_exe_exists) {
        // If REAPER is already installed, we are not going to install REAPER from scratch, so we don't need
        // to let the user confirm the EULA again.
        return true;
    }
    if (!resolvedConfig.install_reaper) {
        // If REAPER has opted out from installing REAPER, there's also no need to get an agreement.
        return true;
    }
    if (!resolvedConfig.portable && resolvedConfig.reaper_ini_exists
        && mainStore.state.installerConfig.install_reaper === undefined) {
        // ReaBoot detected an existing main REAPER resource path but no main binaries, at least not at their default
        // location. Ask user if he wants to install REAPER.
        const installReaper = await confirmReaperInstall(resolvedConfig);
        await configureInstaller({
            installReaper
        });
        if (!installReaper) {
            // User opted out from REAPER installation
            return true;
        }
    }
    if (mainStore.state.agreedToReaperEula) {
        // User agreed to EULA within this ReaBoot session
        return true;
    }
    // User has not opted out from REAPER install and not confirmed the EULA yet. Display dialog.
    if (await confirmReaperEula()) {
        mainStore.agreeToEula();
        return true;
    } else {
        return false;
    }
}