import {FaRegularCircleCheck} from "solid-icons/fa";
import {Match, Switch} from "solid-js";

export type Phase = {
    index: number,
    todoLabel: string,
    doneLabel: string,
    inProgressLabel: string,
    unnecessaryLabel: string,
    darkMode: boolean,
    status: PhaseStatus
};

export type PhaseStatus = "todo" | "in-progress" | "done" | "unnecessary";

export function PhasePanel(props: Phase) {
    const colorClasses = () => {
        switch (props.status) {
            case "todo":
                return props.darkMode ? "bg-base-300 text-base" : "bg-info text-info-base";
            case "in-progress":
                return props.darkMode ? "bg-base-300 text-warning" : "bg-info text-info-warning";
            case "done":
            case "unnecessary":
                return props.darkMode ? "bg-base-300 text-success" : "bg-success text-success-content";
        }
    };
    return <div class={`grow card flex flex-row items-center px-6 whitespace-nowrap ${colorClasses()}`}>
        <div class="grow">
            {props.index + 1}.&nbsp;
            <Switch>
                <Match when={props.status === "todo"}>
                    {props.todoLabel}
                </Match>
                <Match when={props.status === "in-progress"}>
                    {props.inProgressLabel}
                </Match>
                <Match when={props.status === "done"}>
                    {props.doneLabel}
                </Match>
                <Match when={props.status === "unnecessary"}>
                    {props.unnecessaryLabel}
                </Match>
            </Switch>
        </div>
        <div class="ml-3">
            <Switch>
                <Match when={props.status === "in-progress"}>
                    <span class="loading loading-ball loading-md"></span>
                </Match>
                <Match when={props.status === "done"}>
                    <FaRegularCircleCheck size={24}/>
                </Match>
            </Switch>
        </div>
    </div>;
}