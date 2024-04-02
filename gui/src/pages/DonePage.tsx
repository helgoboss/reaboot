import {ButtonRow} from "../components/ButtonRow.tsx";
import {mainStore} from "../globals.ts";
import {FaSolidCircleCheck, FaSolidRocket, FaSolidX} from "solid-icons/fa";
import {Match, Show, Switch} from "solid-js";
import {startReaperAndQuit} from "../epics/done.ts";

export function DonePage() {
    let modal;
    return (
        <div class="grow hero">
            <div class="hero-content text-center">
                <div class="max-w-md">
                    <Switch>
                        <Match when={mainStore.state.installationStage.stage.kind === "Finished"}>
                            <FaSolidCircleCheck class="inline text-success" size="96"/>
                            <p class="py-6">
                                Installation successful
                            </p>
                        </Match>
                        <Match when={mainStore.state.installationStage.stage.kind === "Failed"}>
                            <FaSolidX class="inline text-error" size="96"/>
                            <p class="py-6">
                                Installation failed
                            </p>
                        </Match>
                    </Switch>
                    <button class="btn btn-primary"
                            onClick={() => startReaperAndQuit()}>
                        <FaSolidRocket/>
                        Launch REAPER!
                    </button>
                    <ButtonRow>
                        <Show when={mainStore.state.installationReportHtml}>
                            <button class="btn btn-link" onClick={() => modal!.showModal()}>
                                Show installation report
                            </button>
                        </Show>
                    </ButtonRow>
                </div>
            </div>
            <dialog ref={modal} class="modal">
                <div class="modal-box flex flex-col">
                    <div class="grow overflow-y-auto">
                        <div class="prose prose-sm" innerHTML={mainStore.state.installationReportHtml}>
                        </div>
                    </div>
                    <div class="flex-none modal-action">
                        <form method="dialog">
                            <button class="btn">Close</button>
                        </form>
                    </div>
                </div>
            </dialog>
        </div>
    );
}
