import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {Page} from "../components/Page.tsx";
import {mainStore} from "../globals.ts";
import {configureInstallation} from "../epics/install.ts";

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
                          placeholder="Paste package URLs here, one per line!"
                          onInput={(e) => processText(e.target.value)}>
                    {mainStore.state.packageUrlsExpression}
                </textarea>
            </div>
            <p class="text-center">
                Valid: {mainStore.state.packageUrls.length} / Invalid: {mainStore.state.invalidPackageUrls.length}
            </p>
            <p class="text-center">
                You can add more packages later at any time
                by starting ReaBoot again, or by using ReaPack in REAPER
                (Extensions&nbsp;→&nbsp;ReaPack&nbsp;→&nbsp;Browse packages).
            </p>
            <ButtonRow>
                <NavButton onClick={() => {
                    configureInstallation({});
                    return mainStore.currentPageId = "install";
                }}>
                    Continue
                </NavButton>
            </ButtonRow>
        </Page>
    );
}

function processText(text: string) {
    const lines = text.split("\n");
    const invalidUrls: string[] = [];
    const validUrls = lines
        .map(line => line.trim())
        .filter(line => {
            if (line.length === 0) {
                return false;
            }
            const isValid = isValidPackageUrl(line);
            if (!isValid) {
                invalidUrls.push(line);
            }
            return isValid;
        });
    // TODO set all at once (improve setter)
    mainStore.packageUrlsExpression = text;
    mainStore.invalidPackageUrls = invalidUrls;
    mainStore.packageUrls = validUrls;
}

function isValidPackageUrl(text: string): boolean {
    try {
        new URL(text);
        return true;
    } catch (e) {
        return false;
    }
}