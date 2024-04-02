import {Index} from "solid-js";
import {PageId} from "../model/page.ts";
import {mainStore} from "../globals.ts";

export type StepperProps = {
    currentPageId?: PageId,
    pages: StepperPage[]
}

export type StepperPage = {
    id: PageId,
    title: string,
}

export function Stepper(props: StepperProps) {
    const currentPageIndex = () => props.pages.findIndex(p => p.id == props.currentPageId);
    return (
        <ol class="steps w-full">
            <Index each={props.pages}>
                {
                    (page, index) => (
                        <li
                            class="step cursor-pointer"
                            classList={{"step-primary": currentPageIndex() >= index}}
                            onClick={() => mainStore.setCurrentPageId(page().id)}
                        >
                            {page().title}
                        </li>
                    )
                }
            </Index>
        </ol>
    );
}