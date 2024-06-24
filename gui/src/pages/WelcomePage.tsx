import {navigateTo, showError, showSuccess} from "../epics/common.tsx";
import {mainStore} from "../globals.ts";
import {Match, Show, Switch} from "solid-js";
import {applyRecipeFromClipboard, applyRecipeFromText} from "../epics/welcome.ts";
import {Help} from "../components/Help.tsx";
import {ButtonRow} from "../components/ButtonRow.tsx";

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
                            <Help help="The DAW we all love">
                                <a href="https://reaper.fm/" target="_blank" class="link">
                                    REAPER
                                </a>
                            </Help>,&#32;
                            <Help help="The standard package manager for REAPER">
                                <a href="https://reapack.com/" target="_blank" class="link">
                                    ReaPack
                                </a>
                            </Help>
                            &#32;and arbitrary&#32;
                            <Help help="3rd-party add-ons for REAPER such as scripts, extensions and themes">
                                packages
                            </Help>
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
                                            <Help help={recipe().description} placement="top">
                                                <a href={recipe().website ?? undefined} target="_blank">
                                                    <Switch>
                                                        <Match when={true}>
                                                            <a href={recipe().website ?? undefined}
                                                               target="_blank"
                                                               class="text-3xl font-bold">
                                                                {recipe().name}
                                                            </a>
                                                            <Show when={recipe().sub_title}>
                                                                <div class="italic">
                                                                    {recipe().sub_title}
                                                                </div>
                                                            </Show>
                                                        </Match>
                                                    </Switch>
                                                </a>
                                            </Help>
                                        }
                                    </Show>
                                </div>
                                <button class="btn btn-primary mt-6"
                                        onClick={() => navigateTo("pick-reaper")}>
                                    Let's go!
                                </button>
                            </Match>
                            <Match when={true}>
                                <div role="alert" class="alert text-center">
                                    <div>
                                        <p>
                                            ReaBoot tried to find an installation recipe in your system clipboard
                                            but there was none. Please choose!
                                        </p>
                                        <ButtonRow>
                                            <Help
                                                help="ReaBoot's default recipe allows to you to install some of the most popular REAPER scripts and extensions out there!">
                                                <button class="btn btn-accent btn-xs mt-2"
                                                        onClick={() => applyRecipeFromText(RECIPE_DEFAULT_URL)}>
                                                    Use default recipe
                                                </button>
                                            </Help>
                                            <Help
                                                help="Choose this if you want to install something that is not contained in the default recipe. First, copy the desired recipe to the clipboard, then paste it here!">
                                                <button class="btn btn-accent btn-xs mt-2"
                                                        onClick={() => applyRecipeFromClipboardWithNotifications()}>
                                                    Paste custom recipe
                                                </button>
                                            </Help>
                                        </ButtonRow>
                                    </div>
                                </div>
                            </Match>
                        </Switch>
                    </div>
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

const RECIPE_DEFAULT_URL = "https://raw.githubusercontent.com/helgoboss/reaboot/main/recipes/default.json";