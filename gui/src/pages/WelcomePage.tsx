import {mainStore} from "../globals.ts";

export function WelcomePage() {
    return (
        <div class="grow hero">
            <div class="hero-content text-center">
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">Welcome!</h1>
                    <p class="py-6">
                        ReaBoot is a convenient all-in-one installer for&#32;
                        <a href="https://reaper.fm/" target="_blank" class="tooltip tooltip-success underline"
                           data-tip="The DAW we all love">
                            REAPER
                        </a>
                        ,&#32;
                        <a href="https://reapack.com/" target="_blank" class="tooltip tooltip-success underline"
                           data-tip="The standard package manager for REAPER">
                            ReaPack
                        </a>
                        &#32;and any&#32;
                        <span class="tooltip underline"
                              data-tip="3rd-party add-ons for REAPER, e.g. scripts, extensions and themes.">
                            packages
                        </span>
                        &#32;of your choice.
                    </p>
                    <button class="btn btn-primary"
                            onClick={() => mainStore.currentPageId = "pick-reaper"}>
                        Let's go!
                    </button>
                </div>
            </div>
        </div>
    );
}