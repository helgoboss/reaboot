import {JSX} from "solid-js";

type Props = {
    index: number,
    title: string,
    children: JSX.Element,
}

export function Step(props: Props) {
    return <div class="flex flex-col justify-center relative">
        <div class="absolute -left-16">
            <div
                class="w-9 h-9 rounded-full flex items-center justify-center bg-base-200 text-base-content">
                {props.index + 1}
            </div>
        </div>
        <div class="card card-compact bg-base-200">
            <div class="card-body text-center items-stretch">
                <h2 class="card-title self-center">{props.title}</h2>
                <div class="flex flex-col gap-3">
                    {props.children}
                </div>
            </div>
        </div>
    </div>
}