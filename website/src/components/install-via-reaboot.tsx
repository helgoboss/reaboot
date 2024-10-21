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

const LATEST_REABOOT_VERSION = "0.7.0";

export function InstallViaReaboot(props: { recipe: ParsedRecipe }) {
    const downloadConfig = getDownloadConfig();
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
            ReaBoot is the easiest way to install <RecipeRef recipe={props.recipe}/>.
            It automatically installs <ReapackRef/> and even <ReaperRef/>, if necessary.
        </div>
        <Step index={0} title="Download ReaBoot">
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
            <div class="mt-3 text-xs">
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
        <Step index={1} title="Start ReaBoot">
            <div>
                After ReaBoot has been downloaded, simply start it and follow its instructions!
            </div>
            <CollapsedInfo title="Having issues?">
                <div class="p-4 prose prose-sm">
                    <dl>
                        <Show when={UA_PARSER_RESULT.os.name === "Windows"}>
                            <dt>
                                Issues with SmartScreen?
                            </dt>
                            <dd class="p-0">
                                Microsoft Defender SmartScreen might complain when you try to start the installer:
                                "Windows protected your PC". In that case, just click
                                "More info" and then "Run anyway".
                            </dd>
                        </Show>
                        <dt>
                            Does ReaBoot ask you for a recipe?
                        </dt>
                        <dd class="p-0">
                            In that case, press&#32;
                            <CopyField text={() => formatRecipeAsJson(props.recipe.raw)}>Copy recipe</CopyField>
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

function getDownloadConfig(): ReabootDownloadConfig {
    switch (UA_PARSER_RESULT.os.name) {
        case "macOS":
            switch (UA_PARSER_RESULT.cpu.architecture) {
                case "arm64":
                    return {
                        downloadComment: <>{SUSPICIOUS_DOWNLOAD_COMMENT}</>,
                        mainDownloads: [macOsArm64Download, macOsX86_64Download],
                        recommendFirstDownload: true,
                    };
                case "amd64":
                    return {
                        downloadComment: <>{SUSPICIOUS_DOWNLOAD_COMMENT}</>,
                        mainDownloads: [macOsX86_64Download, macOsArm64Download],
                        recommendFirstDownload: true,
                    };
                default:
                    return {
                        downloadComment: <>{SUSPICIOUS_DOWNLOAD_COMMENT}</>,
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
                            You are running an older Windows version. If you want to use the portable download, you will
                            probably have to install the&#32;
                            <a class="link" href="https://go.microsoft.com/fwlink/p/?LinkId=2124703">
                                Microsoft Edge WebView2 runtime
                            </a>
                            first, otherwise ReaBoot will not work.
                        </>,
                        mainDownloads: [windowsX64NsisDownload, windowsX64ExeDownload],
                        recommendFirstDownload: true,
                    };
                case "11":
                    return {
                        downloadComment: <>
                            {SUSPICIOUS_DOWNLOAD_COMMENT}
                        </>,
                        mainDownloads: [windowsX64ExeDownload, windowsX64MsiDownload],
                        recommendFirstDownload: true,
                    };
                // Windows 10 or not detected
                default:
                    return {
                        downloadComment: <>
                            {SUSPICIOUS_DOWNLOAD_COMMENT} If the portable download doesn't work, either use the
                            installer or first install the&#32;
                            <a class="link" href="https://go.microsoft.com/fwlink/p/?LinkId=2124703">
                                Microsoft Edge WebView2 runtime
                            </a>.
                        </>,
                        mainDownloads: [windowsX64ExeDownload, windowsX64MsiDownload],
                        recommendFirstDownload: true,
                    };
            }
        case "Linux":
            return {
                downloadComment: <>Requires at least glibc 2.35 (e.g. Ubuntu 22.04+)</>,
                mainDownloads: [linuxX86_64Download],
                recommendFirstDownload: false,
            };
        default:
            return {
                downloadComment: <>ReaBoot is not available for your system. Try installation via ReaPack instead!</>,
                mainDownloads: [],
                recommendFirstDownload: false,
            };
    }
}

const UA_PARSER_RESULT = UAParser();

const PREFER_PORTABLE_COMMENT = "If possible, take the portable download instead, because installing an installer is not optimal ;)";
const SUSPICIOUS_DOWNLOAD_COMMENT = "It's possible that some browsers flag the download as suspicious. In this case, you need to ignore the warning!";

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
    label: "Windows x64 (Portable)",
    asset: "ReaBoot-windows-x64.exe",
    description: "This runs ReaBoot directly without having to install it first (recommended)."
};
const windowsX64NsisDownload = {
    label: "Windows x64 (NSIS Installer)",
    asset: "ReaBoot-windows-x64-setup.exe",
    description: `This is an installer for ReaBoot that's compatible even with older Windows versions. ${PREFER_PORTABLE_COMMENT}`,
};
const windowsX64MsiDownload = {
    label: "Windows x64 (MSI Installer)",
    asset: "ReaBoot-windows-x64-setup.msi",
    description: `This is a native installer for ReaBoot. ${PREFER_PORTABLE_COMMENT}`,
};

const reabootDownloads = [
    macOsArm64Download,
    macOsX86_64Download,
    windowsX64ExeDownload,
    windowsX64NsisDownload,
    windowsX64MsiDownload,
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