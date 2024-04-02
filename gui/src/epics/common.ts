import {toast} from "solid-toast";

export function showError(message: any) {
    toast.error(message);
}