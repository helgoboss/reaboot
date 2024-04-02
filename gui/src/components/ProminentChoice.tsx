import {JSX} from "solid-js";

type Props = {
    selected: boolean,
    topRightIndicator?: JSX.Element,
    bottomRightIndicator?: JSX.Element,
    icon?: JSX.Element,
    onClick?: (event: MouseEvent) => void,
    children: JSX.Element,
}

export function ProminentChoice(props: Props) {
    return <div class="card card-compact w-2/3 min-h-28 bg-base-200 cursor-pointer indicator"
                onClick={props.onClick}>
        {props.topRightIndicator &&
            <span class="indicator-item badge badge-neutral">{props.topRightIndicator}</span>
        }
        {props.bottomRightIndicator &&
            <span class="indicator-item indicator-bottom">{props.bottomRightIndicator}</span>
        }
        <div class="card-body flex flex-row items-center">
            <div class="flex-none px-4">
                <input type="radio" class="radio radio-warning" checked={props.selected}
                       onClick={e => e.preventDefault()}/>
            </div>
            <div class="grow px-4">
                {props.children}
            </div>
            <div class="flex-none px-4">
                {props.icon}
            </div>
        </div>
    </div>;
}