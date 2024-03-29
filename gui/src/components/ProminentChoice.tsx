import {JSX} from "solid-js";

type Props = {
    selected: boolean,
    children: JSX.Element,
}

export function ProminentChoice(props: Props) {
    return <div class="card card-compact w-2/3 min-h-28 bg-base-200 cursor-pointer">
        <div class="card-body flex flex-row items-center">
            <div class="flex-none px-4">
                <input type="radio" class="radio radio-warning" checked={props.selected}/>
            </div>
            <div class="grow px-4">
                {props.children}
            </div>
        </div>
    </div>;
};