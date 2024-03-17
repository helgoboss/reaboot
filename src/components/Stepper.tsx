import { Index } from "solid-js";
import { PageId } from "../model/page.ts";
import {mainStore} from "../services/globals.ts";

export type StepperProps = {
    currentPageId?: PageId,
    pages: StepperPage[]
}

export type StepperPage = {
    id: PageId,
    title: string,
}

export function Stepper(props: StepperProps) {
    return (
        <ol class="flex w-full text-sm font-medium text-center text-gray-500 dark:text-gray-400 sm:text-base items-stretch">
            <Index each={props.pages}>
                {
                    (page, index) => (
                        <li
                            class="flex md:w-full items-center after:content-[''] after:w-full after:h-1 after:border-b after:border-gray-200 after:border-1 after:hidden sm:after:inline-block after:mx-6 xl:after:mx-10 dark:after:border-gray-700 cursor-pointer"
                            classList={{ "bg-amber-200": page().id == props.currentPageId }}
                            onClick={() => mainStore.openPage(page().id)}
                        >
                            <span
                                class="flex items-center after:content-['/'] sm:after:hidden after:mx-2 after:text-gray-200 dark:after:text-gray-500">
                                <span class="me-2">{index + 1}</span>
                                {page().title}
                            </span>
                        </li>
                    )
                }
            </Index>
        </ol>
    );
}