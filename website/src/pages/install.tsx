import {createMemo, createResource, For, Match, Show, Switch} from 'solid-js';
import {Step} from "../components/step";
import {FaRegularClipboard, FaSolidDownload} from "solid-icons/fa";
import {UAParser} from 'ua-parser-js';
import {Params, useParams, useSearchParams} from "@solidjs/router";
import {Recipe} from "../../../core/bindings/Recipe";
import {tryExtractRecipe, tryParsePackageUrlFromRaw} from "../../../commons/src/recipe-util";
import {PackageUrl} from "../../../reapack/bindings/PackageUrl";
import {VersionRef} from "../../../reapack/bindings/VersionRef";

export default function Install() {
    const params = useParams();
    const [searchParams, setSearchParams] = useSearchParams();
    const [recipeResource] = createResource(params, tryGetRecipeFromParams);

    function via() {
        return searchParams.via;
    }

    function setVia(via: string) {
        setSearchParams({
            via
        });
    }

    return (
        <div class="grow flex flex-col">
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
                            <a role="tab" class="tab tab-active" onClick={() => setVia("reaboot")}>Via ReaBoot</a>
                            <a role="tab" class="tab" onClick={() => setVia("reapack")}>Via ReaPack</a>
                        </div>
                        <Switch>
                            <Match when={searchParams.via === "reapack"}>
                                <InstallViaReapack recipe={recipe()}/>
                            </Match>
                            <Match when={true}>
                                <InstallViaReaboot recipe={recipe()}/>
                            </Match>
                        </Switch>
                    </>
                    }
                </Match>
            </Switch>
        </div>
    );
}

type InstallViaProps = {
    recipe: Recipe,
}

function InstallViaReapack(props: InstallViaProps) {
    const packageUrls = createMemo(() => {
        return props.recipe.package_urls
            .map(tryParsePackageUrlFromRaw)
            .filter(u => u !== null) as PackageUrl[]
    });
    const remotes = createMemo(() => [...new Set(packageUrls().map(u => u.repository_url))]);
    const nonDefaultRemotes = createMemo(() => remotes().filter(r => !defaultRemotes.has(r)));
    const needsRestart = () => packageUrls().some(p => p.package_version_ref.package_path.category === "Extensions");
    return <div class="prose">
        <h3>
            If REAPER is not installed yet:
        </h3>
        <ol>
            <li>
                Find the correct download on the&#32;
                <a href="https://www.reaper.fm/download.php" target="_blank">REAPER website</a> and download it
            </li>
            <li>
                Execute the downloaded installer or DMG and follow the instructions
            </li>
            <li>
                Start REAPER at least once
            </li>
        </ol>
        <h3>
            If ReaPack is not installed yet:
        </h3>
        <ol>
            <li>
                Find the correct download on the&#32;
                <a href="https://reapack.com/" target="_blank">ReaPack website</a> and download it
            </li>
            <li>
                Open the file manager and copy the downloaded ReaPack shared library to the correct location within the
                REAPER resource folder (REAPER → Options → Show REAPER resource path in explorer/finder...)
            </li>
            <li>
                Restart REAPER
            </li>
        </ol>
        <h3>
            Always:
        </h3>
        <ol>
            <Show when={nonDefaultRemotes().length > 0}>
                <li>
                    REAPER → Extensions → ReaPack → Import repositories...
                </li>
                <li>
                    Paste the following text and press OK:
                    <pre>
                        <For each={nonDefaultRemotes()}>
                            {r => r}
                        </For>
                    </pre>
                </li>
            </Show>
            <li>
                REAPER → Extensions → ReaPack → Browse packages...
                <ul>
                    <For each={packageUrls()}>
                        {purl =>
                            <li>
                                Search for&#32;
                                <span class="font-mono">
                                    {purl.package_version_ref.package_path.package_name}
                                </span>,
                                right-click the corresponding package and
                                choose&#32;
                                <em>{getReapackMenuEntry(purl.package_version_ref.version_ref)}</em>.
                            </li>
                        }
                    </For>
                </ul>
            </li>
            <li>
                Press OK
            </li>
            <Show when={needsRestart()}>
                <li>
                    Restart REAPER (because an extension was installed)
                </li>
            </Show>
        </ol>
    </div>
}

function getReapackMenuEntry(versionRef: VersionRef): string {
    switch (versionRef) {
        case "latest":
            return "Install v...";
        case "latest-pre":
            return "Versions → First version in the menu";
        default:
            return `Versions → ${versionRef};`
    }
}

function InstallViaReaboot(props: InstallViaProps) {
    const primaryDownload = getOptimalReabootDownload();
    const otherDownloads = reabootDownloads.filter(d => d.label !== primaryDownload?.label);
    return <div class="grow flex flex-col max-w-lg items-stretch gap-6">
        <div class="text-center">
            ReaBoot is the easiest way to install {displayRecipeNormal(props.recipe)}.
            It automatically installs REAPER and ReaPack, if you don't have those yet!
        </div>
        <Step index={0} title="Download ReaBoot">
            <div>
                Here's the download matching your current system:
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
                            Doesn't work on your system?
                        </dt>
                        <dd>
                            It's possible that your system is not modern enough to run ReaBoot.
                            Try installation via ReaPack instead!
                        </dd>
                    </dl>
                </div>
            </div>
        </Step>
    </div>
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

const defaultRemotes = new Set([
    "https://reapack.com/index.xml",
    "https://github.com/ReaTeam/ReaScripts/raw/master/index.xml",
    "https://github.com/ReaTeam/JSFX/raw/master/index.xml",
    "https://github.com/ReaTeam/Themes/raw/master/index.xml",
    "https://github.com/ReaTeam/LangPacks/raw/master/index.xml",
    "https://github.com/ReaTeam/Extensions/raw/master/index.xml",
    "https://github.com/MichaelPilyavskiy/ReaScripts/raw/master/index.xml",
    "https://github.com/X-Raym/REAPER-ReaScripts/raw/master/index.xml",
]);