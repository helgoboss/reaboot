import {copyTextToClipboard} from "../util/clipboard-util";
import {Step} from "./step";
import {For, JSX, Match, Show, Switch} from "solid-js";
import {FaSolidDownload, FaSolidLightbulb, FaSolidThumbsUp} from "solid-icons/fa";
import {Collapsible} from "@kobalte/core";
import {CopyField} from "./copy-field";
import {UAParser} from "ua-parser-js";
import {formatRecipeAsJson, ParsedRecipe} from "reaboot-commons/src/recipe-util";
import {RecipeRef} from "./recipe-ref";
import {ReaperRef} from "reaboot-commons/src/components/ReaperRef";
import {showToast} from "../util/toast-util";
import {Help} from "reaboot-commons/src/components/Help";
import {ReapackRef} from "reaboot-commons/src/components/ReapackRef";

const LATEST_REABOOT_VERSION = "1.0.0";

export function InstallViaReaboot(props: { recipe: ParsedRecipe }) {
    const osName = UA_PARSER_RESULT.os.name;
    const downloadConfig = getDownloadConfig(osName);
    const otherDownloads = reabootDownloads.filter(d => {
        return downloadConfig.mainDownloads.every(optimalDownload => d.label !== optimalDownload?.label);
    });
    const copyRecipeMain = async () => {
        const success = await copyTextToClipboard(formatRecipeAsJson(props.recipe.raw));
        if (success) {
            showToast("alert-success", "Started download and copied installation recipe to clipboard successfully!");
        } else {
            showToast("alert-warning", "Started download but couldn't copy installation recipe to clipboard!")
        }
    };

    return <div class="grow flex flex-col max-w-xl items-stretch">
        <div class="text-center">
            Using the installer is the easiest way to install <RecipeRef recipe={props.recipe}/>.
            It automatically installs <ReapackRef/> and even <ReaperRef/>, if necessary.
        </div>
        <Step index={0} title="Download Installer">
            <Switch>
                <Match when={downloadConfig.mainDownloads.length > 0}>
                    <>
                        <div class="grid grid-flow-col justify-center gap-2">
                            <For each={downloadConfig.mainDownloads}>
                                {(d, i) =>
                                    <Help help={d.description}>
                                        <a href={buildDownloadUrl(d)}
                                           onclick={() => copyRecipeMain()}
                                           class="btn btn-accent">
                                            <FaSolidDownload/>
                                            {d.label}
                                            {downloadConfig.recommendFirstDownload && i() === 0 &&
                                                <Help help="Good default choice!">
                                                    <FaSolidThumbsUp/>
                                                </Help>
                                            }
                                        </a>
                                    </Help>
                                }
                            </For>
                        </div>
                    </>
                </Match>
            </Switch>
            <div class="mt-3">
                {downloadConfig.downloadComment}
            </div>

            <CollapsedInfo title="Looking for another download?">
                <div class="flex flex-wrap justify-center gap-3 mb-4">
                    <For each={otherDownloads}>
                        {d =>
                            <Help help={d.description}>
                                <a href={buildDownloadUrl(d)}
                                   onclick={() => copyRecipeMain()}
                                   class="btn btn-xs">
                                    {d.label}
                                </a>
                            </Help>
                        }
                    </For>
                </div>
            </CollapsedInfo>
        </Step>
        <Step index={1} title="Start Installer">
            <div>
                Once you've downloaded the installer, please run it and follow the instructions!
            </div>
            <CollapsedInfo title="Having issues?">
                <div class="p-4 prose prose-sm">
                    <dl>
                        <Show when={UA_PARSER_RESULT.os.name === "Windows"}>
                            <dt>
                                Issues with SmartScreen?
                            </dt>
                            <dd class="p-0">
                                Microsoft Defender SmartScreen may warn you with
                                "Windows protected your PC" when starting the installer. If that happens, click
                                "More info" and then "Run anyway".
                            </dd>
                            <dt>
                                Does the installer window open and immediately close again?
                            </dt>
                            <dd class="p-0">
                                First, install the&#32;
                                <a class="link" href="https://go.microsoft.com/fwlink/p/?LinkId=2124703">
                                    Microsoft Edge WebView2 runtime
                                </a>, then try running the installer again.
                            </dd>
                        </Show>
                        <dt>
                            Is the installer asking for a recipe?
                        </dt>
                        <dd class="p-0">
                            If so, press&#32;
                            <CopyField text={() => formatRecipeAsJson(props.recipe.raw)}>Copy recipe</CopyField>
                            &#32;and then&#32; paste the recipe into the installer!
                        </dd>
                        <dt>
                            Installer not starting at all?
                        </dt>
                        <dd class="p-0">
                            Your system might not be modern enough to run the installer.
                            Try&nbsp;
                            <a href="?via=reapack" class="link">installation via ReaPack</a>
                            &nbsp;instead!
                        </dd>
                    </dl>
                </div>
            </CollapsedInfo>
        </Step>
    </div>
}

type ReabootDownloadConfig = {
    downloadComment: JSX.Element,
    mainDownloads: ReabootDownload[],
    recommendFirstDownload: boolean,
}

type ReabootDownload = {
    label: string,
    asset: string,
    description: string,
}

function getDownloadConfig(osName: string): ReabootDownloadConfig {
    // Unknown or undetected
    switch (osName) {
        case "macOS":
            switch (UA_PARSER_RESULT.cpu.architecture) {
                case "arm64":
                    return {
                        downloadComment: <>{REGULAR_DOWNLOAD_COMMENT}</>,
                        mainDownloads: [macOsArm64Download, macOsX86_64Download],
                        recommendFirstDownload: true,
                    };
                case "amd64":
                    return {
                        downloadComment: <>{REGULAR_DOWNLOAD_COMMENT}</>,
                        mainDownloads: [macOsX86_64Download, macOsArm64Download],
                        recommendFirstDownload: true,
                    };
                default:
                    return {
                        downloadComment: <>{REGULAR_DOWNLOAD_COMMENT}</>,
                        mainDownloads: [macOsArm64Download, macOsX86_64Download],
                        recommendFirstDownload: false,
                    };
            }
        case "Windows":
            switch (UA_PARSER_RESULT.os.version) {
                case "7":
                case "8":
                    return {
                        downloadComment: <>
                            <span class="text-error font-bold mr-1">Attention:</span>
                            Before executing the installer, you <span class="font-bold">must</span> install the&#32;
                            <a class="link" href="https://go.microsoft.com/fwlink/p/?LinkId=2124703">
                                Microsoft Edge WebView2 runtime
                            </a>!
                            This is necessary on Windows 7 and 8 only.
                        </>,
                        mainDownloads: [windowsX64ExeDownload],
                        recommendFirstDownload: false,
                    };
                case "10":
                case "11":
                    return {
                        downloadComment: <>
                            {REGULAR_DOWNLOAD_COMMENT}
                        </>,
                        mainDownloads: [windowsX64ExeDownload],
                        recommendFirstDownload: false,
                    };
                default:
                    // Unknown or undetected
                    return {
                        downloadComment: <>
                            <span class="text-error font-bold mr-1">Attention:</span>
                            If you are running Windows 7 or 8, you might have to install the&#32;
                            <a class="link" href="https://go.microsoft.com/fwlink/p/?LinkId=2124703">
                                Microsoft Edge WebView2 runtime
                            </a> before executing the installer. Otherwise it won't start!
                        </>,
                        mainDownloads: [windowsX64ExeDownload],
                        recommendFirstDownload: false,
                    };
            }
        // case "Linux":
        //     return {
        //         downloadComment: <>Requires at least glibc 2.35 (e.g. Ubuntu 22.04+)</>,
        //         mainDownloads: [linuxX86_64Download],
        //         recommendFirstDownload: false,
        //     };
        default:
            return {
                downloadComment: <>
                    The installer is not available for your system. Try&nbsp;
                    <a href="?via=reapack" class="link">installation via ReaPack</a>
                    &nbsp;instead!
                </>,
                mainDownloads: [],
                recommendFirstDownload: false,
            };
    }
}

const UA_PARSER_RESULT = UAParser();

const REGULAR_DOWNLOAD_COMMENT = "Some browsers may flag the download as suspicious — just ignore the warning! You might also need to grant your browser permission to copy text to the clipboard.";

const macOsArm64Download = {
    label: "macOS ARM64",
    asset: "ReaBoot-macos-arm64.zip",
    description: "Choose this if you have an Apple Silicon CPU (M1 or newer) and don't use Rosetta."
};
const macOsX86_64Download = {
    label: "macOS Intel",
    asset: "ReaBoot-macos-x86_64.zip",
    description: "Choose this if you have an Intel 64-bit CPU or if you want to use Rosetta on an Apple Silicon CPU."
};
const linuxX86_64Download = {
    label: "Linux x86_64 (deb)",
    asset: "ReaBoot-linux-x86_64.deb",
    description: "This is a package suitable for Debian and Debian derivatives (e.g. Ubuntu). If you need other Linux package formats, write to info@helgoboss.org.",
};
const windowsX64ExeDownload = {
    label: "Windows x64",
    asset: "ReaBoot-windows-x64.exe",
    description: "This runs ReaBoot directly without having to install it first (recommended)."
};

const reabootDownloads = [
    macOsArm64Download,
    macOsX86_64Download,
    windowsX64ExeDownload,
    linuxX86_64Download,
];

function buildDownloadUrl(download: ReabootDownload): string {
    return `https://github.com/helgoboss/reaboot/releases/download/v${LATEST_REABOOT_VERSION}/${download.asset}`
}

function CollapsedInfo(props: { title: string, children: JSX.Element }) {
    return <Collapsible.Root class="mt-3 collapse collapse-arrow data-[expanded]:collapse-open bg-base-300">
        <Collapsible.Trigger class="collapse-title flex flex-row items-center justify-center">
            <FaSolidLightbulb class="mr-2"/> {props.title}
        </Collapsible.Trigger>
        <Collapsible.Content>
            {props.children}
        </Collapsible.Content>
    </Collapsible.Root>
}