import {Toast, toaster} from "@kobalte/core";
import {FaSolidX} from "solid-icons/fa";

export function showToast(clazz: string, message: string) {
    toaster.show(props => (
        <Toast.Root toastId={props.toastId} class={`alert ${clazz}`} duration={8000}>
            <div class="flex flex-row justify-between">
                <div>
                    <Toast.Description>
                        {message}
                    </Toast.Description>
                </div>
                <Toast.CloseButton class="ml-5">
                    <FaSolidX/>
                </Toast.CloseButton>
            </div>
        </Toast.Root>
    ));
}