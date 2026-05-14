# Setup

## Prerequisites

- Node.js 20 or newer
- pnpm 9 or newer
- Rust toolchain via `rustup` (stable channel)
- macOS: Xcode command line tools (`xcode-select --install`)
- Linux (optional, M8): `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`,
  `libayatana-appindicator3-dev`, `librsvg2-dev`, `build-essential`,
  `curl`, `wget`, `file`

## Install

```bash
pnpm install
```

This installs frontend dependencies. Cargo dependencies are fetched on
first `tauri dev` / `tauri build`.

## Run the desktop app

```bash
pnpm tauri dev
```

Vite serves the frontend at `http://localhost:1420` and Tauri opens the
desktop window pointed at it. The Rust backend rebuilds on save.

## Regenerate placeholder icons

The repo ships with a placeholder icon set. To regenerate from the SVG
source after editing it:

```bash
pnpm tauri icon src-tauri/icons/icon.svg
```

This writes `.icns`, `.ico`, and `.png` variants into
`src-tauri/icons/`.

## Verify the install

```bash
pnpm lint
cd src-tauri && cargo clippy -- -D warnings && cargo test
```

All three should pass on a fresh checkout.
