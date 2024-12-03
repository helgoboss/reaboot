import {MainService} from "./main-service.ts";
import {Observable, Subject} from "rxjs";
import {InstallerConfig} from "../../../core/bindings/InstallerConfig.ts";
import {ReabootEvent} from "../../src-tauri/bindings/ReabootEvent.ts";

export class DummyMainService implements MainService {
    private progressEventsSubject = new Subject<number>();

    private normalEventsSubject = new Subject<ReabootEvent>();

    async configure(config: InstallerConfig) {
        this.normalEventsSubject.next({
            kind: "ConfigResolved",
            config: {
                reaper_resource_dir: config.custom_reaper_resource_dir ?? "main/resource/dir",
                portable: config.custom_reaper_resource_dir != null,
                concurrent_downloads: config.concurrent_downloads ?? 5,
                dry_run: config.dry_run,
                keep_temp_dir: config.keep_temp_dir,
                num_download_retries: config.num_download_retries ?? 3,
                package_urls: [],
                reaper_exe_exists: true,
                reaper_exe: "bla",
                reaper_ini_exists: true,
                update_reaper: false,
                reaper_version: config.reaper_version ?? "latest",
                skip_failed_packages: config.skip_failed_packages,
                temp_parent_dir: config.temp_parent_dir ?? "/tmp",
                platform: "linux-aarch64",
                recipe: {
                    name: "ReaLearn",
                    sub_title: "Helgoboss Projects",
                    required_packages: [],
                    website: "https://www.helgoboss.org/projects/realearn/",
                },
                install_reapack: true,
            }
        });
    }

    getNormalEvents(): Observable<ReabootEvent> {
        return this.normalEventsSubject;
    }

    getProgressEvents(): Observable<number> {
        return this.progressEventsSubject;
    }

    async getReaperEula() {
        return "";
    }

    async startInstallation() {
        await simulateProgress(this.progressEventsSubject, 3000);
    }

    async startReaper() {
    }

    async startReaperInstaller(_path: string) {
    }

    async confirm(_answer: boolean) {
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