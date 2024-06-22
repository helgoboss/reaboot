import {formatRecipeAsJson, ParsedRecipe} from "reaboot-commons/src/recipe-util";
import {RecipeRef} from "./recipe-ref";
import {CopyField} from "./copy-field";
import {HighlightJson} from "./highlight-json";

export function ShowRecipe(props: { recipe: ParsedRecipe }) {
    return <div class="h-responsive-prose">
        <div class="text-center">
            Below you can see the ReaBoot recipe for installing
            <RecipeRef recipe={props.recipe}/>. You can inspect it, use it as a starting
            point for your own recipe or just copy and paste it into ReaBoot.
        </div>
        <div class="text-right">
            <CopyField text={() => formatRecipeAsJson(props.recipe.raw)}>Copy recipe to clipboard</CopyField>
        </div>
        <HighlightJson>
            {formatRecipeAsJson(props.recipe.raw)}
        </HighlightJson>

    </div>
}