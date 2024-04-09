import {createSignal, JSX} from "solid-js";
import {Dialog, DialogContentProps, DialogProps} from "./Dialog.tsx";

const [globalDialogContentProps, setGlobalDialogContentProps] = createSignal<DialogContentProps | undefined>(undefined);

export type ShowDialogArgs<T> = {
    title: string,
    content: JSX.Element,
    fullScreen?: boolean,
    buildButtons: (close: (value: T | undefined) => void) => JSX.Element,
}

export function showDialog<T>(args: ShowDialogArgs<T>): Promise<T | undefined> {
    return new Promise(resolve => {
        function close(value: T | undefined) {
            resolve(value);
            setGlobalDialogContentProps(undefined);
        }

        const dialogProps: DialogContentProps = {
            title: args.title,
            body: args.content,
            fullScreen: args.fullScreen ?? false,
            buttons: args.buildButtons(close),
        };
        setGlobalDialogContentProps(dialogProps);
    });
}

export function GlobalDialog() {
    const props = (): DialogProps => {
        const content = globalDialogContentProps();
        if (content) {
            return {
                open: true,
                setOpen: (value) => {
                    if (!value) {
                        setGlobalDialogContentProps(undefined);
                    }
                },
                content,
            };
        } else {
            return {
                open: false,
                setOpen: (_) => {
                },
                content: {
                    title: "Not set",
                    body: "Not set",
                    buttons: [],
                    fullScreen: false,
                }
            }
        }
    };
    return <Dialog {...props()} />;
}