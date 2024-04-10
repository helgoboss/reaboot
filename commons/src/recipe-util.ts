import {Recipe} from "../../core/bindings/Recipe";
import {PackageUrl} from "../../reapack/bindings/PackageUrl";
import {PackageVersionRef} from "../../reapack/bindings/PackageVersionRef";
import {PackagePath} from "../../reapack/bindings/PackagePath";
import {Convert, Recipe as JsonSchemaRecipe} from "./recipe-parser";
import {Feature} from "../../core/bindings/Feature";


export type ParsedRecipe = {
    raw: Recipe,
    requiredPackages: PackageUrl[],
    features: Record<string, ParsedFeature>,
}

export type ParsedFeature = {
    raw: Feature,
    packages: PackageUrl[],
}

export async function tryExtractRecipe(text: string): Promise<ParsedRecipe | null> {
    return getOrNullAsync(() => extractRecipe(text));
}

export async function extractRecipe(text: string): Promise<ParsedRecipe> {
    const url = getOrNull(() => new URL(text));
    return url ? await getRecipeFromUrl(url) : parseRecipeFromRawString(text);
}

async function getRecipeFromUrl(url: URL): Promise<ParsedRecipe> {
    const packageUrl = getOrNull(() => parsePackageUrl(url));
    return packageUrl ? buildRecipeFromPackageUrl(url, packageUrl) : await fetchRecipeFromUrl(url);
}

export function parsePackageUrlFromRawString(raw: string): PackageUrl {
    try {
        const url = new URL(raw);
        return parsePackageUrlInternal(url);
    } catch (cause) {
        throw new Error(`"${raw}" is not a valid package URL:\n${cause}`);
    }
}

export function parsePackageUrl(url: URL): PackageUrl {
    try {
        return parsePackageUrlInternal(url);
    } catch (cause) {
        throw new Error(`"${url}" is not a valid package URL:\n${cause}`);
    }
}

function parsePackageUrlInternal(url: URL): PackageUrl {
    const fragmentId = url.hash.substring(1);
    if (fragmentId.length === 0) {
        throw new Error("Fragment identifier missing");
    }
    const packageVersionRef = parsePackageVersionRef(fragmentId);
    const repositoryUrl = new URL(url);
    repositoryUrl.hash = "";
    return {
        repository_url: repositoryUrl.toString(),
        package_version_ref: packageVersionRef
    }
}

function parsePackageVersionRef(fragmentId: string): PackageVersionRef {
    const params = new URLSearchParams(fragmentId);
    const rawPackagePath = params.get("p");
    if (!rawPackagePath) {
        throw new Error("Package path missing");
    }
    const packagePath = parsePackagePath(rawPackagePath);
    const versionRef = params.get("v") ?? "latest";
    return {
        package_path: packagePath,
        version_ref: versionRef
    }
}

function parsePackagePath(text: string): PackagePath {
    const lastSlashIndex = text.lastIndexOf("/");
    if (lastSlashIndex === -1) {
        throw new Error("Package category missing");
    }
    return {
        category: text.substring(0, lastSlashIndex),
        package_name: text.substring(lastSlashIndex + 1)
    }
}

function buildRecipeFromPackageUrl(rawUrl: URL, parsedPackageUrl: PackageUrl): ParsedRecipe {
    const recipe: Recipe = {
        name: parsedPackageUrl.package_version_ref.package_path.package_name,
        required_packages: [rawUrl.toString()]
    };
    return parseRecipe(recipe);
}

async function fetchRecipeFromUrl(url: URL): Promise<ParsedRecipe> {
    // Send request
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error("Non-successful response status code from recipe URL");
    }
    // Don't continue if response contains too much data
    const maxContentLength = 100 * 1024;
    const contentLength = response.headers.get("Content-Length");
    if (contentLength === null || parseInt(contentLength) > maxContentLength) {
        throw new Error("Recipe URL response too big");
    }
    // Try to read response as text
    const text = await response.text().catch(() => null);
    if (!text) {
        throw new Error("Recipe URL doesn't return text");
    }
    // Parse response text as recipe
    return parseRecipeFromRawString(text);
}

function parseRecipeFromRawString(text: string): ParsedRecipe {
    const recipe = Convert.toRecipe(text);
    return parseRecipe(recipe);
}

function parseRecipe(recipe: Recipe): ParsedRecipe {
    const rawFeatures = getOrEmptyRecord(recipe.features);
    const parsedFeatures: Record<string, ParsedFeature> = {};
    for (const featureId in rawFeatures) {
        const rawFeature = rawFeatures[featureId];
        parsedFeatures[featureId] = parseFeature(rawFeature);
    }

    return {
        raw: recipe,
        requiredPackages: getOrEmptyArray(recipe.required_packages).map(parsePackageUrlFromRawString),
        features: parsedFeatures,
    };
}

function parseFeature(feature: Feature): ParsedFeature {
    return {
        raw: feature,
        packages: getOrEmptyArray(feature.packages).map(parsePackageUrlFromRawString),
    };
}

function getOrNull<R>(f: () => R): R | null {
    try {
        return f();
    } catch {
        return null;
    }
}

async function getOrNullAsync<R>(f: () => Promise<R>): Promise<R | null> {
    try {
        return await f();
    } catch {
        return null;
    }
}

function getOrEmptyArray<T>(items: T[] | null | undefined): T[] {
    return items ?? [];
}

function getOrEmptyRecord<T>(items: Record<string, T> | null | undefined): Record<string, T> {
    return items ?? {};
}

function parsePackageUrls(packageUrls: string[]): PackageUrl[] {
    return packageUrls.map(raw => parsePackageUrlFromRawString(raw));
}