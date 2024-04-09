import {A} from "@solidjs/router";
import {FaBrandsGithub} from "solid-icons/fa";

export function Footer() {
    return <footer class="footer items-center p-4 bg-base-300">
        <A href="/faq" class="link">FAQ</A>
        <nav class="grid-flow-col gap-4 md:place-self-center md:justify-self-end">
            <a href="https://github.com/helgoboss/reaboot">
                <FaBrandsGithub/>
            </a>
        </nav>
    </footer>;
}