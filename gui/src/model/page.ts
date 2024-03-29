import {Component} from "solid-js";

export type PageId = "welcome" | "pick-reaper" | "pick-bundles" | "install" | "done";

export type PageDescriptor = {
    id: PageId,
    title: string,
    description: string,
    content: Component,
    showNav?: boolean,
}