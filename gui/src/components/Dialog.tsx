import {JSX} from "solid-js";
import {AlertDialog} from "@kobalte/core";

export type DialogProps = {
    content: DialogContentProps,
    open: boolean,
    setOpen: (value: boolean) => void,
}

export type DialogContentProps = {
    title: string,
    body: JSX.Element,
    buttons: JSX.Element,
    fullScreen: boolean,
}

export function Dialog(props: DialogProps) {
    return <AlertDialog.Root open={props.open} onOpenChange={props.setOpen}>
        <AlertDialog.Portal>
            <AlertDialog.Overlay class="fixed inset-0 z-50 backdrop-brightness-50"/>
            <div class="fixed inset-0 z-50 flex flex-col items-center justify-center p-5">
                <AlertDialog.Content
                    class={`z-50 flex flex-col bg-base-100 p-5 ${props.content.fullScreen ? "w-screen h-screen" : "modal-box"}`}>
                    <AlertDialog.Title class="text-xl font-bold text-center">{props.content.title}</AlertDialog.Title>
                    <AlertDialog.Description class="flex flex-col min-h-0 mt-5">
                        <div class="grow flex flex-col min-h-0">
                            {props.content.body}
                        </div>
                        <div class="flex-none modal-action justify-center">
                            <form method="dialog" class="grid grid-flow-col gap-3">
                                {props.content.buttons}
                            </form>
                        </div>
                    </AlertDialog.Description>
                </AlertDialog.Content>
            </div>
        </AlertDialog.Portal>
    </AlertDialog.Root>;
}