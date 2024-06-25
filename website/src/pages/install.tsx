import {createMemo, createResource, For, Index, Match, Show, Switch} from 'solid-js';
import {Params, useParams, useSearchParams} from "@solidjs/router";
import {extractRecipe, ParsedRecipe} from "reaboot-commons/src/recipe-util";
import {Tabs} from "@kobalte/core";
import {recipeNameIsSpecial} from "../util/recipe-util";
import {InstallViaReaboot} from "../components/install-via-reaboot";
import {InstallViaReapack} from "../components/install-via-reapack";
import {Page} from "../components/page";
import {ShowRecipe} from "../components/show-recipe";

export default function Install() {
    const params = useParams();
    const [recipeResource] = createResource(params, getRecipeFromParams);

    return (
        <Page disableHeader={true} enablePoweredBy={true}>
            <div class="grow flex flex-col">
                <Switch>
                    <Match when={recipeResource.loading}>
                        <span class="loading loading-ball loading-md"/>
                    </Match>
                    <Match when={recipeResource()}>
                        {recipe => <InstallInternal recipe={recipe()}/>}
                    </Match>
                </Switch>
            </div>
        </Page>
    );
}

function InstallInternal({recipe}: { recipe: ParsedRecipe }) {
    const [searchParams, setSearchParams] = useSearchParams();
    const features = createMemo(() => Object.values(recipe.features));
    const via = () => searchParams.via ?? "reaboot";

    const setVia = (via: string) => {
        setSearchParams({
            via
        });
    };

    return <>
        <h1 class="text-center text-xl lg:text-3xl font-bold">
            Let's install {displayRecipeHeading(recipe)}!
        </h1>
        <Show when={features().length > 0}>
            <ul class="mt-2 flex flex-row justify-center gap-2 overflow-x-auto horizontal-list">
                <For each={features()}>
                    {feature =>
                        <li
                            class="text-sm whitespace-nowrap opacity-40"
                            title={feature.raw.description ?? ""}>
                            {feature.raw.name}
                        </li>
                    }
                </For>
            </ul>
        </Show>
        <Tabs.Root value={via()} onChange={setVia} class="flex flex-col sm:items-center">
            <Tabs.List class="tabs tabs-boxed m-4 self-center">
                <Tabs.Trigger value="reaboot" class="tab data-[selected]:tab-active !h-auto">
                    Via ReaBoot
                </Tabs.Trigger>
                <Tabs.Trigger value="reapack" class="tab data-[selected]:tab-active !h-auto">
                    Via ReaPack
                </Tabs.Trigger>
                <Tabs.Trigger value="recipe" class="tab data-[selected]:tab-active !h-auto">
                    Show recipe
                </Tabs.Trigger>
            </Tabs.List>
            <Tabs.Content value="reaboot">
                <InstallViaReaboot recipe={recipe}/>
            </Tabs.Content>
            <Tabs.Content value="reapack">
                <InstallViaReapack recipe={recipe}/>
            </Tabs.Content>
            <Tabs.Content value="recipe">
                <ShowRecipe recipe={recipe}/>
            </Tabs.Content>
        </Tabs.Root>
    </>
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