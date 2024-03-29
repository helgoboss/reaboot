import {mainStore} from "../globals.ts";
import {MainButton} from "../components/MainButton.tsx";

export function WelcomePage() {
    return (
        <div class="grow hero">
            <div class="hero-content text-center">
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">Welcome!</h1>
                    <p class="py-6">
                        ReaBoot is an all-in-one installer for REAPER, ReaPack and the packages of your
                        choice.
                    </p>
                    <MainButton onClick={() => mainStore.currentPageId = "pick-reaper"}>
                        Let's go!
                    </MainButton>
                </div>
            </div>
        </div>
    );
}