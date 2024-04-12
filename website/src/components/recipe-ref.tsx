import {ParsedRecipe} from "reaboot-commons/src/recipe-util";
import {recipeNameIsSpecial} from "../util/recipe-util";

export function RecipeRef({recipe}: { recipe: ParsedRecipe }) {
    if (recipeNameIsSpecial(recipe.raw.name)) {
        return <span class="badge">{recipe.raw.name}</span>;
    } else {
        return recipe.raw.name;
    }
}