import {Component, JSX} from "solid-js";

type SecondaryButtonProps = {
    onClick?: (event: MouseEvent) => void,
    children: JSX.Element,
}

const SecondaryButton: Component<SecondaryButtonProps> = (props) => {
    return <button class="w-96 text-2xl bg-slate-50"
                   onClick={props.onClick}>
        {props.children}
    </button>;
};

export default SecondaryButton;