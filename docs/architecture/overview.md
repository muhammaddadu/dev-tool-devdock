# Architecture Overview

DevDock is a thin React frontend over a Rust core that talks to the OS
through platform adapters and persists state in SQLite.

## Layers

```
+------------------------------------------------------------+
|  Frontend  (React + TS + Vite + Tailwind + shadcn/ui)      |
|                                                            |
|  TrayPanel -> ServiceSection -> ServiceCard / EmptyState   |
|  Zustand store, theme, optimistic UI, no OS access         |
+----------------------------|-------------------------------+
                             | Tauri IPC (invoke)
+----------------------------v-------------------------------+
|  Tauri commands  (src-tauri/src/commands/*.rs)             |
|  Thin handlers: validate, delegate, return DTOs            |
+----------------------------|-------------------------------+
                             |
+----------------------------v-------------------------------+
|  Core engines  (src-tauri/src/core/*)                      |
|                                                            |
|  port_watcher | process_enricher | service_runner |        |
|  project_scanner | ai_router | log_writer                  |
+--------------|----------------|----------------|-----------+
               |                |                |
+--------------v---+   +--------v-------+   +----v-----------+
|  Platform        |   |  Storage       |   |  Filesystem    |
|  adapters        |   |  (SQLx,        |   |  (logs,        |
|  macos/linux/    |   |   SQLite)      |   |   metadata)    |
|  windows_stub    |   |                |   |                |
+------------------+   +----------------+   +----------------+
```

## Responsibilities

- **Frontend.** Presentation, interaction, theme, layout. Never calls OS
  APIs. Reaches the backend only through Tauri commands.
- **Tauri commands.** Type-safe entry points. Validate input, hand off to
  core engines, return DTOs. No business logic.
- **Core engines.** Port watching, enrichment, service lifecycle, project
  scanning, AI routing, log capture. Composed of pure functions where
  possible. No direct OS calls — all OS access goes through adapters.
- **Platform adapters.** OS-specific implementations of the
  `PlatformAdapter` trait. Parse `lsof` / `ss` / `/proc`, send signals,
  resolve cwd. See [`platform-adapters.md`](./platform-adapters.md).
- **Storage.** SQLite via SQLx. Schema is in
  [`storage.md`](./storage.md).
- **Filesystem.** Per-run log files and serialized project metadata under
  the OS app-data directory.

## Data flow examples

- **Port discovery:** watcher tick → adapter `list_listening_sockets` →
  diff against cached snapshot → enrich new PIDs → persist to
  `detected_ports` → emit Tauri event → frontend re-renders.
- **Run service:** UI calls `run_service` → runner spawns child in fresh
  process group → writes `service_runs` row → tails stdout/stderr into a
  log file → emits status events.

## Where to find things

- `src-tauri/src/commands/` — Tauri command surface.
- `src-tauri/src/core/` — engines (port watcher, runner, scanner, ai).
- `src-tauri/src/platform/` — adapters and trait.
- `src-tauri/src/storage/` — SQLx repositories and migrations.
- `src/` — frontend.
