import {MainButton} from "../components/MainButton.tsx";
import {StepperTask, TaskStatus, TaskStepper} from "../components/TaskStepper.tsx";
import {from} from "solid-js";
import {mainService, mainStore} from "../globals.ts";
import {InstallationStatus} from "../../src-lib/bindings/InstallationStatus.ts";
import {InstallationStatusPanel} from "../components/InstallationStatusPanel.tsx";

export function InstallPanel() {
    const installationStatus = mainStore.state.installationStatus;
    const installationStatusProgress = from(mainService.getProgressEvents());
    const effectiveInstallationStatusProgress = () => installationStatusProgress() ?? 0.0;
    const stepperTasks = () => deriveStepperTasks(installationStatus);
    const progressInPercent = () => `${(effectiveInstallationStatusProgress() * 100).toFixed(2)}`
    return (
        <>
            <div>
                <div>
                    REAPER resource directory:
                </div>
                <div class="font-mono">{mainStore.state.resolvedConfig?.reaper_resource_dir}</div>
            </div>
            <div class="flex flex-row gap-10">
                <TaskStepper tasks={stepperTasks()}/>
                <div class="grow flex flex-row items-center gap-5 bg-amber-100">
                    <div class="flex-1">
                        <InstallationStatusPanel status={installationStatus}/>
                    </div>
                    <div class="flex-1">
                        {progressInPercent()}%
                    </div>
                </div>
            </div>
            <MainButton onClick={() => mainService.startInstallation()}>
                Start installation
            </MainButton>
        </>
    );
}

function deriveStepperTasks(status: InstallationStatus): StepperTask[] {
    const actualTaskPos = getTaskPos(status);
    return [
        {
            status: getTaskStatus(actualTaskPos, INSTALL_REAPER_POS),
            title: "Install REAPER"
        },
        {
            status: getTaskStatus(actualTaskPos, INSTALL_REAPACK_POS),
            title: "Install ReaPack"
        },
        {
            status: getTaskStatus(actualTaskPos, INSTALL_PACKAGES_POS),
            title: "Install packages"
        },
    ];
}

function getTaskStatus(actualTaskPos: number, expectedTaskPos: number): TaskStatus {
    if (actualTaskPos < expectedTaskPos) {
        return "todo";
    }
    if (actualTaskPos === expectedTaskPos) {
        return "in-progress";
    }
    return "done";
}

function getTaskPos(status: InstallationStatus) {
    switch (status.kind) {
        case "Initial":
            return INITIAL_POS;
        case "CheckingLatestReaperVersion":
        case "DownloadingReaper":
            return INSTALL_REAPER_POS;
        case "InstalledReaper":
            return INSTALLED_REAPER_POS;
        case "CheckingLatestReaPackVersion":
        case "DownloadingReaPack":
            return INSTALL_REAPACK_POS;
        case "InstalledReaPack":
            return INSTALLED_REAPACK_POS;
        case "DownloadingRepositoryIndexes":
        case "ParsingRepositoryIndexes":
        case "PreparingPackageInstallation":
        case "DownloadingPackageFiles":
        case "InstallingPackage":
            return INSTALL_PACKAGES_POS;
        case "Done":
            return DONE_POS;
    }
}

const INITIAL_POS = 0;
const INSTALL_REAPER_POS = 1;
const INSTALLED_REAPER_POS = 2;
const INSTALL_REAPACK_POS = 3;
const INSTALLED_REAPACK_POS = 4;
const INSTALL_PACKAGES_POS = 5;
const DONE_POS = 4;