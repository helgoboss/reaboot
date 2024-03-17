import {GetInstallationEventsReply, InstallationRequest, MainService} from "./main-service.ts";
import {Observable} from "rxjs";
import {InstallationStatusEvent} from "../../src-lib/bindings/InstallationStatusEvent.ts";
import {invoke} from "@tauri-apps/api/tauri";
import {WorkerCommand} from "../../src-lib/bindings/WorkerCommand.ts";
import {listen} from "@tauri-apps/api/event";

export class TauriMainService implements MainService {
    statusEvents: Observable<InstallationStatusEvent> = new Observable((subscriber) => {
        listen("installation-status", event => {
            subscriber.next(event.payload as InstallationStatusEvent);
        });
    });


    statusProgressEvents: Observable<number> = new Observable((subscriber) => {
        listen("installation-status-progress", event => {
            subscriber.next(event.payload as number);
        });
    });

    async getMainReaperResourceDir() {
        return Math.random() < 0.5 ? undefined : "/bla/foo";
    }

    getInstallationEvents(): GetInstallationEventsReply {
        return {
            statusEvents: this.statusEvents,
            statusProgress: this.statusProgressEvents,
        }
    }

    async startInstallation(_: InstallationRequest) {
        const command: WorkerCommand = {
            kind: "Install"
        };
        await invoke('work', {
            command
        });
    }
}
