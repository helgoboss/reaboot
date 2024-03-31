import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {Page} from "../components/Page.tsx";

export function PickPackagesPage() {
    return (
        <Page>
            <p class="text-center">
                ReaBoot can install packages for you. It will do so in a way that is compatible
                with ReaPack. Packages that are already installed will be replaced with the version that
                you provide here.
            </p>
            <div class="grow flex flex-col py-4">
                <textarea class="grow textarea textarea-primary"
                          placeholder="Paste package URLs here, one per line!">
                </textarea>
            </div>
            <p class="text-center">
                You can add more packages later at any time
                by using ReaPack in REAPER (Extensions → ReaPack → Browse packages) or by starting
                ReaBoot again.
            </p>
            <ButtonRow>
                <NavButton>Continue</NavButton>
            </ButtonRow>
        </Page>
    );
}

