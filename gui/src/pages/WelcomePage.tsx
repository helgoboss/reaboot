import {navigateTo} from "../epics/common.tsx";
import {mainStore} from "../globals.ts";
import {Match, Show, Switch} from "solid-js";

export function WelcomePage() {
    return (
        <div class="grow hero">
            <Show when={mainStore.state.resolvedConfig?.recipe?.logo}>
                {logo =>
                    <div style={`background-image: url(${logo()}); opacity: 0.03`}
                         class="bg-no-repeat bg-center h-full w-full">
                    </div>
                }
            </Show>
            <div class="hero-content text-center">
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">Welcome!</h1>
                    <p class="pt-6">
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
                            &#32;and arbitrary&#32;
                            <span class="tooltip underline"
                                  data-tip="3rd-party add-ons for REAPER, e.g. scripts, extensions and themes">
                                packages
                            </span>
                        </span>.
                    </p>
                    <Show when={mainStore.state.recipeId || mainStore.state.resolvedConfig?.recipe}>
                        <div>
                            <div class="divider"></div>
                            <p>Today we are going to install the package</p>
                            <div class="pb-6">
                                <Show when={mainStore.state.resolvedConfig?.recipe} fallback={
                                    <span class="loading loading-ball loading-lg"></span>
                                }>
                                    {recipe =>
                                        <a href={recipe().website ?? undefined} target="_blank">
                                            <Switch>
                                                {/*<Match when={recipe().logo}>*/}
                                                {/*    {logo =>*/}
                                                {/*        <img src={logo()} alt={recipe().name} class="h-20 m-auto"/>*/}
                                                {/*    }*/}
                                                {/*</Match>*/}
                                                <Match when={true}>
                                                    <div class="text-3xl font-bold">
                                                        {recipe().name}
                                                    </div>
                                                    <Show when={recipe().manufacturer}>
                                                        <div class="italic">
                                                            by {recipe().manufacturer}
                                                        </div>
                                                    </Show>
                                                </Match>
                                            </Switch>
                                        </a>
                                    }
                                </Show>
                            </div>
                        </div>
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