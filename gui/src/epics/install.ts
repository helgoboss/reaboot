import {mainService, mainStore} from "../globals.ts";

export function configureInstallation(args: { portable?: boolean }) {
    const portable = args.portable === undefined ? (mainStore.state.resolvedConfig?.portable ?? false) : args.portable;
    mainService.configure({
        custom_reaper_resource_dir: portable ? mainStore.state.portableReaperDir : undefined,
        package_urls: mainStore.state.packageUrls,
        keep_temp_dir: false,
        dry_run: false,
        skip_failed_packages: false,
    });
}