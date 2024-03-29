import {InstallationStage} from "../../../core/bindings/InstallationStage.ts";

type Props = {
    stage: InstallationStage,
}

export function InstallationStatusPanel(props: Props) {
    return <div>
        <div>{props.stage.kind}</div>
        {build(props.stage)}
    </div>;
}

function build(status: InstallationStage) {
    switch (status.kind) {
        case "DownloadingRepositoryIndexes":
        case "DownloadingPackageFiles":
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
