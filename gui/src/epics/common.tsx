import {PageId} from "../model/page.ts";
import {mainService, mainStore, pages} from "../globals.ts";
import {showDialog} from "../components/GlobalDialog.tsx";
import {createResource, Match, Switch} from "solid-js";
import {Toast, toaster} from "@kobalte/core";
import {FaSolidX} from "solid-icons/fa";

export function showError(message: any) {
    showToast("alert-error", message);
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
    if (mainStore.state.resolvedConfig?.reaper_exe_exists) {
        // If REAPER is already installed, we are not going to install REAPER from scratch, so we don't need
        // to let the user confirm the EULA again.
        return true;
    }
    if (mainStore.state.agreedToReaperEula) {
        // User agreed to EULA within this ReaBoot session
        return true;
    }
    if (await confirmReaperEula()) {
        mainStore.agreeToEula();
        return true;
    } else {
        return false;
    }
}