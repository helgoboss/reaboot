import {GetInstallationEventsReply, InstallationRequest, MainService} from "./main-service.ts";
import {BehaviorSubject} from "rxjs";
import {InstallationStatusEvent} from "../../src-lib/bindings/InstallationStatusEvent.ts";

export class DummyMainService implements MainService {
    statusProgressSubject = new BehaviorSubject<number>(0);

    statusSubject = new BehaviorSubject<InstallationStatusEvent>({
        kind: "Idle",
    });

    async getMainReaperResourceDir() {
        return Math.random() < 0.5 ? undefined : "/bla/foo";
    }

    getInstallationEvents(): GetInstallationEventsReply {
        return {
            statusEvents: this.statusSubject.asObservable(),
            statusProgress: this.statusProgressSubject.asObservable(),
        }
    }

    async startInstallation(_: InstallationRequest) {
        this.statusSubject.next({
            kind: "DownloadingReaper",
            file: {
                label: "REAPER v7.11 for macOS 10.15+",
                url: "https://www.reaper.fm/files/7.x/reaper711_universal.dmg",
            }
        });
        await simulateProgress(this.statusProgressSubject, 4000);
        this.statusSubject.next({
            kind: "DownloadingReaPack",
            file: {
                label: "ReaPack for macOS 10.15+",
                url: "https://www.reaper.fm/files/7.x/reaper711_universal.dmg",
            }
        });
        await simulateProgress(this.statusProgressSubject, 7000);
        this.statusSubject.next({
            kind: "DownloadingRepositoryIndex",
            file: {
                label: "Nice repo",
                url: "https://www.reaper.fm/files/7.x/reaper711_universal.dmg",
            }
        });
    }
}

function timeout(value: number) {
    return new Promise(resolve => setTimeout(resolve, value));
}

async function simulateProgress(subject: BehaviorSubject<number>, millis: number) {
    for (var i = 0; i < millis; i += 30) {
        subject.next(i / millis);
        await timeout(1);
    }
}