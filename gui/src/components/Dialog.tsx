import {JSX} from "solid-js";

export type DialogProps = {
    content: JSX.Element,
    buttons: JSX.Element,
    fullScreen: boolean,
}

export function Dialog(props: DialogProps) {
    return <dialog open={true} class="modal">
        <div
            class={`modal-box flex flex-col min-h-0 ${props.fullScreen ? "h-screen w-screen max-h-full max-w-full" : ""}`}>
            <div class="grow flex flex-col min-h-0">
                {props.content}
            </div>
            <div class="flex-none modal-action">
                <form method="dialog" class="flex flex-row gap-3">
                    {props.buttons}
                </form>
            </div>
        </div>
    </dialog>;
}