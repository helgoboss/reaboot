import {JSX} from 'solid-js';
import {A, RouteSectionProps} from '@solidjs/router';

import {FaBrandsGithub} from "solid-icons/fa";

export function App(props: RouteSectionProps) {
    return (
        <div class="w-screen h-screen flex flex-row">
            <main class="grow flex flex-col items-center p-6 overflow-y-auto">
                {props.children}
            </main>
            <header class="max-w-sm bg-base-200 flex flex-col overflow-y-auto">
                <div class="grow hero">
                    <div class="hero-content text-center">
                        <div class="max-w-md">
                            <h1>
                                <span class="italic">Powered by</span>
                                &#32;
                                <span class="text-2xl font-bold">ReaBoot</span>
                            </h1>
                            <p class="py-6">
                                <a href="https://www.reaboot.com/" target="_blank" class="link">ReaBoot</a>
                                &#32;is a convenient all-in-one online installer for&#32;
                                <a href="https://reaper.fm/" target="_blank"
                                   class="tooltip tooltip-success underline"
                                   data-tip="The DAW we all love">
                                    REAPER
                                </a>,&#32;
                                <a href="https://reapack.com/" target="_blank"
                                   class="tooltip tooltip-success underline"
                                   data-tip="The standard package manager for REAPER">
                                    ReaPack
                                </a>
                                &#32;and arbitrary&#32;
                                <span class="tooltip underline"
                                      data-tip="3rd-party add-ons for REAPER, e.g. scripts, extensions and themes">
                                    packages
                                </span>.
                            </p>
                            <p class="py-6">
                                Do you want to share REAPER packages or even complete REAPER distributions and make
                                them installable in seconds (even for beginners)?
                            </p>
                            <A href="/share" class="btn btn-primary">Here's how!</A>
                        </div>
                    </div>
                </div>
                <footer class="footer items-center p-4 bg-base-300">
                    <A href="/faq" class="link">FAQ</A>
                    <nav class="grid-flow-col gap-4 md:place-self-center md:justify-self-end">
                        <a href="https://github.com/helgoboss/reaboot">
                            <FaBrandsGithub/>
                        </a>
                    </nav>
                </footer>
            </header>
        </div>
    );
}