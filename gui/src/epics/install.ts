import {mainService, mainStore} from "../globals.ts";
import {showError} from "./common.tsx";

type PatchConfigurationArgs = {
    custom_reaper_resource_dir?: string | null,
    packageUrls?: string[]
};

export async function install() {
    // At first, reset outcome of potential previous installation
    mainStore.setInstallationReportHtml(undefined);
    // Then start installation
    await mainService.startInstallation();
}

export async function configureInstaller(args: PatchConfigurationArgs) {
    const oldConfig = mainStore.state.installerConfig;
    const newConfig = {
        ...oldConfig,
        custom_reaper_resource_dir: args.custom_reaper_resource_dir === undefined ? oldConfig.custom_reaper_resource_dir : (args.custom_reaper_resource_dir ?? undefined),
        package_urls: args.packageUrls ?? oldConfig.package_urls,
    };
    try {
        await mainService.configure(newConfig);
        // Only write config to store if configuration was successful
        mainStore.setInstallerConfig(newConfig);
    } catch (e) {
        showError(e);
    }
}