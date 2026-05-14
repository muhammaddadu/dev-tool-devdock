# DevDock

A macOS-first Tauri desktop app that lives in the menu bar and helps developers manage local development services.

## What it does

- Discovers what is listening on localhost
- Identifies the owning PID, process, command, and cwd
- Remembers commands so you can run/stop/restart them tomorrow
- Captures logs for services it launched
- Optional local AI CLI suggestions for project scans

## Quickstart

```bash
pnpm install
pnpm tauri dev
```

Prerequisites:

- Node 20+
- pnpm 9+
- Rust toolchain (`rustup`, `cargo`)
- Xcode command line tools (macOS)

## Docs

See [`docs/`](./docs/README.md) for product, architecture, engineering, design, and decision docs.

The contract for AI coding agents is in [`AGENTS.md`](./AGENTS.md).

## PR checklist

- [ ] Tests added/updated
- [ ] Docs updated
- [ ] UI checked in dark mode
- [ ] UI checked in light mode
- [ ] Destructive actions confirmed
- [ ] No new background expensive work
