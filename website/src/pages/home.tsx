import {Welcome} from "../components/welcome";
import {NormalPage} from "../components/normal-page";

export default function Home() {
    return (
        <NormalPage>
            <div class="flex flex-col gap-4 items-center lg:flex-row">
                <Welcome/>
                <div>
                    <h2 class="text-center text-2xl mb-4 lg:hidden">Introduction video</h2>
                    <iframe
                        class="w-full aspect-video"
                        width="560" height="315"
                        src="https://www.youtube-nocookie.com/embed/LFveUpUrHFA?si=04UBLIDqVSpfjMXD"
                        title="YouTube video player" frameborder="0"
                        allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                        referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>
                </div>
            </div>
        </NormalPage>
    );
}
