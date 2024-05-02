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

### Publish new version

1. Increase version number in `Cargo.toml` (`workspace.package.version`) to `X.Y.Z`
2. Commit and push
3. `git tag vX.Y.Z`
4. `git push origin vX.Y.Z`
5. Wait for release draft to be built via GitHub Actions (https://github.com/helgoboss/reaboot/actions)
6. Publish release draft `https://github.com/helgoboss/reaboot/releases`
7. Adjust `LATEST_REABOOT_VERSION` in `website` project (so that ReaBoot website refers to latest downloads)

### Publish website changes

Just push main branch.