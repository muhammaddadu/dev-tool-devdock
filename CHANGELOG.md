# Changelog

All notable changes to DevDock are recorded here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

_Nothing yet — add user-visible changes under the appropriate heading_
_(`Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`)._

## [0.1.0] - 2026-05-28

First MVP release. Tray-only menu bar app for macOS; Linux adapter scaffolded
but not implemented yet.

### Added

#### Service discovery
- Localhost TCP listener discovery via `lsof -nP -iTCP -sTCP:LISTEN` with
  IPv4/IPv6 dedupe per `(pid, port)`.
- Process enrichment with `sysinfo` for the batch path and per-PID `ps` / `lsof`
  fallbacks for fields macOS's libproc returns empty.
- Project root inference: walks ancestors looking for `.git`, `package.json`,
  `Cargo.toml`, `pyproject.toml`, `Gemfile`, `go.mod`, and friends.
- Smart project naming: prefers manifest `name` (package.json / Cargo.toml),
  strips npm scopes (`@acme/web` → `web`), and walks past generic folder names
  (`src`, `app`, `apps`, `packages`, `web`, `api`, …) to a meaningful ancestor
  (`monorepo/web`).
- Three-bucket classification: **Dev** (foreground), **Tooling** (IDE helpers,
  language servers, `adb`, Docker Desktop, OrbStack, anything in `.app/Contents/`,
  and DevDock itself in dev mode), **System** (Apple daemons, consumer apps).

#### Saved services
- SQLite persistence at `~/Library/Application Support/DevDock/devdock.sqlite`
  with WAL journal and foreign keys.
- Save the currently-detected service from the Save button on its card.
- Add Command dialog for entries with no currently-running process; native
  folder picker via `tauri-plugin-dialog`.
- Edit any saved service to change label, command, cwd, or expected ports.
- Pin / Unpin / Forget actions from the card's ⋯ menu.

#### Managed runtime
- Run, Stop, and Restart for saved services. Spawns via `$SHELL -ic` in the
  saved cwd so PATH matches what Terminal sees (`nvm`, `asdf`, etc.).
- Own process group per spawn — `kill(-pid, SIGTERM)` reaps the entire
  descendant tree (e.g. Vite → esbuild workers).
- SIGTERM with SIGKILL escalation after 5 s for stubborn services.
- stdout / stderr captured to per-run log files at
  `~/Library/Application Support/DevDock/logs/<run-id>.log`.
- Live logs viewer with auto-poll, sticky scroll-to-bottom, "open in editor"
  fallback. Sandboxed to the logs directory.
- Kill action for externally-running services with full PID / command / cwd
  confirmation dialog.

#### UI
- Tray-only menu-bar app (no Dock icon via `ActivationPolicy::Accessory`).
- Panel anchored under the tray icon, clamped to the monitor.
- Compact, calm cards with status dot, label, port chip, age, command line,
  cwd, project tag.
- Three sections: Running (foreground Dev) · Pinned · Recently Seen · with
  Tooling and System as collapsible groups below.
- Foreground-quiet inline note when nothing Dev is running but background
  services exist.
- Auto-expand Tooling when foreground is empty so it doesn't feel dead.
- Compact command display: `node /Users/me/.nvm/.../vite.js` → `vite.js`;
  collapses absolute-path args to basenames, drops `node`/`python3` when a
  script follows.
- ⋯ menu: View Logs · Copy URL · Copy command (with `cd '<cwd>' &&` prefix) ·
  Open in *each detected editor* (VS Code / Cursor / Windsurf / Zed / Sublime /
  Fleet) · Open in Terminal · Reveal in Finder · Edit · Pin/Unpin · Forget.
- Tray-icon badge showing count of foreground Dev services.
- Dark / light / system theme.
- Keyboard shortcuts: Esc to hide panel, ⌘R to refresh, ⌘F to focus search.
- Auto-hide on focus loss (menu-bar convention).

#### Performance
- Pre-load: 10 s background tick while the panel is hidden, 1.5 s while
  focused. Cache always warm by the time the user clicks the tray.
- Process metadata fetched once per refresh in a single batch.
- Project-root walks cached forever by cwd.
- `lsof` parsed in pure Rust; no per-line shellouts.

#### Settings
- Start at login (macOS LaunchAgent via `tauri-plugin-autostart`).
- Refresh interval slider (500 ms – 60 s) — persisted to SQLite.
- Theme picker.
- Manual update check — wired to `tauri-plugin-updater` (inert until you
  configure a signing key and host updates; see `docs/engineering/release.md`).

#### Reliability
- Quit confirmation when DevDock is managing services. Cancel / Quit anyway /
  Stop & quit. Stop & quit sends SIGTERM to every managed group and waits up
  to 3 s for clean shutdown before exiting.
- Production dev-root detection: skip self-classification entirely when the
  binary lives inside an `.app` bundle (release builds), avoiding false
  positives.

#### Security
- Kill command refuses pid=0 and refuses DevDock's own pid.
- Log reader sandboxed to `~/Library/Application Support/DevDock/logs/`;
  canonicalized paths reject `..` and symlink escapes.
- URL opener restricted to `http://` and `https://` schemes.
- Auto-updater verifies payloads against the embedded `tauri-signer` public
  key.

### Architecture
- `PlatformAdapter` trait isolates all OS-specific calls; macOS implementation
  uses `lsof` + `sysinfo` + `ps` fallbacks. Linux and Windows stubs return
  empty / `NotSupported`.
- Pure `service_engine` merges detected sockets, saved services, and managed
  runtime state into the panel's `ServiceView` list. Fully unit-tested.
- 56 Rust tests + 2 frontend tests; `cargo clippy --all-targets -- -D warnings`
  clean.

### Known limitations
- macOS only. Linux adapter is a stub.
- Project scanner deferred post-MVP — the button is not surfaced anywhere in
  the UI.
- AI provider integration deferred post-MVP — `detect_ai_providers` returns
  empty.
- Run history (`service_runs` table) tracked in memory only; logs are only
  reachable while the run is alive.

[Unreleased]: https://github.com/REPLACE_ME/devdock/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/REPLACE_ME/devdock/releases/tag/v0.1.0
