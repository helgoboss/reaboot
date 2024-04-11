import {mainService, mainStore} from "../globals.ts";
import {showError} from "./common.tsx";
import {Recipe} from "../../../core/bindings/Recipe.ts";

type PatchConfigurationArgs = {
    recipe?: Recipe | null,
    customReaperResourceDir?: string | null,
    packageUrls?: string[],
    selectedFeatures?: string[],
    updateReaper?: boolean,
};

export async function install() {
    // At first, reset outcome of potential previous installation
    mainStore.setInstallationReport(undefined, false);
    // Then start installation
    await mainService.startInstallation();
}

export async function configureInstaller(args: PatchConfigurationArgs) {
    const oldConfig = mainStore.state.installerConfig;
    const newConfig = {
        ...oldConfig,
        recipe: args.recipe === undefined ? oldConfig.recipe : (args.recipe ?? undefined),
        custom_reaper_resource_dir: args.customReaperResourceDir === undefined ? oldConfig.custom_reaper_resource_dir : (args.customReaperResourceDir ?? undefined),
        package_urls: args.packageUrls ?? oldConfig.package_urls,
        selected_features: args.selectedFeatures ?? oldConfig.selected_features,
        update_reaper: args.updateReaper ?? oldConfig.update_reaper,
    };
    try {
        await mainService.configure(newConfig);
        // Only write config to store if configuration was successful
        mainStore.setInstallerConfig(newConfig);
    } catch (e) {
        showError(e);
    }
}