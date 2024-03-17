import {mainStore} from "../services/globals.ts";
import {MainButton} from "../components/MainButton.tsx";

export function WelcomePanel() {
    return (
        <>
            <MainButton onClick={() => mainStore.openPage("pick-reaper")}>
                Let's go!
            </MainButton>
        </>
    );
}