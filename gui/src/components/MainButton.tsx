import {Component, JSX} from "solid-js";

export type MainButtonProps = {
    onClick?: (event: MouseEvent) => void,
    children: JSX.Element,
}

export const MainButton: Component<MainButtonProps> = (props) => {
    return <button class="btn btn-primary"
                   onClick={props.onClick}>
        {props.children}
    </button>;
};