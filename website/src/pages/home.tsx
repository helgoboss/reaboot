import {Welcome} from "../components/welcome";
import {NormalPage} from "../components/normal-page";

export default function Home() {
    return (
        <NormalPage>
            <Welcome poweredBy={false} examples={true}/>
        </NormalPage>
    );
}
