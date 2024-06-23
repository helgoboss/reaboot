import {formatRecipeAsJson, ParsedRecipe} from "reaboot-commons/src/recipe-util";
import {RecipeRef} from "./recipe-ref";
import {CopyField} from "./copy-field";
import {RecipeCode} from "./recipe-code";

export function ShowRecipe(props: { recipe: ParsedRecipe }) {
    return <div class="h-responsive-prose">
        <div class="text-center">
            Here is the ReaBoot recipe for installing&#32;<RecipeRef recipe={props.recipe}/>.
            You can inspect it, use it as a starting
            point for your own recipe or just copy and paste it into ReaBoot.
        </div>
        <div class="text-right">
            <CopyField text={() => formatRecipeAsJson(props.recipe.raw)}>Copy recipe to clipboard</CopyField>
        </div>
        <RecipeCode>
            {formatRecipeAsJson(props.recipe.raw)}
        </RecipeCode>

    </div>
}