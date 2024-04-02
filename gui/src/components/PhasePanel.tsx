import {FaRegularCircle, FaRegularCircleCheck} from "solid-icons/fa";
import {Match, Switch} from "solid-js";

export type Phase = {
    index: number,
    todoLabel: string,
    doneLabel: string,
    inProgressLabel: string,
    darkMode: boolean,
    status: PhaseStatus
};

export type PhaseStatus = "todo" | "in-progress" | "done";

export function PhasePanel(props: Phase) {
    const colorClasses = () => {
        switch (props.status) {
            case "todo":
                return props.darkMode ? "bg-base-300 text-info" : "bg-info text-info-content";
            case "in-progress":
                return props.darkMode ? "bg-base-300 text-info" : "bg-info text-info-content";
            case "done":
                return props.darkMode ? "bg-base-300 text-success" : "bg-success text-success-content";
        }
    };
    return <div class={`grow card flex flex-row items-center px-6 ${colorClasses()}`}>
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
            </Switch>
        </div>
        <Switch>
            <Match when={props.status === "todo"}>
                <FaRegularCircle size={24}/>
            </Match>
            <Match when={props.status === "in-progress"}>
                <span class="loading loading-ball loading-md"></span>
            </Match>
            <Match when={props.status === "done"}>
                <FaRegularCircleCheck size={24}/>
            </Match>
        </Switch>
    </div>;
}