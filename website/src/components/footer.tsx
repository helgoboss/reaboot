import {A} from "@solidjs/router";
import {FaBrandsGithub} from "solid-icons/fa";

export function Footer() {
    return <footer class="text-xs p-4 bg-base-300 flex flex-row gap-4 justify-center items-center">
        <aside class="grow hidden sm:block">
            <p>© 2024 <a href="https://www.helgoboss.org/" class="link">Helgoboss Projects</a></p>
        </aside>
        <A href="/faq" class="link">FAQ</A>
        <A href="/beta" class="card bg-warning text-warning-content px-4">Beta</A>
        <a href="https://github.com/helgoboss/reaboot">
            <FaBrandsGithub/>
        </a>
    </footer>;
}