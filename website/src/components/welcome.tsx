import {A} from "@solidjs/router";
import {Index, Show} from "solid-js";
import ReabootLogo from '../../../commons/src/reaboot-logo.svg?component-solid';

type Props = {
    poweredBy: boolean,
    examples: boolean
}

export function Welcome(props: Props) {
    return <div class="grow hero">
        <div class="hero-content text-center">
            <div class="flex flex-col items-center max-w-md">
                <A href="/"><ReabootLogo class="fill-primary mb-6"/></A>
                {props.poweredBy && <h1>
                    <span class="italic">Powered by</span>
                    &#32;
                    <A href="/" class="text-2xl font-bold">ReaBoot</A>
                </h1>
                }
                <p class="py-6">
                    ReaBoot is a convenient all-in-one online installer for&#32;
                    <span class="whitespace-nowrap"><a href="https://reaper.fm/" target="_blank"
                                                       class="tooltip tooltip-success underline"
                                                       data-tip="The DAW we all love">
                            REAPER
                        </a>,</span>&#32;
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
                <Show when={props.examples}>
                    <div class="mb-3 font-bold">Some example links:</div>
                    <div class="flex flex-wrap justify-center gap-4">
                        <Index each={examples}>
                            {example =>
                                <a href={example()[2]} class="badge badge-outline"
                                   classList={{[`${example()[1]}`]: true}}>
                                    Install {example()[0]}
                                </a>
                            }
                        </Index>
                    </div>
                </Show>
                <p class="py-6">
                    Want to share REAPER packages or complete REAPER distributions and make
                    them installable in seconds - even for beginners?
                </p>
                <A href="/share" class="btn btn-primary">Here's how!</A>
            </div>
        </div>
    </div>
}

const examples = [
    ["ReaLearn", "bg-lime-200", "http://localhost:3000/install/https%3A%2F%2Fraw.githubusercontent.com%2Fhelgoboss%2Freaboot-recipes%2Fmain%2Frecipes%2Frealearn.json"],
    ["Rodilab Color Palette", "bg-sky-200", "http://localhost:3000/install/https%3A%2F%2Fgithub.com%2FReaTeam%2FReaScripts%2Fraw%2Fmaster%2Findex.xml%23p%3DVarious%2Frodilab_Color%2520palette.lua%26v%3Dlatest"],
    ["ReaPack", "bg-amber-200", "http://localhost:3000/install/%7B%0A%20%20%20%20%22name%22%3A%20%22ReaPack%22%2C%0A%20%20%20%20%22website%22%3A%20%22https%3A%2F%2Freapack.com%2F%22%2C%0A%20%20%20%20%22author%22%3A%20%22Christian%20Fillion%22%2C%0A%20%20%20%20%22required_packages%22%3A%20%5B%0A%22https%3A%2F%2Fraw.githubusercontent.com%2Fcfillion%2Freapack%2Fmaster%2Findex.xml%23p%3DExtensions%2FReaPack.ext%22%0A%20%20%20%20%5D%0A%7D"],
];