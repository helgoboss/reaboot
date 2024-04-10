import {createMemo, For, Show} from "solid-js";
import {parsePackageUrl} from "../../../commons/src/recipe-util";
import {PackageUrl} from "../../../reapack/bindings/PackageUrl";
import {Recipe} from "../../../core/bindings/Recipe";
import {VersionRef} from "../../../reapack/bindings/VersionRef";

export function InstallViaReapack(props: { recipe: Recipe }) {
    const requiredPackageUrls = createMemo(() => {
        const required_packages = props.recipe.required_packages;
        if (!required_packages) {
            return [];
        }
        return required_packages.map(raw => parsePackageUrl(new URL(raw)))
    });
    const remotes = createMemo(() => [...new Set(requiredPackageUrls().map(u => u.repository_url))]);
    const nonDefaultRemotes = createMemo(() => remotes().filter(r => !defaultRemotes.has(r)));
    const needsRestart = () => requiredPackageUrls().some(p => p.package_version_ref.package_path.category === "Extensions");
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
            </li>
            <li>
                Install required packages
                <ReapackPackageInstalls urls={requiredPackageUrls()}/>
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

const defaultRemoteDescs = [
    "cfillion/reapack",
    "ReaTeam/ReaScripts",
    "ReaTeam/Extensions",
    "ReaTeam/JSFX",
    "ReaTeam/Themes",
    "ReaTeam/LangPacks",
    "MichaelPilyavskiy/ReaScripts",
    "X-Raym/REAPER-ReaScripts",
];

const defaultRemotes = new Set(
    defaultRemoteDescs.flatMap(desc => [
        `https://github.com/${desc}/raw/master/index.xml`,
        `https://raw.githubusercontent.com/${desc}/master/index.xml`,
    ])
);