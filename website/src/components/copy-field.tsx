import {createSignal, JSX} from "solid-js";
import {copyTextToClipboard} from "../util/clipboard-util";
import {FaRegularClipboard, FaSolidCheck} from "solid-icons/fa";

type Props = {
    text: () => string,
    children: JSX.Element,
}

export function CopyField(props: Props) {
    const [copied, setCopied] = createSignal(false);
    const copyText = async () => setCopied(await copyTextToClipboard(props.text()));
    return <button class="btn btn-xs btn-accent" onclick={() => copyText()}>
        {props.children}
        {copied() && <FaSolidCheck/> || <FaRegularClipboard/>}
    </button>
}