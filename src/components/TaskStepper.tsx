import { Index, Match, Switch } from "solid-js";

export type TaskStepperProps = {
    tasks: StepperTask[]
}

export type StepperTask = {
    id: string,
    title: string,
    status: TaskStatus,
}

export type TaskStatus = "todo" | "in-progress" | "done";

export function TaskStepper(props: TaskStepperProps) {
    return (
        <ol class="space-y-4 w-72">
            <Index each={props.tasks}>
                {
                    (task, index) => (
                        <li>
                            <Switch>
                                <Match when={task().status === "todo"}>
                                    <div class="w-full p-4 text-gray-900 bg-gray-100 border border-gray-300 rounded-lg dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400" role="alert">
                                        <div class="flex items-center justify-between">
                                            <h3 class="font-medium">{index + 1}. {task().title}</h3>
                                        </div>
                                    </div>
                                </Match>
                                <Match when={task().status === "in-progress"}>
                                    <div class="w-full p-4 text-blue-700 bg-blue-100 border border-blue-300 rounded-lg dark:bg-gray-800 dark:border-blue-800 dark:text-blue-400" role="alert">
                                        <div class="flex items-center justify-between">
                                            <h3 class="font-medium">{index + 1}. {task().title}</h3>
                                            <svg class="rtl:rotate-180 w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 10">
                                                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M1 5h12m0 0L9 1m4 4L9 9" />
                                            </svg>
                                        </div>
                                    </div>
                                </Match>
                                <Match when={task().status === "done"}>
                                    <div class="w-full p-4 text-green-700 border border-green-300 rounded-lg bg-green-50 dark:bg-gray-800 dark:border-green-800 dark:text-green-400" role="alert">
                                        <div class="flex items-center justify-between">
                                            <h3 class="font-medium">{index + 1}. {task().title}</h3>
                                            <svg class="w-4 h-4" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 16 12">
                                                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M1 5.917 5.724 10.5 15 1.5" />
                                            </svg>
                                        </div>
                                    </div>
                                </Match>
                            </Switch>
                        </li>
                    )
                }
            </Index>
        </ol>
    );
}