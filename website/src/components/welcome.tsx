import {FaBrandsGithub} from "solid-icons/fa";
import {A} from "@solidjs/router";
import {Index, Show} from "solid-js";

type Props = {
    poweredBy: boolean,
    examples: boolean
}

export function Welcome(props: Props) {
    return <div class="grow hero">
        <div class="hero-content text-center">
            <div class="max-w-md">
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
                                   classList={{[`bg-${example()[1]}-200`]: true}}>
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
    ["ReaLearn", "lime", "http://localhost:3000/install/https%3A%2F%2Fraw.githubusercontent.com%2Fhelgoboss%2Freaboot-recipes%2Fmain%2Frecipes%2Frealearn.json"],
    ["Rodilab Color Palette", "sky", "http://localhost:3000/install/https%3A%2F%2Fgithub.com%2FReaTeam%2FReaScripts%2Fraw%2Fmaster%2Findex.xml%23p%3DVarious%2Frodilab_Color%2520palette.lua%26v%3Dlatest"],
];