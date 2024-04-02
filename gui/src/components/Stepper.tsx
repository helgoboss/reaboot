import {Index} from "solid-js";
import {PageDescriptor, PageId} from "../model/page.ts";
import {mainStore} from "../globals.ts";

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
                        const isRandomlyAccessible = page().isRandomlyAccessible ?? true;
                        return (
                            <li
                                class="step"
                                classList={{
                                    "step-primary": currentPageIndex() >= index,
                                    "cursor-pointer": isRandomlyAccessible,
                                }}
                                onClick={isRandomlyAccessible ? () => mainStore.setCurrentPageId(page().id) : undefined}
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