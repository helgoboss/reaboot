import {Component, JSX} from "solid-js";

export type MainButtonProps = {
    onClick?: (event: MouseEvent) => void,
    children: JSX.Element,
}

export const MainButton: Component<MainButtonProps> = (props) => {
    return <button class="w-96 text-4xl bg-slate-100"
                   onClick={props.onClick}>
        {props.children}
    </button>;
};