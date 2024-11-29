import {} from "@tauri-apps/api";
import {extractRecipe, getOrEmptyRecord, ParsedRecipe} from "reaboot-commons/src/recipe-util.ts";
import {configureInstaller} from "./install.ts";
import {mainStore} from "../globals.ts";
import * as clipboard from "@tauri-apps/plugin-clipboard-manager"

export async function applyRecipeFromClipboard() {
    const text = await clipboard.readText();
    if (text == null) {
        throw new Error("Clipboard doesn't contain text");
    }
    await applyRecipeFromText(text);
}

export async function applyRecipeFromText(text: string) {
    const recipe = await extractRecipe(text);
    await configureInstaller({
        recipe: recipe.raw,
        selectedFeatures: getDefaultFeatureIdsFromRecipe(recipe)
    });
    mainStore.setParsedRecipe(recipe);
}


function getDefaultFeatureIdsFromRecipe(recipe: ParsedRecipe) {
    const defaults = [];
    const map = getOrEmptyRecord(recipe.features);
    for (const featureId in map) {
        const feature = map[featureId];
        if (feature.raw.default) {
            defaults.push(featureId);
        }
    }
    return defaults;
}