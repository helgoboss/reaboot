import {navigateTo, showError, showSuccess} from "../epics/common.tsx";
import {mainStore} from "../globals.ts";
import {Match, Show, Switch} from "solid-js";
import {applyRecipeFromClipboard} from "../epics/welcome.ts";

export function WelcomePage() {
    return (
        <div class="grow hero">
            <div class="hero-content text-center">
                <div class="max-w-lg">
                    <h1 class="text-5xl font-bold">Welcome!</h1>
                    <p class="pt-6">
                        <a href="https://www.reaboot.com/" target="_blank" class="link">ReaBoot</a>
                        &#32;is a convenient all-in-one online installer for&#32;
                        <span class="whitespace-nowrap">
                            <a href="https://reaper.fm/" target="_blank"
                               class="underline"
                               title="The DAW we all love">
                                REAPER
                            </a>,&#32;
                            <a href="https://reapack.com/" target="_blank"
                               class="underline"
                               title="The standard package manager for REAPER">
                                ReaPack
                            </a>
                            &#32;and arbitrary&#32;
                            <span class="underline"
                                  title="3rd-party add-ons for REAPER such as scripts, extensions and themes">
                                packages
                            </span>
                        </span>.
                    </p>
                    <div class="divider"></div>
                    <div>
                        <Switch>
                            <Match when={mainStore.state.resolvedConfig?.recipe}>
                                <p>Today we are going to install</p>
                                <div>
                                    <Show when={mainStore.state.resolvedConfig?.recipe} fallback={
                                        <span class="loading loading-ball loading-lg"></span>
                                    }>
                                        {recipe =>
                                            <a href={recipe().website ?? undefined} target="_blank">
                                                <Switch>
                                                    <Match when={true}>
                                                        <a href={recipe().website ?? undefined}
                                                           target="_blank"
                                                           class="text-3xl font-bold">
                                                            {recipe().name}
                                                        </a>
                                                        <Show when={recipe().author}>
                                                            <div class="italic">
                                                                by {recipe().author}
                                                            </div>
                                                        </Show>
                                                    </Match>
                                                </Switch>
                                            </a>
                                        }
                                    </Show>
                                </div>
                            </Match>
                            <Match when={true}>
                                <div role="alert" class="alert text-center">
                                    <div>
                                        <p>
                                            ReaBoot is currently running&#32;
                                            <span class="underline"
                                                  title="That means it's not pre-configured and will by default install REAPER and ReaPack only.">
                                                without recipe
                                            </span>.
                                            If you wanted to install something particular, please paste the recipe now!
                                        </p>
                                        <button class="btn btn-accent btn-xs mt-2"
                                                onClick={() => applyRecipeFromClipboardWithNotifications()}>
                                            Paste recipe
                                        </button>
                                    </div>
                                </div>
                            </Match>
                        </Switch>
                    </div>
                    <button class="btn btn-primary mt-6"
                            onClick={() => navigateTo("pick-reaper")}>
                        Let's go!
                    </button>
                </div>
            </div>
        </div>
    );
}

export async function applyRecipeFromClipboardWithNotifications() {
    try {
        await applyRecipeFromClipboard();
        showSuccess("Recipe pasted successfully!");
    } catch {
        showError("The clipboard doesn't contain any valid recipe!");
    }
}
