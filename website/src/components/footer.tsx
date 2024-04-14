import {A} from "@solidjs/router";
import {FaBrandsGithub} from "solid-icons/fa";

type Props = {
    enablePoweredBy?: boolean,
}

export function Footer(props: Props) {
    return <footer class="text-xs p-4 bg-base-300 flex flex-row gap-4 items-center"
                   classList={{
                       "justify-between": props.enablePoweredBy,
                       "justify-center sm:justify-between": !props.enablePoweredBy,
                   }}>
        <aside class="hidden sm:block">
            <p>© 2024 <a href="https://www.helgoboss.org/" class="link">Helgoboss Projects</a></p>
        </aside>
        {props.enablePoweredBy && <h1>
            <span class="italic mr-2">Powered by</span>
            <A href="/" class="text-lg font-bold link">ReaBoot</A>
        </h1>
        }
        <div class="flex flex-row gap-4">
            <A href="/faq" class="link">FAQ</A>
            <A href="/beta" class="card h-warning px-4">Beta</A>
            <a href="https://github.com/helgoboss/reaboot">
                <FaBrandsGithub/>
            </a>
        </div>
    </footer>;
}