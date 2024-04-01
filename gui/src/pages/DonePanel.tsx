import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {mainStore} from "../globals.ts";
import {SolidMarkdown} from "solid-markdown";
import {InstallationStage} from "../../../core/bindings/InstallationStage.ts";
import {FaSolidCircleCheck, FaSolidRocket} from "solid-icons/fa";

export function DonePanel() {
    let modal;
    return (
        <div class="grow hero">
            <div class="hero-content text-center">
                <div class="max-w-md">
                    <FaSolidCircleCheck class="inline" size="96" color="green"/>
                    <p class="py-6">
                        Installation successful
                    </p>
                    <button class="btn btn-primary"
                            onClick={() => mainStore.currentPageId = "pick-reaper"}>
                        <FaSolidRocket/>
                        Launch REAPER!
                    </button>
                    <ButtonRow>
                        <button class="btn btn-link" onClick={() => modal!.showModal()}>
                            Show installation report
                        </button>
                    </ButtonRow>
                </div>
            </div>
            <dialog ref={modal} class="modal">
                <div class="modal-box">
                    <SolidMarkdown class="prose prose-sm">
                        {mainStore.state.installationReportMarkdown}
                    </SolidMarkdown>
                    <div class="modal-action">
                        <form method="dialog">
                            <button class="btn">Close</button>
                        </form>
                    </div>
                </div>
            </dialog>
        </div>
    );
}

function buildPageContent(stage: InstallationStage, report: string | undefined) {
    if (stage.kind === "Failed") {
        return (
            <>
                <div class="grow flex flex-col py-4">
                    Error: {stage.display_msg}
                </div>
                <ButtonRow>
                    <NavButton>Try again</NavButton>
                    <NavButton>Close</NavButton>
                </ButtonRow>
            </>
        );
    } else {
        return <>
            <div class="grow flex flex-col p-4 card bg-base-300">
                <SolidMarkdown class="prose prose-sm" children={report}/>
            </div>
            <ButtonRow>
                <NavButton>Close</NavButton>
                <NavButton>Close & Launch REAPER</NavButton>
            </ButtonRow>
        </>
    }
}