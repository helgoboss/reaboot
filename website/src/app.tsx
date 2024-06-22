import {Toast} from '@kobalte/core';
import {RouteSectionProps} from '@solidjs/router';
import {Portal} from "solid-js/web";


export function App(props: RouteSectionProps) {
    return <>
        <div>
            {props.children}
        </div>
        <Portal>
            <Toast.Region>
                <Toast.List class="toast toast-bottom toast-center z-50"/>
            </Toast.Region>
        </Portal>
    </>;
}