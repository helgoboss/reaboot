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
                <A href="/"><ReabootLogo class="h-32 mb-6"/></A>
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
                                <a href={example()[2]} class={`badge badge-outline ${example()[1]}`}>
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
    ["ReaLearn", "bg-lime-200 text-black", "http://localhost:3000/install/https%3A%2F%2Fraw.githubusercontent.com%2Fhelgoboss%2Freaboot-recipes%2Fmain%2Frecipes%2Frealearn.json"],
    ["Rodilab Color Palette", "bg-sky-200 text-black", "http://localhost:3000/install/https%3A%2F%2Fgithub.com%2FReaTeam%2FReaScripts%2Fraw%2Fmaster%2Findex.xml%23p%3DVarious%2Frodilab_Color%2520palette.lua%26v%3Dlatest"],
    ["just ReaPack", "bg-amber-200 text-black", "http://localhost:3000/install/%7B%0A%20%20%20%20%22name%22%3A%20%22ReaPack%22%2C%0A%20%20%20%20%22website%22%3A%20%22https%3A%2F%2Freapack.com%2F%22%2C%0A%20%20%20%20%22author%22%3A%20%22Christian%20Fillion%22%2C%0A%20%20%20%20%22required_packages%22%3A%20%5B%0A%22https%3A%2F%2Fraw.githubusercontent.com%2Fcfillion%2Freapack%2Fmaster%2Findex.xml%23p%3DExtensions%2FReaPack.ext%22%0A%20%20%20%20%5D%0A%7D"],
    ["a large tool collection with optional features", "bg-fuchsia-200 text-black", "http://localhost:3000/install/%7B%22name%22%3A%22Helgo's%20random%20tool%20collection%22%2C%22description%22%3A%22This%20is%20just%20an%20example%20recipe%20for%20ReaBoot%20in%20order%20to%20demonstrate%20how%20to%20share%20a%20complete%20collection%20of%20packages%20and%20make%20some%20of%20them%20optional.%22%2C%22author%22%3A%22helgoboss%22%2C%22website%22%3A%22https%3A%2F%2Fwww.reaboot.com%2F%22%2C%22required_packages%22%3A%5B%22https%3A%2F%2Fraw.githubusercontent.com%2FReaTeam%2FExtensions%2Fmaster%2Findex.xml%23p%3DAPI%2Freaper_imgui.ext%22%5D%2C%22features%22%3A%7B%22sws%22%3A%7B%22name%22%3A%22SWS%2FS%26M%20Extension%22%2C%22default%22%3Atrue%2C%22description%22%3A%22Popular%20and%20established%20extension%20that%20adds%20a%20variety%20of%20smaller%20features%20to%20REAPER.%20Considered%20as%20must-have%20by%20many.%22%2C%22packages%22%3A%5B%22https%3A%2F%2Fraw.githubusercontent.com%2FReaTeam%2FExtensions%2Fmaster%2Findex.xml%23p%3DExtensions%2Freaper-oss_SWS.ext%26v%3Dlatest-pre%22%5D%7D%2C%22libraries%22%3A%7B%22name%22%3A%22Common%20libraries%22%2C%22description%22%3A%22Frequently%20used%20libraries%20that%20provide%20functions%20for%20ReaScripts%22%2C%22packages%22%3A%5B%22https%3A%2F%2Fgithub.com%2FUltraschall%2Fultraschall-lua-api-for-reaper%2Fraw%2Fmaster%2Fultraschall_api_index.xml%23p%3DUltraschall-API-category%2FUltraschall%2520API%2520package%22%2C%22https%3A%2F%2Fgithub.com%2FReaTeam%2FReaScripts%2Fraw%2Fmaster%2Findex.xml%23p%3DDevelopment%2FLokasenna_GUI%2520library%2520v2.lua%22%2C%22https%3A%2F%2Fgithub.com%2FReaTeam%2FReaScripts%2Fraw%2Fmaster%2Findex.xml%23p%3DDevelopment%2FLokasenna_Scythe%2520library%2520v3.lua%22%5D%7D%2C%22realearn%22%3A%7B%22name%22%3A%22ReaLearn%22%2C%22description%22%3A%22The%20%5C%22Swiss%20Army%20Knife%5C%22%20among%20the%20REAPER%20controller%20integration%20tools%22%2C%22packages%22%3A%5B%22https%3A%2F%2Fgithub.com%2Fhelgoboss%2Freaper-packages%2Fraw%2Fmaster%2Findex.xml%23p%3DExtensions%2FReaLearn-x64%26v%3Dlatest%22%5D%7D%7D%7D"],
];