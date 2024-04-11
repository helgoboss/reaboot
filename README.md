# ReaBoot

ReaBoot is a convenient all-in-one online installer for REAPER, ReaPack and arbitrary packages.

## Development

More dev docs coming soon. User docs are available at https://reaboot.com.

### Regenerate TypeScript and JsonSchema type definitions

`cargo test`

### Generate all icons from initial SVG icon

Do this after changing `commons/src/reaboot-logo.svg`:

1. Rasterize `reaboot-logo.svg` as 1024x1024 pixel PNG (e.g. using Inkscape)
2. Follow https://tauri.app/v1/guides/features/icons/:
   ```sh
   cd gui
   npm run tauri icon ../commons/src/reaboot-logo.png
   ```
3. Copy `reaboot-logo.svg` into `website/src/assets`