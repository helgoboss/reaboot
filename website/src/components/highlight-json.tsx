import {JSX} from "solid-js";

export function HighlightJson(props: { children: JSX.Element }) {
    return <pre class="!text-xs">
        <code>
            {props.children}
        </code>
    </pre>;
}