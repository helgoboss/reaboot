import {WelcomePanel} from "./pages/WelcomePanel.tsx";
import {Page} from "./components/Page.tsx";
import {PageDescriptor} from "./model/page.ts";
import {PickReaperPanel} from "./pages/PickReaperPanel.tsx";
import { InstallPanel } from "./pages/InstallPanel.tsx";
import {DonePanel} from "./pages/DonePanel.tsx";
import {Stepper} from "./components/Stepper.tsx";
import {mainStore} from "./services/globals.ts";

export function App() {
    const currentPageDescriptor = () => pages.find((p) => p.id == mainStore.state.currentPageId)!;
    return <div class="w-screen h-screen flex flex-col">
        <header class="flex-none bg-gray-100"><Stepper pages={pages} currentPageId={currentPageDescriptor().id}/>
        </header>
        <main class="flex-grow bg-white">
            <Page title={currentPageDescriptor().title} description={currentPageDescriptor().description}>
                {currentPageDescriptor().content({})}
            </Page>
        </main>
    </div>
}

const pages: PageDescriptor[] = [
    {
        id: "welcome",
        title: "Welcome!",
        description: "This installer provides an easy and clean way to set up REAPER, ReaPack and add-ons of your choice.",
        content: WelcomePanel,
    },
    {
        id: "pick-reaper",
        title: "Pick REAPER",
        description: "You can install REAPER from scratch or choose an existing installation.",
        content: PickReaperPanel,
    },
    {
        id: "pick-bundles",
        title: "Pick bundles",
        description: "ReaBoot allows you to add initial bundles. Bundles are simply collections of ReaPack packages. You can add packages at a later time, either here or within ReaPack itself.",
        content: PickReaperPanel,
    },
    {
        id: "install",
        title: "Install",
        description: "Now it's time to review your settings and start the installation.",
        content: InstallPanel,
    },
    {
        id: "done",
        title: "Done",
        description: "Congratulations! Installation is finished",
        content: DonePanel,
    },
];