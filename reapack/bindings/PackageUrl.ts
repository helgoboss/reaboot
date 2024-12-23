// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { PackageVersionRef } from "./PackageVersionRef";

/**
 * The Package URL is a URL that points to a specific repository index and uniquely identifies a
 * specific version of a package within that index.
 *
 * # Examples
 *
 * - `https://github.com/ReaTeam/ReaScripts/raw/master/index.xml#p=Various/rodilab_Color%20palette.lua&v=1.2.3-pre`
 * - `https://github.com/helgoboss/reaper-packages/raw/master/index.xml#p=Extensions/Helgobox-x64`
 *
 * # Structure
 *
 * The URL follows this schema:
 *
 * - **PACKAGE_URL =** `{REPOSITORY_INDEX_URL}#{PACKAGE_VERSION_REF}`
 * - **PACKAGE_VERSION_REF =** `p={PACKAGE_PATH}&v={VERSION_REF}`
 * - **PACKAGE_PATH =** `{CATEGORY}/{PACKAGE_NAME}`
 * - **VERSION_REF =** `{VERSION_NAME}` or `latest` or `latest-pre`
 */
export type PackageUrl = { 
/**
 * Repository index URL.
 *
 * - Must not contain any `#` character.
 */
repository_url: string, 
/**
 * Package version reference.
 *
 * - This part is technically the [fragment ID](https://www.w3.org/Addressing/URL/4_2_Fragments.html)
 *   of the URL.
 * - See [`PackageVersionRef`]
 */
package_version_ref: PackageVersionRef, };
