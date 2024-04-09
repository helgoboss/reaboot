import {toast} from "solid-toast";
import {PageId} from "../model/page.ts";
import {mainService, mainStore, pages} from "../globals.ts";
import {showDialog} from "../components/GlobalDialog.tsx";
import {createResource, Match, Switch} from "solid-js";

export function showError(message: any) {
    toast.error(message);
}

export function showInfo(message: any) {
    toast.success(message);
}

export function showWarning(message: any) {
    toast.error(message);
}

export async function navigateTo(pageId: PageId) {
    const destPage = getPage(pageId);
    if (destPage.requiresReaperEulaAgreement
        && !mainStore.state.agreedToReaperEula
        && !mainStore.state.resolvedConfig?.reaper_exe_exists) {
        const [eulaResource] = createResource(mainService.getReaperEula);
        const userAgreedToEula = await showDialog<boolean>({
            title: "REAPER license agreement",
            fullScreen: true,
            content: <div class="flex flex-col min-h-0 gap-4">
                <div class="text-center">
                    ReaBoot is going to download and install REAPER because it's not yet installed at the location of
                    your choice. In order to continue, you need to accept the REAPER license agreement.
                </div>
                <div class="grow card min-h-0 bg-base-300">
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
        if (userAgreedToEula) {
            mainStore.agreeToEula();
        } else {
            // Don't change page = installation not possible.
            return;
        }
    }
    mainStore.setCurrentPageId(pageId);
}

export function getPage(pageId: PageId) {
    return pages.find((p) => p.id == pageId)!;
}