import {Observable} from "rxjs";
import {InstallationStatusEvent} from "../../src-lib/bindings/InstallationStatusEvent.ts";

export type MainService = {
    // Returns the resource directory of the main REAPER installation, if one has been found.
    getMainReaperResourceDir: () => Promise<string | undefined>,
    // Returns installation events.
    getInstallationEvents: () => GetInstallationEventsReply,
    // Starts installation.
    startInstallation: (request: InstallationRequest) => void
}

export type InstallationRequest = {

}

export type GetInstallationEventsReply = {
    statusEvents: Observable<InstallationStatusEvent>,
    statusProgress: Observable<number>,
}