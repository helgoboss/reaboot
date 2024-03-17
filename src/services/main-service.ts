export type MainService = {
    // Returns the resource directory of the main REAPER installation, if one has been found.
    getMainReaperResourceDir: () => Promise<string | undefined>,
}