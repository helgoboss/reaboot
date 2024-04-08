import {createResource, For, Match, Switch} from 'solid-js';
import {Step} from "../components/step";
import {FaRegularClipboard, FaSolidDownload} from "solid-icons/fa";
import {UAParser} from 'ua-parser-js';
import {Params, useParams} from "@solidjs/router";
import {Recipe} from "../../../core/bindings/Recipe";
import {tryExtractRecipe} from "../../../commons/src/recipe-util";

export default function Install() {
    const params = useParams();
    const [recipeResource] = createResource(params, tryGetRecipeFromParams);
    const primaryDownload = getOptimalReabootDownload();
    const otherDownloads = reabootDownloads.filter(d => d.label !== primaryDownload?.label);
    return (
        <div class="grow flex flex-col max-w-lg items-stretch gap-4">
            <Switch>
                <Match when={recipeResource.loading}>
                    <span class="loading loading-ball loading-md"/>
                </Match>
                <Match when={recipeResource()}>
                    {recipe => <>
                        <h1 class="text-center text-3xl font-bold">
                            Let's install {displayRecipeHeading(recipe())}!
                        </h1>
                        <div role="tablist" class="tabs tabs-boxed m-4">
                            <a role="tab" class="tab tab-active">Via ReaBoot</a>
                            <a role="tab" class="tab">Via ReaPack</a>
                        </div>
                        <div class="text-center">
                            Using ReaBoot is the easiest way to install {displayRecipeNormal(recipe())}, especially if
                            you
                            don't have
                            ReaPack yet.
                        </div>
                        <Step index={0} title="Download ReaBoot">
                            <div>
                                ReaBoot is an installer that takes care of&#32;
                                <span class="underline tooltip tooltip-right"
                                      data-tip="It even installs REAPER and ReaPack, if you don't have those yet!">
                            everything
                        </span>.
                            </div>
                            <Switch>
                                <Match when={primaryDownload}>
                                    {d =>
                                        <a href={buildDownloadUrl(d())} class="btn btn-accent">
                                            <FaSolidDownload/>
                                            ReaBoot for {d().label}
                                        </a>
                                    }
                                </Match>
                                <Match when={true}>
                                    <a class="btn btn-primary disabled">
                                        <FaSolidDownload/>
                                        OS probably unsupported
                                    </a>
                                </Match>
                            </Switch>
                            <div class="text-xs">
                                <div class="divider">Looking for another download?</div>
                                <div class="flex flex-wrap justify-center gap-3">
                                    <For each={otherDownloads}>
                                        {d => <a href={buildDownloadUrl(d)} class="btn btn-xs">{d.label}</a>}
                                    </For>
                                </div>
                            </div>
                        </Step>
                        <Step index={1} title="Start ReaBoot">
                            <div>
                                Start the installer and follow its instructions.
                            </div>
                            <div class="collapse collapse-arrow bg-base-300">
                                <input type="checkbox"/>
                                <div class="collapse-title text-xl">
                                    Having issues?
                                </div>
                                <div class="collapse-content prose prose-sm">
                                    <dl>
                                        <dt>
                                            Does the installer ask you for a recipe?
                                        </dt>
                                        <dd>
                                            In that case, press&#32;
                                            <button class="btn btn-xs btn-accent">
                                                <FaRegularClipboard/>
                                                Copy recipe
                                            </button>
                                            &#32;and then&#32; paste the recipe in ReaBoot!
                                        </dd>
                                        <dt>
                                            Doesn't it work on your system?
                                        </dt>
                                        <dd>
                                            Install {displayRecipeNormal(recipe())} <a href="" class="link">via
                                            ReaPack</a> instead!
                                        </dd>
                                    </dl>
                                </div>
                            </div>
                        </Step>
                    </>
                    }
                </Match>
            </Switch>
        </div>
    );
}

type ReabootDownload = {
    label: string,
    asset: string,
}

function getOptimalReabootDownload(): ReabootDownload | null {
    const parser = UAParser();
    switch (parser.os.name) {
        case "Mac OS":
            return macOsDownload;
        case "Windows":
            return windowsX64Download
        case "Linux":
            switch (parser.cpu.architecture) {
                case "amd64":
                    return linuxX86_64Download;
                case "arm64":
                    return linuxAarch64Download;
                default:
                    return null;
            }
        default:
            return null;
    }
}

const macOsDownload = {
    label: "macOS Universal",
    asset: "reaboot-universal.dmg"
};
const linuxX86_64Download = {
    label: "Linux x86_64 (DEB)",
    asset: "reaboot-x86_64.deb",
};
const linuxAarch64Download = {
    label: "Linux aarch64 (DEB)",
    asset: "reaboot-aarch64.deb",
};
const windowsX64Download = {
    label: "Windows x64",
    asset: "reaboot.exe",
};

const reabootDownloads = [
    macOsDownload,
    linuxX86_64Download,
    linuxAarch64Download,
    windowsX64Download,
];

function buildDownloadUrl(download: ReabootDownload): string {
    return `https://github.com/helgoboss/reaboot/releases/download/latest/${download.asset}`
}

async function tryGetRecipeFromParams(params: Partial<Params>): Promise<Recipe | null> {
    const thing = params.thing;
    if (!thing) {
        return null;
    }
    const decodedThing = tryDecodeThing(thing);
    if (!decodedThing) {
        return null;
    }
    return tryExtractRecipe(decodedThing);
}

function tryDecodeThing(thing: string): string | null {
    try {
        return decodeURIComponent(thing);
    } catch {
        return null;
    }
}

function displayRecipeNormal(recipe: Recipe) {
    if (recipeNameIsSpecial(recipe.name)) {
        return <span class="badge">{recipe.name}</span>;
    } else {
        return recipe.name;
    }
}

function displayRecipeHeading(recipe: Recipe) {
    return recipeNameIsSpecial(recipe.name) ? `"${recipe.name}"` : recipe.name;
}

function recipeNameIsSpecial(name: string) {
    return name.includes(".");
}