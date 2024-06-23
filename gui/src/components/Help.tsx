import {JSX} from "solid-js";
import {Tooltip} from "@kobalte/core";

type Props = {
    help: string,
    children: JSX.Element,
}

export function Help(props: Props) {
    return <Tooltip.Root openDelay={500}>
        <Tooltip.Trigger class="tooltip__trigger">
            {props.children}
        </Tooltip.Trigger>
        <Tooltip.Portal>
            <Tooltip.Content class="tooltip__content">
                <Tooltip.Arrow/>
                <p>{props.help}</p>
            </Tooltip.Content>
        </Tooltip.Portal>
    </Tooltip.Root>;
}