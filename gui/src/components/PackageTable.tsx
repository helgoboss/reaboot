import {PackageUrl} from "../../src-tauri/bindings/PackageUrl.ts";
import {For, Show} from "solid-js";

type Props = {
    packages: PackageUrl[],
}

export function PackageTable(props: Props) {
    return <div>
        <table class="table table-xs table-pin-rows">
            <thead>
            <tr>
                <th>Name</th>
                <th>Category</th>
                <th>Version</th>
            </tr>
            </thead>
            <tbody>
            <For each={props.packages}>
                {url =>
                    <tr class="border-none">
                        <td>
                            {url.package_version_ref.package_path.package_name}
                        </td>
                        <td>
                            {url.package_version_ref.package_path.category}
                        </td>
                        <td>
                            {url.package_version_ref.version_ref}
                        </td>
                    </tr>
                }
            </For>
            </tbody>
        </table>
        <Show when={props.packages.length === 0}>
            <div class="text-center text-neutral text-sm pt-1">
                No packages are planned for installation
            </div>
        </Show>
    </div>;
}
