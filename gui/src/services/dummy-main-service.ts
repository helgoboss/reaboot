import {MainService} from "./main-service.ts";
import {Observable, Subject} from "rxjs";
import {InstallerConfig} from "../../../core/bindings/InstallerConfig.ts";
import {ReabootEvent} from "../../src-tauri/bindings/ReabootEvent.ts";

export class DummyMainService implements MainService {
    private progressEventsSubject = new Subject<number>();

    private normalEventsSubject = new Subject<ReabootEvent>();

    configure(config: InstallerConfig) {
        this.normalEventsSubject.next({
            kind: "ConfigResolved",
            config: {
                reaper_resource_dir: config.custom_reaper_resource_dir ?? "main/resource/dir",
                portable: config.custom_reaper_resource_dir != null,
                concurrent_downloads: config.concurrent_downloads ?? 5,
                dry_run: config.dry_run,
                keep_temp_dir: config.keep_temp_dir,
                num_download_retries: config.num_download_retries ?? 3,
                package_urls: config.package_urls,
                reaper_resource_dir_exists: true,
                reaper_version: config.reaper_version ?? "latest",
                skip_failed_packages: config.skip_failed_packages,
                temp_parent_dir: config.temp_parent_dir ?? "/tmp",
                platform: "LinuxAarch64",
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