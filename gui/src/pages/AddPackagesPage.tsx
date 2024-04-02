import {ButtonRow} from "../components/ButtonRow.tsx";
import {NavButton} from "../components/NavButton.tsx";
import {Page} from "../components/Page.tsx";
import {mainStore} from "../globals.ts";
import {configureInstaller} from "../epics/install.ts";
import {WaitingForDataPage} from "./WaitingForDataPage.tsx";
import {PackageTable} from "../components/PackageTable.tsx";
import {clipboard} from "@tauri-apps/api";
import {FaSolidCirclePlus} from "solid-icons/fa";
import {showError} from "../epics/common.ts";

export function AddPackagesPage() {
    const resolvedConfig = mainStore.state.resolvedConfig;
    if (!resolvedConfig) {
        return <WaitingForDataPage/>;
    }
    return (
        <Page>
            <p class="text-center font-bold">
                Optional: Add more packages
            </p>
            <ButtonRow>
                <button class="btn" onClick={() => addPackageUrlsFromClipboard()}>
                    <FaSolidCirclePlus size={14}/> Add packages from clipboard
                </button>
            </ButtonRow>
            <div class="grow card bg-base-200 my-2 min-h-0">
                <div class="card-body min-h-0">
                    <div class="overflow-y-auto">
                        <PackageTable packages={resolvedConfig.package_urls}/>
                    </div>
                </div>
            </div>
            <p class="text-center">
                Packages which are already installed will be
                replaced with the version that
                you provide here.
                You can add more packages later at any time
                by starting ReaBoot again, or by using ReaPack in REAPER
                (Extensions&nbsp;→&nbsp;ReaPack&nbsp;→&nbsp;Browse packages).
            </p>
            <ButtonRow>
                <NavButton onClick={() => mainStore.setCurrentPageId("install")}>
                    {resolvedConfig.package_urls.length == 0 ? "Skip" : "Continue"}
                </NavButton>
            </ButtonRow>
        </Page>
    );
}

async function addPackageUrlsFromClipboard() {
    const text = await clipboard.readText();
    if (text == null) {
        showError("Clipboard is empty");
        return;
    }
    await addPackageUrls(text);
}


async function addPackageUrls(text: string) {
    // Parse lines
    const newPackageUrls = text.split("\n").map(line => line.trim());
    // Merge with existing package URLs and deduplicate
    const allPackages = new Set([...mainStore.state.installerConfig.package_urls, ...newPackageUrls]);
    // Configure
    await configureInstaller({packageUrls: [...allPackages]});
}
