import {JSX} from "solid-js";

type PageProps = {
    title: string,
    description: string,
    children?: JSX.Element
}

function Page(props: PageProps) {
    return (
        <div class="flex flex-col text-center justify-center">
            <h1 class="flex-none text-3xl font-bold">{props.title}</h1>
            <div>{props.description}</div>
            <div class="grow bg-slate-200 m-12 p-6">
                {props.children}
            </div>
        </div>
    );
}

export default Page;