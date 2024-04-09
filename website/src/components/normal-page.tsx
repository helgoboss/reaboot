import {JSX} from "solid-js";
import {Footer} from "./footer";
import {A} from "@solidjs/router";

type Props = {
    children: JSX.Element,
}

export function NormalPage(props: Props) {
    return <div class="w-screen h-screen flex flex-col">
        <header class="navbar bg-base-200">
            <div class="navbar-start"></div>
            <div class="navbar-center">
                <A href="/" class="btn btn-ghost text-2xl">ReaBoot</A>
            </div>
            <div class="navbar-end"></div>
        </header>
        <main class="grow min-h-0 overflow-y-auto flex flex-col items-center p-6">
            {props.children}
        </main>
        <Footer/>
    </div>
}