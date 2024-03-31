import {Component} from "solid-js";

export type PageId = "welcome" | "pick-reaper" | "pick-packages" | "install" | "done";

export type PageDescriptor = {
    id: PageId,
    title: string,
    description: string,
    content: Component,
    showFooter?: boolean,
}