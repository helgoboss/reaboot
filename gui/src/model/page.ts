import {Component} from "solid-js";

export type PageId = "welcome" | "pick-reaper" | "add-packages" | "install" | "done";

export type PageDescriptor = {
    // Page ID.
    id: PageId,
    // Page title, currently only shown in stepper.
    title: string,
    // Page content component.
    content: Component,
    // Whether the footer should be shown when this page is displayed.
    showFooter?: boolean,
    // Whether user can directly jump to that page using the stepper.
    isRandomlyAccessible?: boolean,
    // Whether a precondition for displaying this page is that the user agreed to the
    // REAPER EULA (only if REAPER not installed yet).
    requiresReaperEulaAgreement: boolean,
}