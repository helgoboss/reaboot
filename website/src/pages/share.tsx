import {createMemo, createResource, createSignal, Match, Show, Switch} from "solid-js";
import {extractRecipe} from "reaboot-commons/src/recipe-util";
import {NormalPage} from "../components/normal-page";
import {CopyField} from "../components/copy-field";

const MAX_URL_LENGTH = 2000;

export function Share() {
    const [payload, setPayload] = createSignal("");
    const [recipeResource] = createResource(payload, extractRecipe);
    const features = createMemo(() => {
        if (recipeResource.state !== "ready") {
            return [];
        }
        return Object.values(recipeResource()!);
    });
    const installationUrl = () => createReabootInstallationUrl(payload());
    return <NormalPage>
        <div class="h-responsive-prose">
            <h1>Share a recipe via ReaBoot</h1>

            <h2>ReaBoot installation link builder</h2>

            <p>Let's create a link, so that you can easily share REAPER packages or complete REAPER distributions!</p>

            <h3>
                1. Enter the recipe, an URL to a recipe or just a package URL
            </h3>

            <p><a href="#explanations">Scroll down</a> to learn more about these terms.</p>

            <textarea class="textarea textarea-bordered h-56 font-mono text-xs w-full mb-3"
                      oninput={evt => setPayload(evt.currentTarget.value)}>
                    {payload()}
            </textarea>
            <Switch>
                <Match when={recipeResource.loading}>
                    <div>Loading...</div>
                </Match>
                <Match when={recipeResource.error}>
                    <div class="alert alert-error">
                        <div>
                            <div class="text-error-content">
                                You must enter a valid recipe, recipe URL or package URL!
                            </div>
                            <div class="text-xs">
                                <pre>{recipeResource.error.toString()}</pre>
                            </div>
                        </div>
                    </div>
                </Match>
                <Match when={recipeResource()}>
                    {recipe =>
                        <>
                            <div class="alert alert-success mb-3">
                                You are sharing a recipe named "{recipe().raw.name}" with&#32;
                                {recipe().requiredPackages.length} required packages and&#32;
                                {features().length} features.
                            </div>
                            <Show when={installationUrl().length > MAX_URL_LENGTH}>
                                <div class="alert alert-warning">
                                    The generated URL contains more than {MAX_URL_LENGTH} characters. This could become
                                    a problem in some browsers! If you want to be on the safe side, consider putting the
                                    recipe somewhere online
                                    (e.g. in a GitHub repository or as a GitHub Gist) and providing an URL to the raw
                                    content of that recipe.
                                </div>
                            </Show>

                            <h3>2. Give it a try</h3>
                            <a href={installationUrl()} target="_blank">Try it!</a>

                            <h3>3. Copy what you need</h3>
                            <div class="flex flex-wrap gap-4">
                                <CopyField text={installationUrl}>
                                    Raw installation URL
                                </CopyField>
                                <CopyField text={() => `[url=${installationUrl()}]Install ${recipe().raw.name}[/url]`}>
                                    REAPER forum link
                                </CopyField>
                                <CopyField
                                    text={() => `<a href="${installationUrl()}">Install ${recipe().raw.name}</a>`}>
                                    HTML link (for website embedding)
                                </CopyField>
                            </div>
                        </>
                    }
                </Match>
            </Switch>

            <h2 id="explanations">Explanations</h2>

            <h3 id="reaboot-recipe">ReaBoot Recipe</h3>
            <p>
                The installer can be customized with a so-called <em>recipe</em>. A recipe describes which
                packages
                should be installed and gives the installer a sort of branding.
            </p>
            <p>
                Here's a very simple example recipe in JSON format:
            </p>
            <pre>{`{
    "name": "ReaLearn",
    "author": "Helgoboss Projects",
    "website": "https://www.helgoboss.org/projects/realearn/",
    "required_packages": [
        "https://github.com/helgoboss/reaper-packages/raw/master/index.xml#p=Extensions/ReaLearn-x64&v=latest"
    ]
}`}</pre>
            <p>
                At startup, the installer checks whether the clipboard contains such a recipe. If yes, it
                pre-configures itself with the data in the recipe. As an alternative, it accepts a package URL or an URL
                that points to a recipe.
            </p>
            <p>Here's a more complicated recipe which demonstrates the use of optional features:</p>
            <pre>{`{
    "name": "Helgo's random tool collection",
    "description": "This is just an example recipe for ReaBoot in order to demonstrate how to share a complete collection of packages and make some of them optional.",
    "author": "helgoboss",
    "website": "https://www.reaboot.com/",
    "required_packages": [
        "https://raw.githubusercontent.com/ReaTeam/Extensions/master/index.xml#p=API/reaper_imgui.ext"
    ],
    "features": {
        "sws": {
            "name": "SWS/S&M Extension",
            "default": true,
            "description": "Popular and established extension that adds a variety of smaller features to REAPER. Considered as must-have by many.",
            "packages": [
                "https://raw.githubusercontent.com/ReaTeam/Extensions/master/index.xml#p=Extensions/reaper-oss_SWS.ext&v=latest-pre"
            ]
        },
        "libraries": {
            "name": "Common libraries",
            "description": "Frequently used libraries that provide functions for ReaScripts",
            "packages": [
                "https://github.com/Ultraschall/ultraschall-lua-api-for-reaper/raw/master/ultraschall_api_index.xml#p=Ultraschall-API-category/Ultraschall%20API%20package",
                "https://github.com/ReaTeam/ReaScripts/raw/master/index.xml#p=Development/Lokasenna_GUI%20library%20v2.lua",
                "https://github.com/ReaTeam/ReaScripts/raw/master/index.xml#p=Development/Lokasenna_Scythe%20library%20v3.lua"
            ]
        },
        "realearn": {
            "name": "ReaLearn",
            "description": "The \\"Swiss Army Knife\\" among the REAPER controller integration tools",
            "packages": [
                "https://github.com/helgoboss/reaper-packages/raw/master/index.xml#p=Extensions/ReaLearn-x64&v=latest"
            ]
        }
    }
}`}</pre>

            <h3 id="package-url">Package URL</h3>
            <p>
                A package URL is a URL that uniquely identifies a ReaPack package. It has the following structure:
            </p>
            <pre>
                {`REPOSITORY_URL#p=PACKAGE_CATEGORY/PACKAGE_NAME&v=VERSION_REF`}
            </pre>
            <p>
                <span class="font-mono">VERSION_REF</span> can be <span class="font-mono">latest</span>,&#32;
                <span class="font-mono">latest-pre</span> or a specific version name.
                This part is optional and defaults to <span class="font-mono">latest</span>.
            </p>
            <p>Examples:</p>
            <pre>{`
https://github.com/helgoboss/reaper-packages/raw/master/index.xml#p=Extensions/ReaLearn-x64
https://github.com/ReaTeam/ReaScripts/raw/master/index.xml#p=Various/rodilab_Color%20palette.lua&v=latest`}</pre>

        </div>
    </NormalPage>;
}

function createReabootInstallationUrl(payload: string): string {
    const encodedPayload = encodeURIComponent(minifyPayload(payload));
    const loc = window.location;
    return `${loc.protocol}//${loc.host}/install/${encodedPayload}`;
}

// Minifies the given payload if it's JSON. Important to keep the URL length small.
function minifyPayload(payload: string): string {
    try {
        const parsed = JSON.parse(payload);
        return JSON.stringify(parsed);
    } catch {
        return payload;
    }
}