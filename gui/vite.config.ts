import {defineConfig} from "vite";
import solid from "vite-plugin-solid";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
    plugins: [solid()],

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
        },
    },
    build: {
        // See https://v2.tauri.app/reference/webview-versions/.
        //
        // - macOS: We want to support 10.13+. 10.13 with latest update has Safari 11.1.2 / WebKit 605.3.8.
        // - Windows: WebView2 can be assumed to be very modern.
        // - Linux: All WebKit versions listed on above website are > 605.3.8, so we should be fine (though Linux
        //          is not a target anyway at the moment due to distribution size).
        target: 'safari11',
    },
}));
