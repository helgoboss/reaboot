import {MainButton} from "../components/MainButton.tsx";
import {StepperTask, TaskStatus, TaskStepper} from "../components/TaskStepper.tsx";
import {from} from "solid-js";
import {mainService, mainStore} from "../globals.ts";
import {InstallationStatusEvent} from "../../src-lib/bindings/InstallationStatusEvent.ts";

export function InstallPanel() {
    const installationEvents = mainService.getInstallationEvents();
    const installationStatus = from(installationEvents.statusEvents);
    const installationStatusProgress = from(installationEvents.statusProgress);
    const effectiveInstallationStatus = () => installationStatus() ?? {kind: "Idle"};
    const effectiveInstallationStatusProgress = () => installationStatusProgress() ?? 0.0;
    const stepperTasks = () => deriveStepperTasks(effectiveInstallationStatus());
    const progressInPercent = () => `${(effectiveInstallationStatusProgress() * 100).toFixed(2)}`
    return (
        <>
            <div>
                <div>
                    REAPER resource directory:
                </div>
                <div class="font-mono">{mainStore.state.chosenReaperResourceDir}</div>
            </div>
            <div class="flex flex-row gap-10">
                <TaskStepper tasks={stepperTasks()}/>
                <div class="grow flex flex-row items-center gap-5 bg-amber-100">
                    <div class="flex-1">
                        {effectiveInstallationStatus().kind}
                    </div>
                    <div class="flex-1">
                        {progressInPercent()}%
                    </div>
                </div>
            </div>
            <MainButton onClick={() => mainService.startInstallation({})}>
                Start installation
            </MainButton>
        </>
    );
}

function deriveStepperTasks(status: InstallationStatusEvent): StepperTask[] {
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

function getTaskPos(status: InstallationStatusEvent) {
    switch (status.kind) {
        case "Idle":
            return INITIAL_POS;
        case "DownloadingReaper":
            return INSTALL_REAPER_POS;
        case "DownloadingReaPack":
        case "InitializingReaPack":
            return INSTALL_REAPACK_POS;
        case "DownloadingRepositoryIndex":
        case "DownloadingPackageFile":
        case "InstallingPackage":
            return INSTALL_PACKAGES_POS;
        case "Done":
            return DONE_POS;
    }
}

const INITIAL_POS = 0;
const INSTALL_REAPER_POS = 1;
const INSTALL_REAPACK_POS = 2;
const INSTALL_PACKAGES_POS = 3;
const DONE_POS = 4;