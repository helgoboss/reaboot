import {Component} from "solid-js";

export type PageId = "welcome" | "pick-reaper" | "customize" | "install" | "done";

export type PageDescriptor = {
    // Page ID.
    id: PageId,
    // Page title, currently only shown in stepper.
    title: string,
    // Page content component.
    content: Component,
    // Whether the footer should be shown when this page is displayed.
    showFooter?: boolean,
}