import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/nav-button.tsx";
import {Page} from "../components/Page.tsx";
import {Phase, PhasePanel, PhaseStatus} from "../components/PhasePanel.tsx";
import {mainService, mainStore} from "../globals.ts";
import {from, Index} from "solid-js";
import {InstallationStage} from "../../../core/bindings/InstallationStage.ts";

export function InstallPage() {
    const installationStatus = mainStore.state.installationStage;
    const installationStatusProgress = from(mainService.getProgressEvents());
    const effectiveInstallationStatusProgress = () => installationStatusProgress() ?? 0.0;
    const phases = () => derivePhases(installationStatus);
    const progressInPercent = () => effectiveInstallationStatusProgress() * 100;
    return (
        <Page>
            <div class="grow flex flex-row items-stretch gap-8">
                <div class="grow-2 flex flex-col gap-4">
                    <Index each={phases()}>
                        {
                            (phase) => <PhasePanel {...phase()}/>
                        }
                    </Index>
                </div>
                <div class="grow-3 card bg-base-300">
                    <div class="card-body text-center">
                        <h2>{installationStatus.kind}</h2>
                        <div>
                            <progress class="progress w-56" value={progressInPercent()} max="100"></progress>
                        </div>
                    </div>
                </div>
            </div>
            <ButtonRow>
                <NavButton class="btn-warning" onClick={() => mainService.startInstallation()}>
                    Start installation
                </NavButton>
            </ButtonRow>
        </Page>
    );
}


function derivePhases(stage: InstallationStage): Phase[] {
    const actualTaskPos = getTaskPos(stage);
    return [
        {
            label: "Install REAPER",
            status: getTaskStatus(actualTaskPos, INSTALL_REAPER_POS),
        },
        {
            label: "Install ReaPack",
            status: getTaskStatus(actualTaskPos, INSTALL_REAPACK_POS),
        },
        {
            label: "Install packages",
            status: getTaskStatus(actualTaskPos, INSTALL_PACKAGES_POS),
        },
    ];
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
        case "ExtractingReaper":
            return INSTALL_REAPER_POS;
        case "InstalledReaper":
            return INSTALLED_REAPER_POS;
        case "CheckingLatestReaPackVersion":
        case "DownloadingReaPack":
            return INSTALL_REAPACK_POS;
        case "InstalledReaPack":
            return INSTALLED_REAPACK_POS;
        case "PreparingTempDirectory":
        case "DownloadingRepositoryIndexes":
        case "ParsingRepositoryIndexes":
        case "PreparingPackageDownloading":
        case "DownloadingPackageFiles":
        case "UpdatingReaPackState":
        case "ApplyingReaPackState":
        case "ApplyingPackage":
            return INSTALL_PACKAGES_POS;
        case "Done":
            return DONE_POS;
    }
}

const NOTHING_INSTALLED_POS = 0;
const INSTALL_REAPER_POS = 1;
const INSTALLED_REAPER_POS = 2;
const INSTALL_REAPACK_POS = 3;
const INSTALLED_REAPACK_POS = 4;
const INSTALL_PACKAGES_POS = 5;
const DONE_POS = 4;