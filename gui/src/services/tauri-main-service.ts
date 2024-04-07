import {MainService} from "./main-service.ts";
import {Observable} from "rxjs";
import {invoke} from "@tauri-apps/api/tauri";
import {listen} from "@tauri-apps/api/event";
import {debug} from "tauri-plugin-log-api";
import {InstallerConfig} from "../../../core/bindings/InstallerConfig.ts";
import {ReabootEvent} from "../../src-tauri/bindings/ReabootEvent.ts";
import {ReabootCommand} from "../../src-tauri/bindings/ReabootCommand.ts";

export class TauriMainService implements MainService {
    private normalEvents: Observable<ReabootEvent> = new Observable((subscriber) => {
        debug("Start listening to reaboot_event...");
        listen("reaboot_event", event => {
            const payload = event.payload as ReabootEvent;
            debug(`Received ReaBoot event: ${payload.kind}`);
            subscriber.next(payload);
        });
    });


    private progressEvents: Observable<number> = new Observable((subscriber) => {
        listen("reaboot_progress", event => {
            const payload = event.payload as number;
            subscriber.next(payload);
        });
    });

    getNormalEvents(): Observable<ReabootEvent> {
        return this.normalEvents;
    }

    getProgressEvents(): Observable<number> {
        return this.progressEvents;
    }

    async getReaperEula() {
        const eula = await invoke('get_reaper_eula', {});
        return eula as string;
    }

    async configure(config: InstallerConfig) {
        await this.invokeCommand({
            kind: "ConfigureInstallation",
            config
        });
    }

    async startInstallation() {
        await this.invokeCommand({
            kind: "StartInstallation",
        });
    }

    async startReaper() {
        await this.invokeCommand({
            kind: "StartReaper",
        });
    }

    async startReaperInstaller(path: string) {
        await this.invokeCommand({
            kind: "StartReaperInstaller",
            path
        });
    }

    private async invokeCommand(command: ReabootCommand) {
        await invoke('reaboot_command', {
            command
        });
    }
}
