/* @refresh reload */
import {render} from "solid-js/web";

import "./styles.css";
import {App} from "./App";

import * as os from "@tauri-apps/plugin-os";
import * as process from "@tauri-apps/plugin-process";
import * as dialog from '@tauri-apps/plugin-dialog';

async function verifyOs() {
    const osType = os.type();
    const osVersion = os.version();
    if (osType === "macos") {
        const minMacosVersion = "10.14";
        const minOptimalMacosVersion = "11";
        if (osVersion < minMacosVersion) {
            await dialog.message(
                `Sorry, you can't run the installer on this machine. You have macOS ${osVersion} but the installer needs at least macOS ${minMacosVersion}.
            
No worries. You can always use ReaPack to install the desired REAPER packages! Just head back to the installation website and choose "Via ReaPack".`,
                {
                    title: "Incompatible macOS version",
                    kind: "error",
                    okLabel: "Exit",
                },
            );
            await process.exit(1);
            return;
        }
        if (osVersion < minOptimalMacosVersion) {
            await dialog.message(
                `You are running macOS ${osVersion}, which is an older version. As a result, the installer's appearance may not be optimal on your system, but rest assured, it should work just fine.`,
                {
                    title: "Old macOS version",
                    kind: "warning",
                    okLabel: "Continue",
                },
            );
        }
    }
}

verifyOs();

render(() => <App/>, document.getElementById("root") as HTMLElement);
