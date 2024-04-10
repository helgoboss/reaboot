export function recipeNameIsSpecial(name: string) {
    return SPECIAL_CHAR_REGEX.test(name);
}

const SPECIAL_CHAR_REGEX = new RegExp("[^A-Za-z0-9]");