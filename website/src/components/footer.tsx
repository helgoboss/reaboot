import {A} from "@solidjs/router";
import {FaBrandsGithub} from "solid-icons/fa";

export function Footer() {
    return <footer class="footer items-center p-4 bg-base-300">
        <aside class="items-center grid-flow-col">
            <p>Copyright © 2024 <a href="https://www.helgoboss.org/" class="link">Helgoboss Projects</a></p>
        </aside>
        <nav class="grid-flow-col gap-4 md:place-self-center md:justify-self-end items-center">
            <A href="/faq" class="link">FAQ</A>
            <a href="https://github.com/helgoboss/reaboot">
                <FaBrandsGithub/>
            </a>
        </nav>
    </footer>;
}