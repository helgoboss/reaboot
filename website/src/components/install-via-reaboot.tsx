import {copyTextToClipboard} from "../util/clipboard-util";
import {Step} from "./step";
import {For, Match, Switch} from "solid-js";
import {FaSolidDownload} from "solid-icons/fa";
import {Collapsible} from "@kobalte/core";
import {CopyField} from "./copy-field";
import {UAParser} from "ua-parser-js";
import {Recipe} from "../../../core/bindings/Recipe";
import {recipeNameIsSpecial} from "../util/recipe-util";
import {ParsedRecipe} from "reaboot-commons/src/recipe-util";

export function InstallViaReaboot(props: { recipe: ParsedRecipe }) {
    const optimalDownloads = getOptimalReabootDownloads();
    const otherDownloads = reabootDownloads.filter(d => {
        return optimalDownloads.every(optimalDownload => d.label !== optimalDownload?.label);
    });
    const getRecipeAsJson = () => JSON.stringify(props.recipe, null, "    ");
    const copyRecipeMain = async () => {
        await copyTextToClipboard(getRecipeAsJson());
    };

    return <div class="grow flex flex-col max-w-lg items-stretch gap-6">
        <div class="text-center">
            ReaBoot is the easiest way to install {displayRecipeNormal(props.recipe)}.
            It automatically installs REAPER and ReaPack if necessary.
        </div>
        <Step index={0} title="Download ReaBoot">
            <Switch>
                <Match when={optimalDownloads.length > 0}>
                    <>
                        <div>
                            For your system:
                        </div>
                        <div class="flex flex-row justify-center gap-2">
                            <For each={optimalDownloads}>
                                {d =>
                                    <a href={buildDownloadUrl(d)}
                                       onclick={() => copyRecipeMain()}
                                       class="btn btn-accent">
                                        <FaSolidDownload/>
                                        {d.label}
                                    </a>
                                }
                            </For>
                        </div>
                    </>
                </Match>
                <Match when={true}>
                    <div>
                        ReaBoot is currently not available for your system.
                        Try installation via ReaPack instead!
                    </div>
                </Match>
            </Switch>
            <div class="text-xs">
                <div class="divider">Looking for another download?</div>
                <div class="flex flex-wrap justify-center gap-3">
                    <For each={otherDownloads}>
                        {d =>
                            <a href={buildDownloadUrl(d)}
                               onclick={() => copyRecipeMain()}
                               class="btn btn-xs">
                                {d.label}
                            </a>
                        }
                    </For>
                </div>
            </div>
        </Step>
        <Step index={1} title="Start ReaBoot">
            <div>
                Start the installer and follow its instructions.
            </div>
            <Collapsible.Root class="collapse collapse-arrow data-[expanded]:collapse-open bg-base-300 ">
                <Collapsible.Trigger class="collapse-title">
                    Having issues?
                </Collapsible.Trigger>
                <Collapsible.Content class="p-4 prose prose-sm">
                    <dl>
                        <dt>
                            Does the installer ask you for a recipe?
                        </dt>
                        <dd class="p-0">
                            In that case, press&#32;
                            <CopyField text={getRecipeAsJson}>Copy recipe</CopyField>
                            &#32;and then&#32; paste the recipe into ReaBoot!
                        </dd>
                        <dt>
                            Doesn't work on your system?
                        </dt>
                        <dd class="p-0">
                            It's possible that your system is not modern enough to run ReaBoot.
                            Try installation via ReaPack instead!
                        </dd>
                    </dl>
                </Collapsible.Content>
            </Collapsible.Root>
        </Step>
    </div>
}

type ReabootDownload = {
    label: string,
    asset: string,
}

function getOptimalReabootDownloads(): ReabootDownload[] {
    const parser = UAParser();
    switch (parser.os.name) {
        case "Mac OS":
            return [macOsArm64Download, macOsX86_64Download];
        case "Windows":
            return [windowsX64Download];
        case "Linux":
            return [linuxX86_64Download];
        // switch (parser.cpu.architecture) {
        //     case "amd64":
        //         return linuxX86_64Download;
        //     case "arm64":
        //         return linuxAarch64Download;
        //     default:
        //         return null;
        // }
        default:
            return [];
    }
}

const macOsArm64Download = {
    label: "macOS ARM64",
    asset: "reaboot-arm64.dmg"
};
const macOsX86_64Download = {
    label: "macOS Intel",
    asset: "reaboot-x86_64.dmg"
};
const linuxX86_64Download = {
    label: "Linux x86_64 (deb)",
    asset: "reaboot-x86_64.deb",
};
const windowsX64Download = {
    label: "Windows x64",
    asset: "reaboot-x64.exe",
};

const reabootDownloads = [
    macOsArm64Download,
    macOsX86_64Download,
    windowsX64Download,
    linuxX86_64Download,
];

function buildDownloadUrl(download: ReabootDownload): string {
    return `https://github.com/helgoboss/reaboot/releases/download/latest/${download.asset}`
}


function displayRecipeNormal(recipe: ParsedRecipe) {
    if (recipeNameIsSpecial(recipe.raw.name)) {
        return <span class="badge">{recipe.raw.name}</span>;
    } else {
        return recipe.raw.name;
    }
}