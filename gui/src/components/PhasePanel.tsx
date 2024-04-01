import {
    FaRegularCircle,
    FaRegularCircleCheck,
    FaRegularSquare,
    FaRegularSquareCheck,
    FaSolidCheck,
    FaSolidSquare,
    FaSolidSquareCheck
} from "solid-icons/fa";
import {Match, Switch} from "solid-js";
import {BsCircle} from "solid-icons/bs";

export type Phase = { label: string, status: PhaseStatus };

export type PhaseStatus = "todo" | "in-progress" | "done";

export function PhasePanel(props: Phase) {
    // TODO In dark mode, it looks better if the background is base and the foreground is special
    // window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', event => {
    //     const newColorScheme = event.matches ? "dark" : "light";
    // });
    const colorClasses = () => {
        switch (props.status) {
            case "todo":
                return "bg-info text-info-content";
            case "in-progress":
                return "bg-info text-info-content";
            case "done":
                return "bg-success text-success-content";
        }
    };
    return <div class={`grow card flex flex-row items-center px-6 ${colorClasses()}`}>
        <div class="grow">
            {props.label}
        </div>
        <div>
            <Switch>
                <Match when={props.status === "todo"}>
                    <FaRegularCircle size={24}/>
                </Match>
                <Match when={props.status === "done"}>
                    <FaRegularCircleCheck size={24}/>
                </Match>
                <Match when={props.status === "in-progress"}>
                    <span class="loading loading-dots loading-sm"></span>
                </Match>
            </Switch>
        </div>
    </div>;
}