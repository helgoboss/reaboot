import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {Page} from "../components/Page.tsx";
import {Phase, PhasePanel, PhaseStatus} from "../components/PhasePanel.tsx";
import {mainService, mainStore, themeSignal} from "../globals.ts";
import {createMemo, For, from, Index, Match, Show, Switch} from "solid-js";
import {InstallationStage} from "../../../core/bindings/InstallationStage.ts";
import {WaitingForDataPage} from "./WaitingForDataPage.tsx";
import {PackageTable} from "../components/PackageTable.tsx";
import {ResolvedInstallerConfig} from "../../src-tauri/bindings/ResolvedInstallerConfig.ts";
import {install} from "../epics/install.ts";

export function InstallPage() {
    const resolvedConfig = mainStore.state.resolvedConfig;
    if (!resolvedConfig) {
        return <WaitingForDataPage/>;
    }
    const installationStageContainer = mainStore.state.installationStage;
    const mainProgress = from(mainService.getProgressEvents());
    const effectiveInstallationStatusProgress = () => mainProgress() ?? 0.0;
    const phases = () => derivePhases(installationStageContainer.stage, resolvedConfig);
    const mainProgressInPercent = () => effectiveInstallationStatusProgress() * 100;
    const installButtonProps = createMemo(() => getInstallButtonProps(installationStageContainer.stage));
    return (
        <Page>
            <p class="text-center font-bold pb-6">
                {installButtonProps().title}
            </p>
            <div class="grow flex flex-row items-stretch min-h-0">
                <div class="basis-1/3 grid grid-flow-row gap-4">
                    <Index each={phases()}>
                        {
                            (phase) => <PhasePanel {...phase()} darkMode={themeSignal() == "dark"}/>
                        }
                    </Index>
                </div>
                <div class="ml-8 basis-2/3 card bg-base-300">
                    <div class="card-body min-h-0">
                        <Switch>
                            <Match when={mainStore.installationIsRunning}>
                                <h2 class="text-center">{installationStageContainer.label}</h2>
                                <div>
                                    <progress class="progress" value={mainProgressInPercent()} max="100"/>
                                </div>
                                <Show when={mainStore.state.current_tasks.length > 0}>
                                    <table class="table table-xs table-fixed">
                                        <tbody>
                                        <For each={mainStore.state.current_tasks}>
                                            {task =>
                                                <tr class="border-none">
                                                    <td class="w-1/2 whitespace-nowrap overflow-hidden p-0">
                                                        {task.label}
                                                    </td>
                                                    <td class="w-1/2 pl-4 pr-0">
                                                        <progress class="progress" value={task.progress * 100}
                                                                  max="100"/>
                                                    </td>
                                                </tr>
                                            }
                                        </For>
                                        </tbody>
                                    </table>
                                </Show>
                            </Match>
                            <Match when={true}>
                                <div class="overflow-y-auto">
                                    <div class="prose prose-sm">
                                        <h4>General</h4>
                                        <ul>
                                            <li>
                                                <b>Destination:</b> REAPER {resolvedConfig.portable ? "portable" : "main"} installation
                                            </li>
                                            <li><b>Platform:</b> <span
                                                class="font-mono">{resolvedConfig.platform}</span></li>
                                            <li>
                                                <b>Error
                                                    handling:</b> {resolvedConfig.skip_failed_packages ? "Ignoring failing packages" : "Prevent incomplete installations"}
                                            </li>
                                        </ul>
                                        <h4>Packages</h4>
                                    </div>
                                    <PackageTable packages={resolvedConfig.package_urls}/>
                                </div>
                            </Match>
                        </Switch>
                    </div>
                </div>
            </div>
            <ButtonRow>
                <NavButton class={installButtonProps().buttonClass} onClick={() => install()}
                           disabled={mainStore.installationIsRunning}>
                    {
                        installButtonProps().buttonLabel
                    }
                </NavButton>
            </ButtonRow>
        </Page>
    );
}

type DisplayProps = {
    title: string,
    buttonLabel: string,
    buttonClass: string,
}

function getInstallButtonProps(stage: InstallationStage): DisplayProps {
    switch (stage.kind) {
        case "NothingInstalled":
        case "InstalledReaper":
            return {
                buttonLabel: "Start installation",
                buttonClass: "btn-warning",
                title: "Please review your choices and start the installation!"
            };
        case "Failed":
            return {
                buttonLabel: "Retry installation",
                buttonClass: "btn-error",
                title: "Installation has failed. Want to try again?"
            };
        case "Finished":
            return {
                buttonLabel: "Restart installation",
                buttonClass: "btn-success",
                title: "Installation succeeded. But you can do it again, if you want."
            };
        default:
            return {buttonLabel: "Installing...", buttonClass: "btn-warning", title: "Installation in progress"};
    }
}

function derivePhases(stage: InstallationStage, config: ResolvedInstallerConfig): Omit<Phase, "darkMode">[] {
    const actualTaskPos = getTaskPos(stage);
    const phases = [
        {
            index: 0,
            todoLabel: "Prepare REAPER",
            inProgressLabel: "Preparing REAPER",
            doneLabel: "REAPER is prepared",
            unnecessaryLabel: "REAPER is installed",
            status: stage.kind === "InstalledReaper" ? "unnecessary" : getTaskStatus(actualTaskPos, PREPARE_REAPER_POS),
        },
    ];
    if (config.package_urls.length > 0) {
        phases.push({
            index: 1,
            todoLabel: "Prepare packages",
            inProgressLabel: "Preparing packages",
            doneLabel: "Packages are prepared",
            unnecessaryLabel: "Packages are installed",
            status: getTaskStatus(actualTaskPos, PREPARE_PACKAGES_POS),
        });
    }
    phases.push({
        index: 2,
        todoLabel: "Install everything",
        inProgressLabel: "Installing everything",
        doneLabel: "Everything is installed",
        unnecessaryLabel: "Everything is installed",
        status: getTaskStatus(actualTaskPos, INSTALL_EVERYTHING_POS),
    });
    return phases;
}

function getTaskStatus(actualTaskPos: number, expectedTaskPos: number): PhaseStatus {
    if (actualTaskPos < expectedTaskPos) {
        return "todo";
    }
    if (actualTaskPos === expectedTaskPos) {
        return "in-progress";
    }
    return "done";
}

function getTaskPos(stage: InstallationStage) {
    switch (stage.kind) {
        case "NothingInstalled":
            return NOTHING_INSTALLED_POS;
        case "CheckingLatestReaperVersion":
        case "DownloadingReaper":
        case "PreparingReaper":
            return PREPARE_REAPER_POS;
        case "InstalledReaper":
            return INSTALLED_REAPER_POS;
        case "PreparingTempDirectory":
        case "DownloadingRepositoryIndexes":
        case "ParsingRepositoryIndexes":
        case "PreparingPackageDownloading":
        case "DownloadingPackageFiles":
        case "PreparingReaPackState":
            return PREPARE_PACKAGES_POS;
        case "ApplyingReaPackState":
        case "InstallingPackage":
        case "InstallingReaper":
            return INSTALL_EVERYTHING_POS;
        case "Finished":
            return FINISHED_POS;
        case "Failed":
            return FAILED_POS;
    }
}

const FAILED_POS = 0;
const NOTHING_INSTALLED_POS = 1;
const PREPARE_REAPER_POS = 2;
const INSTALLED_REAPER_POS = 3;
const PREPARE_PACKAGES_POS = 4;
const INSTALL_EVERYTHING_POS = 5;
const FINISHED_POS = 6;