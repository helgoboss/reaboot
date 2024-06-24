import {Index} from "solid-js";
import {PageDescriptor, PageId} from "../model/page.ts";
import {navigateTo} from "../epics/common.tsx";

export type StepperProps = {
    currentPageId?: PageId,
    pages: PageDescriptor[]
}


export function Stepper(props: StepperProps) {
    const currentPageIndex = () => props.pages.findIndex(p => p.id == props.currentPageId);
    return (
        <ol class="steps w-full">
            <Index each={props.pages}>
                {
                    (page, index) => {
                        return (
                            <li
                                class="step cursor-pointer"
                                classList={{
                                    "step-primary": currentPageIndex() >= index,
                                }}
                                onClick={() => navigateTo(page().id)}
                            >
                                {page().title}
                            </li>
                        );
                    }
                }
            </Index>
        </ol>
    );
}