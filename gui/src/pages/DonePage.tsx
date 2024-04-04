import {ButtonRow} from "../components/ButtonRow.tsx";
import {mainStore} from "../globals.ts";
import {FaSolidCircleCheck, FaSolidFaceSadTear, FaSolidRocket} from "solid-icons/fa";
import {Match, Show, Switch} from "solid-js";
import {showDialog} from "../components/GlobalDialog.tsx";
import {startReaperAndQuit, startReaperInstaller} from "../epics/done.ts";

export function DonePage() {
    return <div class="grow hero">
        <div class="hero-content text-center">
            <div class="max-w-md">
                <Show when={mainStore.state.installationStage.stage}>
                    {
                        stage => {
                            const s = stage();
                            switch (s.kind) {
                                case "Failed":
                                    return <>
                                        <FaSolidFaceSadTear class="inline text-error" size="96"/>
                                        <p class="py-6">
                                            Installation failed
                                        </p>
                                        <div role="alert"
                                             class="alert alert-error max-h-24 text-pre text-xs select-all overflow-y-hidden">
                                            <span>{s.display_msg}</span>
                                        </div>
                                        <p class="pt-3 text-xs">
                                            If above error message looks cryptic or doesn't seem to make sense,
                                            please report it to&nbsp;
                                            <a href="mailto:info@helgoboss.org" class="link"
                                               target="_blank">info@helgoboss.org</a> (Right click → Copy).
                                            Thanks!
                                        </p>
                                    </>;
                                case "Finished":
                                    return <>
                                        <FaSolidCircleCheck class="inline text-success" size="96"/>
                                        <p class="py-6">
                                            Installation successful
                                        </p>
                                        <Switch>
                                            <Match when={mainStore.state.manualReaperInstallPath}>
                                                {path =>
                                                    <>
                                                        <p class="pb-3">
                                                            ReaBoot was not able to install REAPER automatically. Please
                                                            install it manually and close ReaBoot when you are done.
                                                        </p>
                                                        <button class="btn btn-primary"
                                                                onClick={() => startReaperInstaller(path())}>
                                                            <FaSolidRocket/>
                                                            Launch REAPER installer!
                                                        </button>
                                                    </>
                                                }
                                            </Match>
                                            <Match when={true}>
                                                <button class="btn btn-primary"
                                                        onClick={() => startReaperAndQuit()}>
                                                    <FaSolidRocket/>
                                                    Launch REAPER and quit!
                                                </button>
                                            </Match>
                                        </Switch>
                                    </>;
                                default:
                                    return <></>;
                            }
                        }
                    }
                </Show>
                <ButtonRow>
                    <Show when={mainStore.state.installationReportHtml}>
                        <button class="btn btn-link" onClick={showInstallationReport}>
                            Show installation report
                        </button>
                    </Show>
                </ButtonRow>
            </div>
        </div>
    </div>;
}

async function showInstallationReport() {
    await showDialog<boolean>({
        content: <div class="prose prose-sm overflow-y-auto" innerHTML={mainStore.state.installationReportHtml}/>,
        buildButtons: (close) => {
            return <>
                <button class="btn" onClick={() => close(false)}>Close</button>
            </>;
        }
    });
}