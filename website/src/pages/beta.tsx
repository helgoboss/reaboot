import {NormalPage} from "../components/normal-page";

export default function Beta() {
    return (
        <NormalPage>
            <div class="prose">
                <h1 class="text-center">ReaBoot is currently being beta tested</h1>
                <p class="font-bold">That means it's being tested by a wider audience and you should use it with
                    care.</p>
                <h2>OS support</h2>
                <p>Here's an overview on what's currently supposed to work on which operating system and what not.</p>
                <table class="table table-zebra table-xs">
                    <thead>
                    <tr>
                        <th></th>
                        <th colspan="2">GUI</th>
                        <th colspan="2">CLI</th>
                        <th colspan="2">REAPER installation</th>
                    </tr>
                    <tr>
                        <th>OS/arch</th>
                        <th>Download</th>
                        <th>Execution</th>
                        <th>Download</th>
                        <th>Execution</th>
                        <th>Main</th>
                        <th>Portable</th>
                    </tr>
                    </thead>
                    <tbody>
                    <tr>
                        <td>macOS 10.15+ ARM 64-bit</td>
                        <td class="bg-success">Available</td>
                        <td class="bg-success">Optimal</td>
                        <td class="bg-error">Not published yet</td>
                        <td class="bg-success">Optimal</td>
                        <td class="bg-success">Guided</td>
                        <td class="bg-success">Fully automated</td>
                    </tr>
                    <tr>
                        <td>macOS 10.15+ x86 64-bit</td>
                        <td class="bg-success">Available</td>
                        <td class="bg-warning">Untested</td>
                        <td class="bg-error">Not published yet</td>
                        <td class="bg-warning">Untested</td>
                        <td class="bg-warning">Untested (should be guided)</td>
                        <td class="bg-warning">Untested (should be fully automated)</td>
                    </tr>
                    <tr>
                        <td>macOS 10.13-10.14 x86 64-bit</td>
                        <td class="bg-error">Not available (anyone need it?)</td>
                        <td>-</td>
                        <td class="bg-error">Not published yet</td>
                        <td class="bg-warning">Untested</td>
                        <td class="bg-warning">Untested (should not work)</td>
                        <td class="bg-success">Untested (should not work)</td>
                    </tr>
                    <tr>
                        <td>macOS * 32-bit</td>
                        <td class="text-neutral-400">Not planned</td>
                        <td>-</td>
                        <td class="text-neutral-400">Not planned</td>
                        <td>-</td>
                        <td>-</td>
                        <td>-</td>
                    </tr>
                    <tr>
                        <td>Windows 10+ 64-bit</td>
                        <td class="bg-warning">
                            Not ideal (not signed, sometimes false positive virus scan, NSIS/MSI)
                        </td>
                        <td class="bg-success">Optimal</td>
                        <td class="bg-error">Not published yet</td>
                        <td class="bg-success">Optimal</td>
                        <td class="bg-success">Fully automated</td>
                        <td class="bg-success">Fully automated</td>
                    </tr>
                    <tr>
                        <td>Windows 7 64-bit</td>
                        <td class="bg-warning">Not ideal (NSIS)</td>
                        <td class="bg-warning">Untested</td>
                        <td class="bg-error">Not published yet</td>
                        <td class="bg-warning">Untested</td>
                        <td class="bg-warning">Untested (should be fully automated)</td>
                        <td class="bg-warning">Untested (should be fully automated)</td>
                    </tr>
                    <tr>
                        <td>Windows * 32-bit</td>
                        <td class="text-neutral-400">Not planned</td>
                        <td>-</td>
                        <td class="text-neutral-400">Not planned</td>
                        <td>-</td>
                        <td>-</td>
                        <td>-</td>
                    </tr>
                    <tr>
                        <td>Linux x86 64-bit</td>
                        <td class="bg-warning">Not ideal (deb)</td>
                        <td class="bg-success">Optimal</td>
                        <td class="bg-error">Not published yet</td>
                        <td class="bg-success">Optimal</td>
                        <td class="bg-error">Not working (misleading info at the end of the installation)</td>
                        <td class="bg-success">Fully automated</td>
                    </tr>
                    <tr>
                        <td>Linux ARM 64-bit</td>
                        <td class="bg-error">Not available (anyone need it?)</td>
                        <td>-</td>
                        <td class="bg-error">Not available (anyone need it?)</td>
                        <td>-</td>
                        <td>-</td>
                        <td>-</td>
                        <td>-</td>
                    </tr>
                    <tr>
                        <td>Linux * 32-bit</td>
                        <td class="text-neutral-400">Not planned</td>
                        <td>-</td>
                        <td class="bg-error">Not available (anyone need it?)</td>
                        <td>-</td>
                        <td>-</td>
                        <td>-</td>
                        <td>-</td>
                    </tr>
                    </tbody>
                </table>
            </div>
        </NormalPage>
    );
}
