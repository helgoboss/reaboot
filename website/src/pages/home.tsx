import {NormalPage} from "../components/normal-page";
import {A} from "@solidjs/router";
import {createSignal, Index, JSX} from "solid-js";
import ReabootLogo from './../assets/reaboot-logo.svg?component-solid';
import {Tabs} from "@kobalte/core";

export default function Home() {
    const [userType, setUserType] = createSignal("user");
    return (
        <NormalPage>
            <div class="flex flex-col gap-4">
                <div class="grow hero">
                    <div class="hero-content text-center">
                        <div class="flex flex-col items-center max-w-md gap-6">
                            <A href="/"><ReabootLogo class="h-32"/></A>
                            <p>
                                ReaBoot is a convenient all-in-one online installer for&#32;
                                <span class="whitespace-nowrap">
                                    <a href="https://reaper.fm/" target="_blank"
                                       class="underline"
                                       title="The DAW we all love">
                                        REAPER
                                    </a>,
                                </span>&#32;
                                <a href="https://reapack.com/" target="_blank"
                                   class="underline"
                                   title="The standard package manager for REAPER">
                                    ReaPack
                                </a>
                                &#32;and arbitrary&#32;
                                <span class="underline"
                                      title="3rd-party add-ons for REAPER, e.g. scripts, extensions and themes">
                                                packages
                                </span>.
                            </p>
                        </div>
                    </div>
                </div>
                <div class="max-w-2xl">
                    <h2 class="font-bold text-2xl text-center">Please choose!</h2>
                    <Tabs.Root value={userType()} onChange={setUserType} class="flex flex-col sm:items-center">
                        <Tabs.List class="tabs tabs-boxed m-4 self-center">
                            <Tabs.Trigger value="user" class="tab data-[selected]:tab-active !h-auto">
                                I'm just a normal REAPER user!
                            </Tabs.Trigger>
                            <Tabs.Trigger value="dev" class="tab data-[selected]:tab-active !h-auto">
                                I'm a script or extension developer!
                            </Tabs.Trigger>
                        </Tabs.List>
                        <div class="card bg-base-200">
                            <div class="card-body">
                                <Tabs.Content value="user" class="flex flex-col gap-6">
                                    <h3 class="font-bold text-xl text-center">As a normal REAPER user ...</h3>
                                    <p>
                                        ... you are probably here for the&#32;
                                        <em>default installer</em>,
                                        which allows you to easily install some of the most
                                        popular REAPER scripts and extensions out there.
                                    </p>
                                    <div class="alert alert-info">
                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"
                                             class="stroke-current shrink-0 w-6 h-6">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                        </svg>
                                        <span>The default installer will be
                                    available soon!
                                </span>
                                    </div>
                                </Tabs.Content>
                                <Tabs.Content value="dev" class="flex flex-col gap-6">
                                    <h3 class="font-bold text-xl text-center">Already familiar with ReaBoot?</h3>
                                    <div class="text-center">
                                        <A href="/share" class="btn btn-secondary">Start creating installation
                                            links!</A>
                                    </div>
                                    <h3 class="font-bold text-xl text-center">ReaBoot is for you ...</h3>
                                    <ul class="list-disc ml-5">
                                        <li>
                                            If you want to offer your users a really easy way to install your own
                                            scripts or extensions.
                                        </li>
                                        <li>
                                            If you want to share your favorite collection of REAPER scripts or
                                            extensions with friends.
                                        </li>
                                    </ul>
                                    <h3 class="font-bold text-xl text-center">How does it work?</h3>
                                    <ol class="list-decimal ml-5">
                                        <li>You use this website to build a so-called &#32;<em>installation
                                            link</em>.
                                        </li>
                                        <li>You share this link with your users.</li>
                                        <li>
                                            Your users click the link and are able to install your scripts and
                                            extensions in seconds.
                                        </li>
                                    </ol>
                                    <h2 class="font-bold text-xl text-center">Some example links</h2>
                                    <p>
                                        The following links are example links, solely made for demonstrating ReaBoot's
                                        features. Don't share them!
                                    </p>
                                    <div class="flex flex-wrap justify-center gap-4">
                                        <Index each={examples}>
                                            {example =>
                                                <RecipeLink href={example()[2]} class={example()[1]}>
                                                    {example()[0]}
                                                </RecipeLink>
                                            }
                                        </Index>
                                    </div>
                                    <h3 class="font-bold text-xl text-center">Introduction video</h3>
                                    <iframe
                                        class="w-full aspect-video"
                                        width="560" height="315"
                                        src="https://www.youtube-nocookie.com/embed/LFveUpUrHFA?si=04UBLIDqVSpfjMXD"
                                        title="YouTube video player" frameborder="0"
                                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                                        referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>
                                </Tabs.Content>
                            </div>
                        </div>
                    </Tabs.Root>
                </div>
            </div>
        </NormalPage>
    );
}

const examples = [
    ["Install ReaLearn", "bg-lime-200 text-black", "https://reaboot.com/install/https%3A%2F%2Fraw.githubusercontent.com%2Fhelgoboss%2Freaboot-recipes%2Fmain%2Frecipes%2Frealearn.json"],
    ["Install Rodilab Color Palette", "bg-sky-200 text-black", "https://reaboot.com/install/https%3A%2F%2Fgithub.com%2FReaTeam%2FReaScripts%2Fraw%2Fmaster%2Findex.xml%23p%3DVarious%2Frodilab_Color%2520palette.lua%26v%3Dlatest"],
    ["Install just ReaPack", "bg-amber-200 text-black", "https://reaboot.com/install/%7B%0A%20%20%20%20%22name%22%3A%20%22ReaPack%22%2C%0A%20%20%20%20%22website%22%3A%20%22https%3A%2F%2Freapack.com%2F%22%2C%0A%20%20%20%20%22author%22%3A%20%22Christian%20Fillion%22%2C%0A%20%20%20%20%22required_packages%22%3A%20%5B%0A%22https%3A%2F%2Fraw.githubusercontent.com%2Fcfillion%2Freapack%2Fmaster%2Findex.xml%23p%3DExtensions%2FReaPack.ext%22%0A%20%20%20%20%5D%0A%7D"],
    ["Install a large tool collection with optional features", "bg-fuchsia-200 text-black", "https://reaboot.com/install/%7B%22name%22%3A%22Helgo's%20random%20tool%20collection%22%2C%22description%22%3A%22This%20is%20just%20an%20example%20recipe%20for%20ReaBoot%20in%20order%20to%20demonstrate%20how%20to%20share%20a%20complete%20collection%20of%20packages%20and%20make%20some%20of%20them%20optional.%22%2C%22author%22%3A%22helgoboss%22%2C%22website%22%3A%22https%3A%2F%2Fwww.reaboot.com%2F%22%2C%22required_packages%22%3A%5B%22https%3A%2F%2Fraw.githubusercontent.com%2FReaTeam%2FExtensions%2Fmaster%2Findex.xml%23p%3DAPI%2Freaper_imgui.ext%22%5D%2C%22features%22%3A%7B%22sws%22%3A%7B%22name%22%3A%22SWS%2FS%26M%20Extension%22%2C%22default%22%3Atrue%2C%22description%22%3A%22Popular%20and%20established%20extension%20that%20adds%20a%20variety%20of%20smaller%20features%20to%20REAPER.%20Considered%20as%20must-have%20by%20many.%22%2C%22packages%22%3A%5B%22https%3A%2F%2Fraw.githubusercontent.com%2FReaTeam%2FExtensions%2Fmaster%2Findex.xml%23p%3DExtensions%2Freaper-oss_SWS.ext%26v%3Dlatest-pre%22%5D%7D%2C%22libraries%22%3A%7B%22name%22%3A%22Common%20libraries%22%2C%22description%22%3A%22Frequently%20used%20libraries%20that%20provide%20functions%20for%20ReaScripts%22%2C%22packages%22%3A%5B%22https%3A%2F%2Fgithub.com%2FUltraschall%2Fultraschall-lua-api-for-reaper%2Fraw%2Fmaster%2Fultraschall_api_index.xml%23p%3DUltraschall-API-category%2FUltraschall%2520API%2520package%22%2C%22https%3A%2F%2Fgithub.com%2FReaTeam%2FReaScripts%2Fraw%2Fmaster%2Findex.xml%23p%3DDevelopment%2FLokasenna_GUI%2520library%2520v2.lua%22%2C%22https%3A%2F%2Fgithub.com%2FReaTeam%2FReaScripts%2Fraw%2Fmaster%2Findex.xml%23p%3DDevelopment%2FLokasenna_Scythe%2520library%2520v3.lua%22%5D%7D%2C%22realearn%22%3A%7B%22name%22%3A%22ReaLearn%22%2C%22description%22%3A%22The%20%5C%22Swiss%20Army%20Knife%5C%22%20among%20the%20REAPER%20controller%20integration%20tools%22%2C%22packages%22%3A%5B%22https%3A%2F%2Fgithub.com%2Fhelgoboss%2Freaper-packages%2Fraw%2Fmaster%2Findex.xml%23p%3DExtensions%2FReaLearn-x64%26v%3Dlatest%22%5D%7D%7D%7D"],
];

function RecipeLink(props: { href?: string, class: string, children: JSX.Element }) {
    return <a href={props.href} class={`badge badge-outline h-auto ${props.class}`}>
        {props.children}
    </a>;
}