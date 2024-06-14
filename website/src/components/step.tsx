import {JSX} from "solid-js";

type Props = {
    index: number,
    title: string,
    children: JSX.Element,
}

export function Step(props: Props) {
    return <div class="mt-6 flex flex-col items-stretch gap-3 md:block md:relative">
        <div class="self-center md:absolute md:-left-16 md:top-1/2 md:-translate-y-1/2">
            <div
                class="w-9 h-9 rounded-full flex items-center justify-center bg-base-200 text-base-content">
                {props.index + 1}
            </div>
        </div>
        <div class="card card-compact bg-base-200">
            <div class="card-body text-center items-stretch">
                <h2 class="card-title self-center">{props.title}</h2>
                <div class="flex flex-col">
                    {props.children}
                </div>
            </div>
        </div>
    </div>
}