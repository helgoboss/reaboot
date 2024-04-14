import {extractRecipe, ParsedRecipe} from "reaboot-commons/src/recipe-util";

const SPECIAL_CHAR_REGEX = new RegExp("[^A-Za-z0-9]");

export function recipeNameIsSpecial(name: string) {
    return SPECIAL_CHAR_REGEX.test(name);
}

export function deconstructRecipe(rawInstallationUrl: string): Promise<ParsedRecipe> {
    const url = new URL(rawInstallationUrl);
    if (!url.pathname.startsWith("/install/")) {
        throw new Error("This is not a valid ReaBoot installation link.");
    }
    const urlEncodedPayload = url.pathname.substring(9);
    const payload = decodeURIComponent(urlEncodedPayload);
    return extractRecipe(payload);
}