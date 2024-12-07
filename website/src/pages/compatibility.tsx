import {Page} from "../components/page";

export default function Compatibility() {
    return (
        <Page>
            <div class="h-responsive-prose">
                <h2>OS support</h2>
                <p>Here's an overview on what's currently supposed to work on which operating system and what not.</p>
                <div class="overflow-x-auto">
                    <table class="table table-zebra table-xs bg-neutral">
                        <thead>
                        <tr>
                            <th></th>
                            <th colspan="2" class="border-gray-600 border-x-[1px]">GUI</th>
                            {/*<th colspan="2" class="border-gray-600 border-x-[1px]">CLI</th>*/}
                            <th colspan="2">REAPER installation</th>
                        </tr>
                        <tr>
                            <th>OS/arch</th>
                            <th class="border-gray-600 border-l-[1px]">Download</th>
                            <th>Execution</th>
                            {/*<th class="border-gray-600 border-l-[1px]">Download</th>*/}
                            {/*<th>Execution</th>*/}
                            <th class="border-gray-600 border-l-[1px]">Main</th>
                            <th>Portable</th>
                        </tr>
                        </thead>
                        <tbody>
                        <tr>
                            <td>macOS 11+ (x86 64-bit and ARM 64-bit)</td>
                            <td class="h-success">Available</td>
                            <td class="h-success">Optimal</td>
                            {/*<td class="h-error">Not published yet</td>*/}
                            {/*<td class="h-success">Optimal</td>*/}
                            <td class="h-success">Guided</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>macOS 10.15 (x86 64-bit)</td>
                            <td class="h-success">Available</td>
                            <td class="h-success" title="Doesn't look too good">Acceptable</td>
                            {/*<td class="h-error">Not published yet</td>*/}
                            {/*<td class="h-warning">Untested</td>*/}
                            <td class="h-success">Guided</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>macOS 10.14 or earlier (x86 64-bit)</td>
                            <td class="h-success">Available</td>
                            <td class="h-error">Unsupported</td>
                            {/*<td class="h-error">Not published yet</td>*/}
                            {/*<td class="h-warning">Untested</td>*/}
                            <td class="h-warning" title="Should NOT work">Untested</td>
                            <td class="h-warning" title="Should NOT work">Untested</td>
                        </tr>
                        <tr>
                            <td>macOS * (32-bit)</td>
                            <td class="text-neutral-400">Not planned</td>
                            <td>-</td>
                            {/*<td class="text-neutral-400">Not planned</td>*/}
                            {/*<td>-</td>*/}
                            <td>-</td>
                            <td>-</td>
                        </tr>
                        <tr>
                            <td>Windows 10+ (64-bit)</td>
                            <td class="h-success">Available</td>
                            <td class="h-success">Optimal</td>
                            {/*<td class="h-error">Not published yet</td>*/}
                            {/*<td class="h-success">Optimal</td>*/}
                            <td class="h-success">Fully automated</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>Windows 7 and 8 (64-bit)</td>
                            <td class="h-warning" title="Requires WebView2 installation">Not ideal</td>
                            <td class="h-success">Optimal</td>
                            {/*<td class="h-error">Not published yet</td>*/}
                            {/*<td class="h-warning">Untested</td>*/}
                            <td class="h-success">Fully automated</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>Windows * (32-bit)</td>
                            <td class="text-neutral-400">Not planned</td>
                            <td>-</td>
                            {/*<td class="text-neutral-400">Not planned</td>*/}
                            {/*<td>-</td>*/}
                            <td>-</td>
                            <td>-</td>
                        </tr>
                        <tr>
                            <td>Linux x86 64-bit</td>
                            <td class="h-warning" title="Only in DEB format (not an easy one-click installer)">Not
                                ideal
                            </td>
                            <td class="h-success">Optimal</td>
                            {/*<td class="h-error">Not published yet</td>*/}
                            {/*<td class="h-success">Optimal</td>*/}
                            <td class="h-error" title="Misleading info at the end of the installation">Not working</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>Linux ARM 64-bit</td>
                            <td class="h-error">Not available yet</td>
                            <td>-</td>
                            {/*<td class="h-error">Not available (anyone need it?)</td>*/}
                            {/*<td>-</td>*/}
                            <td>-</td>
                            <td>-</td>
                        </tr>
                        <tr>
                            <td>Linux * 32-bit</td>
                            <td class="text-neutral-400">Not planned</td>
                            <td>-</td>
                            {/*<td class="h-error">Not available (anyone need it?)</td>*/}
                            {/*<td>-</td>*/}
                            <td>-</td>
                            <td>-</td>
                        </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </Page>
    );
}
