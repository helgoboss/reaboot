import {JSX} from "solid-js";

export type PageProps = {
    // title: string,
    // description: string,
    children?: JSX.Element
}

export function Page(props: PageProps) {
    return (
        <div class="grow flex flex-col px-6 min-h-0">
            {props.children}
        </div>
    );
}