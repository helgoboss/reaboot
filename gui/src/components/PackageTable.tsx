import {PackageUrl} from "../../src-tauri/bindings/PackageUrl.ts";
import {For} from "solid-js";

type Props = {
    packages: PackageUrl[],
}

export function PackageTable(props: Props) {
    return <table class="table table-xs table-pin-rows">
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
    </table>;
}
