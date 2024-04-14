import {createResource, Match, Switch} from 'solid-js';
import {Params, useParams, useSearchParams} from "@solidjs/router";
import {extractRecipe, ParsedRecipe} from "reaboot-commons/src/recipe-util";
import {Tabs} from "@kobalte/core";
import {recipeNameIsSpecial} from "../util/recipe-util";
import {InstallViaReaboot} from "../components/install-via-reaboot";
import {InstallViaReapack} from "../components/install-via-reapack";
import {Page} from "../components/page";

export default function Install() {
    const params = useParams();
    const [searchParams, setSearchParams] = useSearchParams();
    const [recipeResource] = createResource(params, getRecipeFromParams);

    const via = () => searchParams.via ?? "reaboot";

    const setVia = (via: string) => {
        setSearchParams({
            via
        });
    };

    return (
        <Page disableHeader={true} enablePoweredBy={true}>
            <div class="grow flex flex-col">
                <Switch>
                    <Match when={recipeResource.loading}>
                        <span class="loading loading-ball loading-md"/>
                    </Match>
                    <Match when={recipeResource()}>
                        {recipe => <>
                            <h1 class="text-center text-xl lg:text-3xl font-bold">
                                Let's install {displayRecipeHeading(recipe())}!
                            </h1>
                            <Tabs.Root value={via()} onChange={setVia} class="flex flex-col sm:items-center">
                                <Tabs.List class="tabs tabs-boxed m-4 self-center">
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
        </Page>
    );
}

async function getRecipeFromParams(params: Partial<Params>): Promise<ParsedRecipe> {
    const thing = params.thing;
    if (!thing) {
        throw new Error("The URL is missing information about what to install!");
    }
    const decodedThing = decodeURIComponent(thing);
    return extractRecipe(decodedThing);
}

function displayRecipeHeading(recipe: ParsedRecipe) {
    return recipeNameIsSpecial(recipe.raw.name) ? `"${recipe.raw.name}"` : recipe.raw.name;
}