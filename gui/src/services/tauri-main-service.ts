import {MainService} from "./main-service.ts";
import {Observable} from "rxjs";
import {invoke} from "@tauri-apps/api/tauri";
import {listen} from "@tauri-apps/api/event";
import {debug} from "tauri-plugin-log-api";
import {ReabootEvent} from "../../../core/bindings/ReabootEvent.ts";
import {ReabootCommand} from "../../../core/bindings/ReabootCommand.ts";
import {ReabootConfig} from "../../../core/bindings/ReabootConfig.ts";

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

    configure(config: ReabootConfig): void {
        this.sendCommand({
            kind: "Configure",
            config
        })
    }

    getNormalEvents(): Observable<ReabootEvent> {
        return this.normalEvents;
    }

    getProgressEvents(): Observable<number> {
        return this.progressEvents;
    }

    startInstallation(): void {
        this.sendCommand({
            kind: "StartInstallation",
        });
    }

    cancelInstallation(): void {
        this.sendCommand({
            kind: "CancelInstallation",
        });
    }

    private sendCommand(command: ReabootCommand) {
        invoke('reaboot_command', {
            command
        })
    }
}
