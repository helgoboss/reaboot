import {Component} from "solid-js";

export type PageId = "welcome" | "pick-reaper" | "add-packages" | "install" | "done";

export type PageDescriptor = {
    id: PageId,
    title: string,
    content: Component,
    showFooter?: boolean,
}