// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Feature } from "./Feature";

export type Recipe = { name: string, sub_title?: string | null, description?: string | null, website?: string | null, skip_additional_packages?: boolean | null, required_packages?: Array<string> | null, features?: { [key in string]?: Feature } | null, };
