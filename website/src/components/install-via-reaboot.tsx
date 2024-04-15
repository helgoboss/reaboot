import {copyTextToClipboard} from "../util/clipboard-util";
import {Step} from "./step";
import {For, JSX, Match, Switch} from "solid-js";
import {FaSolidDownload} from "solid-icons/fa";
import {Collapsible} from "@kobalte/core";
import {CopyField} from "./copy-field";
import {UAParser} from "ua-parser-js";
import {ParsedRecipe} from "reaboot-commons/src/recipe-util";
import {RecipeRef} from "./recipe-ref";

const LATEST_REABOOT_VERSION = "0.2.0";

export function InstallViaReaboot(props: { recipe: ParsedRecipe }) {
    const downloadConfig = getDownloadConfig();
    const otherDownloads = reabootDownloads.filter(d => {
        return downloadConfig.mainDownloads.every(optimalDownload => d.label !== optimalDownload?.label);
    });
    const getRecipeAsJson = () => JSON.stringify(props.recipe.raw, null, "    ");
    const copyRecipeMain = async () => {
        await copyTextToClipboard(getRecipeAsJson());
    };

    return <div class="grow flex flex-col max-w-lg items-stretch gap-6">
        <div class="text-center">
            ReaBoot is the easiest way to install <RecipeRef recipe={props.recipe}/>.
            It automatically installs REAPER and ReaPack if necessary.
        </div>
        <Step index={0} title="Download ReaBoot">
            <div>
                {downloadConfig.downloadComment}
            </div>
            <Switch>
                <Match when={downloadConfig.mainDownloads.length > 0}>
                    <>
                        <div class="flex flex-row justify-center gap-2">
                            <For each={downloadConfig.mainDownloads}>
                                {(d, i) =>
                                    <div class="indicator">
                                        {i() === 0 && downloadConfig.recommendFirstDownload &&
                                            <span class="indicator-item badge badge-secondary">typing…</span>
                                        }
                                        <a href={buildDownloadUrl(d)}
                                           onclick={() => copyRecipeMain()}
                                           class="btn btn-accent">
                                            <FaSolidDownload/>
                                            {d.label}
                                        </a>
                                    </div>
                                }
                            </For>
                        </div>
                    </>
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

type ReabootDownloadConfig = {
    downloadComment: JSX.Element,
    mainDownloads: ReabootDownload[],
    recommendFirstDownload: boolean,
}

type ReabootDownload = {
    label: string,
    asset: string,
}

function getDownloadConfig(): ReabootDownloadConfig {
    // console.log(UA_PARSER_RESULT.cpu.architecture);
    switch (UA_PARSER_RESULT.os.name) {
        case "Mac OS":
            switch (UA_PARSER_RESULT.cpu.architecture) {
                case "arm64":
                    return {
                        downloadComment: <>For your system:</>,
                        mainDownloads: [macOsArm64Download, macOsX86_64Download],
                        recommendFirstDownload: true,
                    };
                default:
                    return {
                        downloadComment: <>Requires at least macOS 10.13</>,
                        mainDownloads: [macOsX86_64Download, macOsArm64Download],
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
                            probably have to&#32;
                            <a class="link" href="https://go.microsoft.com/fwlink/p/?LinkId=2124703">
                                install the Microsoft Edge WebView2 runtime
                            </a>
                            first, otherwise ReaBoot will not work.
                        </>,
                        mainDownloads: [windowsX64NsisDownload, windowsX64ExeDownload],
                        recommendFirstDownload: true,
                    };
                case "11":
                    return {
                        downloadComment: <>
                            For your system:
                        </>,
                        mainDownloads: [windowsX64ExeDownload, windowsX64MsiDownload],
                        recommendFirstDownload: true,
                    };
                // Windows 10 or not detected
                default:
                    return {
                        downloadComment: <>
                            If the portable download doesn't work, either use the installer or first&#32;
                            <a class="link" href="https://go.microsoft.com/fwlink/p/?LinkId=2124703">
                                install the Microsoft Edge WebView2 runtime
                            </a>!
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

const macOsArm64Download = {
    label: "macOS ARM64",
    asset: "reaboot-macos-arm64.zip",
};
const macOsX86_64Download = {
    label: "macOS Intel",
    asset: "reaboot-macos-x86_64.zip"
};
const linuxX86_64Download = {
    label: "Linux x86_64 (deb)",
    asset: "reaboot-linux-x86_64.deb",
};
const windowsX64ExeDownload = {
    label: "Windows x64 (Portable)",
    asset: "reaboot-windows-x64.exe",
};
const windowsX64NsisDownload = {
    label: "Windows x64 (NSIS Installer)",
    asset: "reaboot-windows-x64-setup.exe",
};
const windowsX64MsiDownload = {
    label: "Windows x64 (MSI Installer)",
    asset: "reaboot-windows-x64-setup.msi",
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