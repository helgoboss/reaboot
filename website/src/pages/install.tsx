import {createMemo, createResource, createSignal, For, Match, Show, Switch} from 'solid-js';
import {Step} from "../components/step";
import {FaSolidCheck, FaSolidDownload} from "solid-icons/fa";
import {UAParser} from 'ua-parser-js';
import {Params, useParams, useSearchParams} from "@solidjs/router";
import {Recipe} from "../../../core/bindings/Recipe";
import {tryExtractRecipe, tryParsePackageUrlFromRaw} from "../../../commons/src/recipe-util";
import {PackageUrl} from "../../../reapack/bindings/PackageUrl";
import {VersionRef} from "../../../reapack/bindings/VersionRef";
import {Collapsible, Tabs} from "@kobalte/core";
import {copyTextToClipboard} from "../util/clipboard-util";
import {CopyField} from "../components/copy-field";
import {Welcome} from "../components/welcome";
import {Footer} from "../components/footer";

export default function Install() {
    const params = useParams();
    const [searchParams, setSearchParams] = useSearchParams();
    const [recipeResource] = createResource(params, tryGetRecipeFromParams);

    const via = () => searchParams.via ?? "reaboot";

    const setVia = (via: string) => {
        setSearchParams({
            via
        });
    };

    return (
        <div class="w-screen h-screen flex flex-row">
            <main class="grow flex flex-col items-center p-6 overflow-y-auto">
                <div class="grow flex flex-col">
                    <Switch>
                        <Match when={recipeResource.loading}>
                            <span class="loading loading-ball loading-md"/>
                        </Match>
                        <Match when={recipeResource()}>
                            {recipe => <>
                                <h1 class="text-center text-4xl font-bold">
                                    Let's install {displayRecipeHeading(recipe())}!
                                </h1>
                                <Tabs.Root value={via()} onChange={setVia} class="flex flex-col items-center">
                                    <Tabs.List class="tabs tabs-boxed m-4">
                                        <Tabs.Trigger value="reaboot" class="tab data-[selected]:tab-active">
                                            Via ReaBoot
                                        </Tabs.Trigger>
                                        <Tabs.Trigger value="reapack" class="tab data-[selected]:tab-active">
                                            Via ReaPack
                                        </Tabs.Trigger>
                                    </Tabs.List>
                                    <Tabs.Content value="reaboot">
                                        <InstallViaReaboot recipe={recipe()}/>
                                    </Tabs.Content>
                                    <Tabs.Content value="reapack">
                                        <InstallViaReapack recipe={recipe()}/>
                                    </Tabs.Content>
                                </Tabs.Root>
                            </>
                            }
                        </Match>
                    </Switch>
                </div>
            </main>
            <header class="max-w-sm bg-base-200 flex flex-col overflow-y-auto">
                <Welcome poweredBy={true} examples={false}/>
                <Footer/>
            </header>
        </div>
    );
}

type InstallViaProps = {
    recipe: Recipe,
}

function InstallViaReapack(props: InstallViaProps) {
    const packageUrls = createMemo(() => {
        const required_packages = props.recipe.required_packages;
        if (!required_packages) {
            return [];
        }
        return required_packages
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
                            <>
                                <li>
                                    Search for&#32;
                                    <span class="font-mono">
                                    {purl.package_version_ref.package_path.package_name}
                                </span> or something similar-sounding (the package description might be different
                                    from the package name)
                                </li>
                                <li>
                                    Right-click the corresponding package and
                                    choose&#32;
                                    <em>{getReapackMenuEntry(purl.package_version_ref.version_ref)}</em>
                                </li>
                            </>
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

function ReapackPackageInstalls(props: { urls: PackageUrl[] }) {
    return <ul>
        <For each={props.urls}>
            {purl =>
                <>
                    <li>
                        Search for&#32;
                        <span class="font-mono">
                                    {purl.package_version_ref.package_path.package_name}
                                </span> or something similar-sounding (the package description might be different
                        from the package name)
                    </li>
                    <li>
                        Right-click the corresponding package and
                        choose&#32;
                        <em>{getReapackMenuEntry(purl.package_version_ref.version_ref)}</em>
                    </li>
                </>
            }
        </For>
    </ul>;
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