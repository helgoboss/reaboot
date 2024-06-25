import {Help} from "./Help";
import {ReaperRef} from "./ReaperRef";
import {ReapackRef} from "./ReapackRef";

export function ReabootDescription(props: { linkToReabootWebsite: boolean }) {
    return <p>
        {props.linkToReabootWebsite ?
            <a href="https://www.reaboot.com/" target="_blank" class="link">ReaBoot</a> : "ReaBoot"}
        &#32;is a convenient all-in-one online installer for&#32;
        <span class="whitespace-nowrap">
            <ReaperRef/>,&#32;
            <ReapackRef/>&#32;and arbitrary&#32;
            <Help help="3rd-party add-ons for REAPER such as scripts, extensions and themes">
                packages
            </Help>
        </span>.
    </p>
}