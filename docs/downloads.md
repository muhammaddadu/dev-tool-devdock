# Downloads

Pre-built DevDock binaries are published on the [GitHub Releases page](https://github.com/muhammaddadu/dev-tool-devdock/releases).

> **Status.** macOS is the primary, fully-supported target. Linux is best-effort
> until milestone M8. Windows currently ships a stub adapter only — the app
> builds but service discovery is not implemented. See
> [ADR-0002: macOS First](./decisions/adr-0002-macos-first.md).

## Latest release

The latest stable artifacts are always available at canonical URLs that never
move between versions — safe to link from a website or download button.

| Platform                   | File pattern                                                   | Direct link                                                                                                                            |
| -------------------------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| macOS (Apple Silicon + Intel universal) | `DevDock_*_universal.dmg`                            | [Latest release](https://github.com/muhammaddadu/dev-tool-devdock/releases/latest)                                                              |
| macOS (auto-update bundle) | `DevDock.app.tar.gz` + `latest.json`                            | [`latest.json`](https://github.com/muhammaddadu/dev-tool-devdock/releases/latest/download/latest.json)                                          |
| Linux (Debian/Ubuntu)      | `dev-dock_*_amd64.deb`                                          | [Latest release](https://github.com/muhammaddadu/dev-tool-devdock/releases/latest)                                                              |
| Linux (AppImage)           | `dev-dock_*_amd64.AppImage`                                     | [Latest release](https://github.com/muhammaddadu/dev-tool-devdock/releases/latest)                                                              |
| Linux (RPM)                | `dev-dock-*-1.x86_64.rpm`                                       | [Latest release](https://github.com/muhammaddadu/dev-tool-devdock/releases/latest)                                                              |
| Windows (MSI installer)    | `DevDock_*_x64_en-US.msi`                                       | [Latest release](https://github.com/muhammaddadu/dev-tool-devdock/releases/latest)                                                              |
| Windows (NSIS installer)   | `DevDock_*_x64-setup.exe`                                       | [Latest release](https://github.com/muhammaddadu/dev-tool-devdock/releases/latest)                                                              |

To pin a specific version, replace `latest` with a tag, e.g.
`https://github.com/muhammaddadu/dev-tool-devdock/releases/download/v0.1.0/DevDock_0.1.0_universal.dmg`.

## Verifying downloads

Each release attaches Tauri update signatures (`.sig` files) alongside the
binaries when the release was built with a configured signing key. The auto-updater
uses these automatically; manual verification is described in
[`engineering/release.md`](./engineering/release.md).

## Build from source

If your platform isn't listed or you want the bleeding edge, build locally:

```bash
git clone https://github.com/muhammaddadu/dev-tool-devdock.git
cd devdock
pnpm install
pnpm desktop:build
```

Artifacts land in `src-tauri/target/release/bundle/`.

## How releases are produced

Releases are cut by the [`Release` workflow](../.github/workflows/release.yml).
Push a `v*` tag (or run the workflow manually with a tag input) and it will:

1. Create a draft GitHub Release with auto-generated notes.
2. Build the Tauri bundle for macOS (universal), Linux (Ubuntu 22.04), and Windows in parallel.
3. Attach each platform's installers to the draft release.

Promote the draft to a published release once you've verified the artifacts.
