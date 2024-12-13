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
                            <th rowspan="2">OS/arch</th>
                            <th rowspan="2">GUI installer</th>
                            <th colspan="2">REAPER installation</th>
                        </tr>
                        <tr>
                            <th>Main</th>
                            <th>Portable</th>
                        </tr>
                        </thead>
                        <tbody>
                        <tr>
                            <td>Windows 10+ (64-bit)</td>
                            <td class="h-success">Optimal</td>
                            <td class="h-success">Fully automated</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>Windows 7 and 8 (64-bit)</td>
                            <td class="h-success" title="Requires WebView2 installation">Acceptable</td>
                            <td class="h-success">Fully automated</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>Windows * (32-bit)</td>
                            <td class="text-neutral-400">Not planned</td>
                            <td>-</td>
                            <td>-</td>
                        </tr>
                        <tr>
                            <td>macOS 11+ (x86 64-bit and ARM 64-bit)</td>
                            <td class="h-success">Optimal</td>
                            <td class="h-success">Guided</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>macOS 10.14 and 10.15 (x86 64-bit)</td>
                            <td class="h-success" title="Doesn't look too good">Acceptable</td>
                            <td class="h-success">Guided</td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>macOS 10.13 or earlier (x86 64-bit)</td>
                            <td class="h-error">Unsupported</td>
                            <td>-</td>
                            <td>-</td>
                        </tr>
                        <tr>
                            <td>macOS * (32-bit)</td>
                            <td class="text-neutral-400">Not planned</td>
                            <td>-</td>
                            <td>-</td>
                        </tr>
                        <tr>
                            <td>Linux x86 64-bit</td>
                            <td class="h-warning" title="Only in DEB format (not an easy one-click installer)">Not
                                ideal
                            </td>
                            <td class="h-error" title="Misleading info at the end of the installation">Not working
                            </td>
                            <td class="h-success">Fully automated</td>
                        </tr>
                        <tr>
                            <td>Linux ARM 64-bit</td>
                            <td class="h-error">Not available yet</td>
                            <td>-</td>
                            <td>-</td>
                        </tr>
                        <tr>
                            <td>Linux * 32-bit</td>
                            <td class="text-neutral-400">Not planned</td>
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
