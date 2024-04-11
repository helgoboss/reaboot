# ReaBoot

## Development

### Regenerate TypeScript and JsonSchema type definitions

`cargo test`

### Generate all icons from initial SVG icon

1. Rasterize `commons/src/reaboot-logo.svg` as 1024x1024 pixel PNG (e.g. using Inkscape)
2. Follow https://tauri.app/v1/guides/features/icons/:
   ```sh
   cd gui
   npm run tauri icon ../commons/src/reaboot-logo.png
   ```