import MainButton from "./MainButton";
import MainStore from "../store/main-store.ts";

function WelcomePanel() {
    return (
        <>
            <MainButton onClick={() => MainStore.openPage("pick-reaper")}>
                Let's go!
            </MainButton>
        </>
    );
}

export default WelcomePanel;