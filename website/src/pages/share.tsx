import {createResource, createSignal, Match, Switch} from "solid-js";
import {tryExtractRecipe} from "../../../commons/src/recipe-util";

export default function Share() {
    const [payload, setPayload] = createSignal("");
    const [recipeResource] = createResource(payload, tryExtractRecipe);
    const installationUrl = () => createReabootInstallationUrl(payload());
    return (
        <div class="max-w-3xl prose">
            <h1>How to share recipes via ReaBoot</h1>

            <h2>ReaBoot Installation URL builder</h2>

            Let's create a <a href="#reaboot-installation-url">ReaBoot installation URL</a>, so that you can easily
            share REAPER packages or complete REAPER distributions!

            <h3>
                1. Enter the <a href="#reaboot-recipe">recipe</a>, an URL to a recipe or just a <a href="#package-url">package
                URL</a>
            </h3>

            <div>
                <textarea class="textarea textarea-bordered h-56 font-mono text-xs w-full"
                          oninput={evt => setPayload(evt.currentTarget.value)}>
                    {payload()}
                </textarea>
            </div>


            <h3>
                2. Copy what you need
            </h3>
            <div>
                Raw installation URL:
                <pre>{installationUrl()}</pre>
            </div>
            <Switch>
                <Match when={recipeResource()}>
                    {recipe =>
                        <div>
                            REAPER forum link:
                            <pre>[url={installationUrl()}]{recipe().name}[/url]</pre>
                        </div>
                    }
                </Match>
                <Match when={true}>
                    <div class="alert alert-error">
                        Looks like you haven't entered a valid recipe, recipe URL or package URL!
                    </div>
                </Match>
            </Switch>
            <a href={installationUrl()} target="_blank">Try it!</a>

            <h2 id="glossary">Glossary</h2>

            <h3 id="package-url">Package URL</h3>
            <p>
                The installer has multiple places where it accepts so-called <em>package URLs</em>.
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

            <h3 id="reaboot-recipe">ReaBoot Recipe</h3>
            <p>
                The installer can be customized with a so-called <em>recipe</em>. A recipe mainly describes which
                packages
                should be installed and gives the installer a sort of branding.
            </p>
            <p>
                Here's an example recipe in JSON format:
            </p>
            <pre>{`{
    "name": "ReaLearn",
    "website": "https://www.helgoboss.org/projects/realearn/",
    "manufacturer": "Helgoboss Projects",
    "logo": "https://www.helgoboss.org/projects/realearn/slide.png",
    "package_urls": [
        "https://github.com/helgoboss/reaper-packages/raw/master/index.xml#p=Extensions/ReaLearn-x64&v=latest"
    ]
}`}</pre>
            <p>
                At startup, the installer checks whether the clipboard contains such a recipe. If yes, it
                pre-configures itself with the data in the recipe. It also accepts a package URL or an URL
                that points to a recipe.
            </p>

            <h3 id="reaboot-installation-url">ReaBoot Installation URL</h3>
            <p>
                This is the URL you share with your users. When clicking on a link to such a URL, the user ends
                up on this website and will be prompted to download the installer. The download button doesn't just
                download the installer, it also copies the recipe, recipe URL or package URL to the clipboard.
            </p>
        </div>
    );
}

function createReabootInstallationUrl(payload: string): string {
    const encodedPayload = encodeURIComponent(payload);
    const loc = window.location;
    return `${loc.protocol}//${loc.host}/install/${encodedPayload}`;
}