import {JSX} from "solid-js";
import {AlertDialog} from "@kobalte/core";

export type DialogProps = {
    title: string,
    content: JSX.Element,
    buttons: JSX.Element,
    fullScreen: boolean,
}

export function Dialog(props: DialogProps) {
    return <AlertDialog.Root open={true}>
        <AlertDialog.Portal>
            <AlertDialog.Overlay class="fixed inset-0 z-50 backdrop-brightness-50"/>
            <div class="fixed inset-0 z-50 flex flex-col items-center justify-center p-5">
                <AlertDialog.Content
                    class={`z-50 flex flex-col bg-base-100 p-5 gap-5 ${props.fullScreen ? "w-screen h-screen" : "modal-box"}`}>
                    <AlertDialog.Title class="text-xl font-bold text-center">{props.title}</AlertDialog.Title>
                    {/*<AlertDialog.CloseButton class="alert-dialog__close-button">*/}
                    {/*    <FaSolidX/>*/}
                    {/*</AlertDialog.CloseButton>*/}
                    <AlertDialog.Description class="flex flex-col min-h-0">
                        <div class="grow flex flex-col min-h-0">
                            {props.content}
                        </div>
                        <div class="flex-none modal-action justify-center">
                            <form method="dialog" class="flex flex-row gap-3">
                                {props.buttons}
                            </form>
                        </div>
                    </AlertDialog.Description>
                </AlertDialog.Content>
            </div>
        </AlertDialog.Portal>
    </AlertDialog.Root>;
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