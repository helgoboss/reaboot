import {createResource, Match, Switch} from 'solid-js';
import {Params, useParams, useSearchParams} from "@solidjs/router";
import {Recipe} from "../../../core/bindings/Recipe";
import {tryExtractRecipe, tryParsePackageUrlFromRaw} from "../../../commons/src/recipe-util";
import {Tabs} from "@kobalte/core";
import {Welcome} from "../components/welcome";
import {Footer} from "../components/footer";
import {recipeNameIsSpecial} from "../util/recipe-util";
import {InstallViaReaboot} from "../components/install-via-reaboot";
import {InstallViaReapack} from "../components/install-via-reapack";

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

function displayRecipeHeading(recipe: Recipe) {
    return recipeNameIsSpecial(recipe.name) ? `"${recipe.name}"` : recipe.name;
}