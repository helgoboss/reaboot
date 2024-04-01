import {mainService, mainStore} from "../globals.ts";

export function configureInstallation(args: { portable: boolean }) {
    mainService.configure({
        custom_reaper_resource_dir: args.portable ? mainStore.state.portableReaperDir : undefined,
        package_urls: mainStore.state.packageUrls,
        keep_temp_dir: false,
        dry_run: false,
        skip_failed_packages: true,
    });
}