import {InstallationStatus} from "../../src-lib/bindings/InstallationStatus.ts";

type Props = {
    status: InstallationStatus,
}

export function InstallationStatusPanel(props: Props) {
    return <div>
        <div>{props.status.kind}</div>
        {build(props.status)}
    </div>;
}

function build(status: InstallationStatus) {
    switch (status.kind) {
        case "DownloadingRepositories":
            const download = status.download;
            return <div>
                <div>Downloads so far: {download.success_count + download.error_count} / {download.total_count}</div>
                <div>Currently downloading: {download.in_progress_count}</div>
                <div>Errors: {download.error_count}</div>
            </div>;
        default:
            return <div></div>
    }
}
