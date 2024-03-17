import MainButton from "./MainButton.tsx";
import { StepperTask, TaskStepper } from "./TaskStepper.tsx";

export function InstallPanel() {
    return (
        <>
            <TaskStepper tasks={tasks}/>
            <MainButton>
                Start installation
            </MainButton>
        </>
    );
}

const tasks: StepperTask[] = [
    {
        id: "install-reaper",
        status: "done",
        title: "Install REAPER"
    },
    {
        id: "install-reapack",
        status: "in-progress",
        title: "Install ReaPack"
    },
    {
        id: "install-packages",
        status: "todo",
        title: "Install packages"
    },
]