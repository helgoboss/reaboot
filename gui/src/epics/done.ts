import {mainService} from "../globals.ts";
import {exit} from '@tauri-apps/plugin-process';

export async function startReaperAndQuit() {
    await mainService.startReaper();
    await quitAfterMoment();
}

export async function startReaperInstaller(path: string) {
    await mainService.startReaperInstaller(path);
}

async function quitAfterMoment() {
    await new Promise(r => setTimeout(r, 1000));
    await exit(0);
}