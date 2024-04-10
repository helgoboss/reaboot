import {Recipe} from "../../core/bindings/Recipe";
import {PackageUrl} from "../../reapack/bindings/PackageUrl";
import {PackageVersionRef} from "../../reapack/bindings/PackageVersionRef";
import {PackagePath} from "../../reapack/bindings/PackagePath";
import {Convert, Recipe as JsonSchemaRecipe} from "./recipe-parser";

export async function tryExtractRecipe(text: string): Promise<Recipe | null> {
    return nullOnErrorAsync(() => extractRecipe(text));
}

export async function extractRecipe(text: string): Promise<Recipe> {
    const url = nullOnError(() => new URL(text));
    return url ? await getRecipeFromUrl(url) : parseRecipe(text);
}


async function getRecipeFromUrl(url: URL): Promise<Recipe> {
    const packageUrl = parsePackageUrl(url);
    return packageUrl ? buildRecipeFromPackageUrl(url, packageUrl) : await fetchRecipeFromUrl(url);
}

export function parseRawPackageUrl(raw: string): PackageUrl {
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

function buildRecipeFromPackageUrl(rawPackageUrl: URL, parsedPackageUrl: PackageUrl): Recipe {
    return {
        name: parsedPackageUrl.package_version_ref.package_path.package_name,
        required_packages: [
            rawPackageUrl.toString()
        ]
    };
}

async function fetchRecipeFromUrl(url: URL): Promise<Recipe> {
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
    return parseRecipe(text);
}

function parseRecipe(text: string): Recipe {
    const recipe = Convert.toRecipe(text);
    validatePackageUrls(recipe.required_packages);
    if (!recipe.features) {
        return recipe;
    }
    for (const feature of Object.values(recipe.features)) {
        validatePackageUrls(feature.packages);
    }
    return recipe;
}

function validatePackageUrls(packageUrls: string[] | null | undefined) {
    if (!packageUrls) {
        return;
    }
    for (const url of packageUrls) {
        parseRawPackageUrl(url);
    }
}

function nullOnError<R>(f: () => R): R | null {
    try {
        return f();
    } catch {
        return null;
    }
}

async function nullOnErrorAsync<R>(f: () => Promise<R>): Promise<R | null> {
    try {
        return await f();
    } catch {
        return null;
    }
}