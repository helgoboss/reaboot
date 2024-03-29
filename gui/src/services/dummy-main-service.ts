import {MainService} from "./main-service.ts";
import {ReabootEvent} from "../../../core/bindings/ReabootEvent.ts";
import {ReabootConfig} from "../../../core/bindings/ReabootConfig.ts";
import {Observable, Subject} from "rxjs";

export class DummyMainService implements MainService {
    private progressEventsSubject = new Subject<number>();

    private normalEventsSubject = new Subject<ReabootEvent>();

    configure(config: ReabootConfig) {
        this.normalEventsSubject.next({
            kind: "ConfigResolved",
            config: {
                reaper_resource_dir: config.custom_reaper_resource_dir ?? "main/resource/dir",
                portable: config.custom_reaper_resource_dir != null,
                reaper_target: "LinuxAarch64",
            }
        });
    }

    cancelInstallation() {
    }

    getNormalEvents(): Observable<ReabootEvent> {
        return this.normalEventsSubject;
    }

    getProgressEvents(): Observable<number> {
        return this.progressEventsSubject;
    }

    async startInstallation() {
        await simulateProgress(this.progressEventsSubject, 3000);
    }
}

function timeout(value: number) {
    return new Promise(resolve => setTimeout(resolve, value));
}

async function simulateProgress(subject: Subject<number>, millis: number) {
    for (var i = 0; i < millis; i += 30) {
        subject.next(i / millis);
        await timeout(1);
    }
}