import {A} from "@solidjs/router";
import {FaBrandsGithub} from "solid-icons/fa";

type Props = {
    poweredBy: boolean,
}

export function Footer(props: Props) {
    return <footer class="text-xs p-4 bg-base-300 flex flex-row gap-4 justify-between items-center">
        <aside class="hidden sm:block">
            <p>© 2024 <a href="https://www.helgoboss.org/" class="link">Helgoboss Projects</a></p>
        </aside>
        {props.poweredBy && <h1>
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