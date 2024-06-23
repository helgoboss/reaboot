import {JSX} from "solid-js";

export function RecipeCode(props: { children: JSX.Element }) {
    return <pre class="!text-xs">
        <code>
            {props.children}
        </code>
    </pre>;
}