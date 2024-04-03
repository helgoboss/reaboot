import {createSignal, JSX, Show} from "solid-js";
import {Dialog, DialogProps} from "./Dialog.tsx";

const [globalDialogProps, setGlobalDialogProps] = createSignal<DialogProps | undefined>(undefined);

export type ShowDialogArgs<T> = {
    content: JSX.Element,
    fullScreen?: boolean,
    buildButtons: (close: (value: T) => void) => JSX.Element,
}

export function showDialog<T>(args: ShowDialogArgs<T>): Promise<T> {
    return new Promise(resolve => {
        function close(value: T) {
            resolve(value);
            setGlobalDialogProps(undefined);
        }

        const dialogProps: DialogProps = {
            content: args.content,
            fullScreen: args.fullScreen ?? false,
            buttons: args.buildButtons(close),
        };
        setGlobalDialogProps(dialogProps);
    });
}

export function GlobalDialog() {
    return <Show when={globalDialogProps()}>
        {props => <Dialog {...props()} />}
    </Show>;
}