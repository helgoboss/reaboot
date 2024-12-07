import {JSX} from "solid-js";
import {Footer} from "./footer";
import {A} from "@solidjs/router";

type Props = {
    disableHeader?: boolean,
    enablePoweredBy?: boolean,
    children: JSX.Element,
}

export function Page(props: Props) {
    return <div class="w-screen h-screen flex flex-col">
        {!props.disableHeader && <header class="navbar bg-base-200">
            <div class="navbar-start"></div>
            <div class="navbar-center">
                <A href="/" class="btn btn-ghost text-2xl">
                    ReaBoot
                </A>
            </div>
            <div class="navbar-end"></div>
        </header>
        }
        <main class="grow min-h-0 overflow-y-auto flex flex-col p-6 sm:items-center">
            {props.children}
        </main>
        <Footer enablePoweredBy={props.enablePoweredBy}/>
    </div>
}