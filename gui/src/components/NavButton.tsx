import {JSX} from "solid-js";

type Props = {
    disabled?: boolean,
    class?: string,
    onClick?: (event: MouseEvent) => void,
    children: JSX.Element,
}

export function NavButton(props: Props) {
    return <button onClick={props.onClick} class={`btn grow btn-primary max-w-96 ${props.class}`}
                   disabled={props.disabled}>
        {props.children}
    </button>
}