import {Page} from "../components/Page.tsx";
import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";

export function DonePanel() {
    return (
        <Page>
            <div class="grow flex flex-col py-4">
                <textarea class="grow textarea textarea-primary"
                          placeholder="REPORT TODO">
                </textarea>
            </div>
            <ButtonRow>
                <NavButton>Close</NavButton>
                <NavButton>Close & Launch REAPER</NavButton>
            </ButtonRow>
        </Page>
    );
}