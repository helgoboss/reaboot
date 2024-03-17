import MainButton from "./MainButton.tsx";
import { ButtonList } from "./ButtonList.tsx";
import SecondaryButton from "./SecondaryButton.tsx";

function DonePanel() {
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

export default DonePanel;