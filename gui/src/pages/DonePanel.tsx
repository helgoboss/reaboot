import {MainButton} from "../components/MainButton.tsx";
import { ButtonList } from "../components/ButtonList.tsx";
import {SecondaryButton} from "../components/SecondaryButton.tsx";

export function DonePanel() {
    return (
        <>
            <ButtonList>
                <MainButton>
                    Close and launch REAPER
                </MainButton>
                <SecondaryButton>
                    Just close
                </SecondaryButton>
            </ButtonList>
        </>
    );
}