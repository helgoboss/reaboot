import {JSX} from "solid-js";

type Props = {
    class?: string,
    children: JSX.Element,
}

export function ButtonRow(props: Props) {
    return <div class={`grid grid-flow-col justify-center gap-8 p-4 ${props.class}`}>
        {props.children}
    </div>;
}