import {navigateTo} from "../epics/common.tsx";
import {mainStore} from "../globals.ts";
import {Show} from "solid-js";

export function WelcomePage() {
    return (
        <div class="grow hero">
            <div class="hero-content text-center">
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">Welcome!</h1>
                    <p class="py-6">
                        ReaBoot is a convenient all-in-one online installer for&#32;
                        <span class="whitespace-nowrap">
                            <a href="https://reaper.fm/" target="_blank"
                               class="tooltip tooltip-success underline"
                               data-tip="The DAW we all love">REAPER</a>,&#32;
                            <a href="https://reapack.com/" target="_blank"
                               class="tooltip tooltip-success underline"
                               data-tip="The standard package manager for REAPER">
                                ReaPack
                            </a>
                            &#32;and any&#32;
                            <span class="tooltip underline"
                                  data-tip="3rd-party add-ons for REAPER, e.g. scripts, extensions and themes.">
                                packages
                            </span>
                        </span>
                        &#32;of your choice.
                    </p>
                    <Show when={mainStore.state.recipeId}>
                        <p class="pb-6">
                            Today we are going to install:&nbsp;
                            <Show when={mainStore.state.resolvedConfig?.recipe} fallback={
                                <span class="loading loading-ball loading-lg"></span>
                            }>
                                {recipe => recipe().name}
                            </Show>
                        </p>
                    </Show>
                    <button class="btn btn-primary"
                            onClick={() => navigateTo("pick-reaper")}>
                        Let's go!
                    </button>
                </div>
            </div>
        </div>
    );
}