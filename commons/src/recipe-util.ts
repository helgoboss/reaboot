import {Recipe} from "../../core/bindings/Recipe";
import {PackageUrl} from "../../reapack/bindings/PackageUrl";
import {PackageVersionRef} from "../../reapack/bindings/PackageVersionRef";
import {PackagePath} from "../../reapack/bindings/PackagePath";

export async function tryExtractRecipe(text: string): Promise<Recipe | null> {
    const url = tryExtractUrl(text);
    return url ? await tryGetRecipeFromUrl(url) : tryParseRecipe(text);
}

async function tryGetRecipeFromUrl(url: URL): Promise<Recipe | null> {
    const packageUrl = tryParsePackageUrl(url);
    return packageUrl ? buildRecipeFromPackageUrl(url, packageUrl) : await tryFetchRecipeFromUrl(url);
}

function tryParsePackageUrl(url: URL): PackageUrl | null {
    const fragmentId = url.hash.substring(1);
    if (fragmentId.length === 0) {
        return null;
    }
    const packageVersionRef = tryParsePackageVersionRef(fragmentId);
    if (!packageVersionRef) {
        return null;
    }
    url.hash = "";
    return {
        repository_url: url.toString(),
        package_version_ref: packageVersionRef
    }
}

function tryParsePackageVersionRef(fragmentId: string): PackageVersionRef | null {
    const params = new URLSearchParams(fragmentId);
    const rawPackagePath = params.get("p");
    if (!rawPackagePath) {
        return null;
    }
    const packagePath = tryParsePackagePath(rawPackagePath);
    if (!packagePath) {
        return null;
    }
    const versionRef = params.get("v") ?? "latest";
    return {
        package_path: packagePath,
        version_ref: versionRef
    }
}

function tryParsePackagePath(text: string): PackagePath | null {
    const lastSlashIndex = text.lastIndexOf("/");
    if (lastSlashIndex === -1) {
        return null;
    }
    return {
        category: text.substring(0, lastSlashIndex),
        package_name: text.substring(lastSlashIndex + 1)
    }
}

function buildRecipeFromPackageUrl(rawPackageUrl: URL, parsedPackageUrl: PackageUrl): Recipe {
    return {
        name: parsedPackageUrl.package_version_ref.package_path.package_name,
        package_urls: [
            rawPackageUrl.toString()
        ]
    };
}

async function tryFetchRecipeFromUrl(url: URL): Promise<Recipe | null> {
    // Send request
    const response = await fetch(url);
    if (!response.ok) {
        return null;
    }
    // Don't continue if response contains too much data
    const maxContentLength = 100 * 1024;
    const contentLength = response.headers.get("Content-Length");
    if (contentLength === null || parseInt(contentLength) > maxContentLength) {
        return null;
    }
    // Try to read response as text
    const text = await response.text().catch(() => null);
    if (!text) {
        return null;
    }
    // Parse response text as recipe
    return tryParseRecipe(text);
}

function tryParseRecipe(text: string): Recipe | null {
    try {
        return JSON.parse(text);
    } catch {
        return null;
    }
}

function tryExtractUrl(text: string): URL | null {
    const lines = text.split("\n");
    if (lines.length !== 1) {
        return null;
    }
    return tryParseUrl(lines[0]);
}

function tryParseUrl(text: string): URL | null {
    try {
        return new URL(text);
    } catch {
        return null;
    }
}