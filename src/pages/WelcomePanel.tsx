import {mainStore} from "../globals.ts";
import {MainButton} from "../components/MainButton.tsx";

export function WelcomePanel() {
    return (
        <>
            <MainButton onClick={() => mainStore.currentPageId = "pick-reaper"}>
                Let's go!
            </MainButton>
        </>
    );
}