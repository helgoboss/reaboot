/* @refresh reload */
import {render} from "solid-js/web";

import "./styles.css";
import {App} from "./App";

import * as os from "@tauri-apps/plugin-os";
import * as process from "@tauri-apps/plugin-process";
import * as dialog from '@tauri-apps/plugin-dialog';

// TODO-high CONTINUE Activate as soon as we are sure on which OS it runs nicely and which not
async function verifyOs() {
    const minMacosVersion = "11";
    const osType = os.type();
    const osVersion = os.version();
    if (osType === "macos" && osVersion < minMacosVersion) {
        await dialog.message(
            `Sorry, you can't run the installer on this machine. You have macOS ${osVersion} but the installer needs at least macOS ${minMacosVersion}.
            
No worries. You can always use ReaPack to install the desired REAPER packages! Just head back to the installation website and choose "Via ReaPack".`,
            {
                title: "Incompatible macOS version",
                kind: "error",
                okLabel: "Exit",
            });
        await process.exit(1);
    }
}

// verifyOs();

render(() => <App/>, document.getElementById("root") as HTMLElement);
