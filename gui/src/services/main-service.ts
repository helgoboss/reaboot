import {Observable} from "rxjs";
import {InstallerConfig} from "../../../core/bindings/InstallerConfig.ts";
import {ReabootEvent} from "../../src-tauri/bindings/ReabootEvent.ts";

// ReaBoot main service.
export type MainService = {
    // Returns a stream of normal ReaBoot events (sent occasionally).
    //
    // Events are our only mechanism to acquire data from the backend. This saves us from
    // introducing additional getter functions and from having to remember calling them.
    // As long as we listen to the event stream at all times, we stay up-to-date.
    getNormalEvents: () => Observable<ReabootEvent>,
    // Returns a stream of progress events (sent much more frequently).
    getProgressEvents: () => Observable<number>,
    // Configures the installation.
    //
    // This will cause the backend to send a basic state change event and is therefore suitable for getting
    // all necessary initial data.
    configure: (config: InstallerConfig) => void,
    // Starts the installation process.
    startInstallation: () => void,
    // Cancels the installation process.
    cancelInstallation: () => void,
}