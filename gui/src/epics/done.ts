import {mainService} from "../globals.ts";
import {exit} from '@tauri-apps/api/process';

export async function startReaperAndQuit() {
    await mainService.startReaper();
    await new Promise(r => setTimeout(r, 1000));
    await exit(0);
}