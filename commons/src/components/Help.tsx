import {JSX} from "solid-js";
import {Tooltip} from "@kobalte/core";

type Props = {
    help?: string | null,
    placement?: "top" | "bottom" | "left" | "right",
    children: JSX.Element,
}

export function Help(props: Props) {
    if (!props.help) {
        return props.children;
    }
    return <Tooltip.Root openDelay={500} placement={props.placement}>
        <Tooltip.Trigger class="h-tooltip__trigger">
            {props.children}
        </Tooltip.Trigger>
        <Tooltip.Portal>
            <Tooltip.Content class="h-tooltip__content">
                <Tooltip.Arrow/>
                <p>{props.help}</p>
            </Tooltip.Content>
        </Tooltip.Portal>
    </Tooltip.Root>;
}