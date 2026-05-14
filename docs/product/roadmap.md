# Roadmap

Milestones are ordered. Each one has explicit acceptance criteria. A
milestone is done only when its criteria and the global definition of done
in [`AGENTS.md`](../../AGENTS.md) are both satisfied.

## M0 — Scaffold (done)

- Tauri v2 app builds and launches.
- React + Vite + Tailwind v4 + shadcn/ui render.
- SQLite migrations run on startup.
- Platform adapter trait compiles for macOS, Linux, and the Windows stub.

## M1 — macOS port detection

- `list_listening_sockets` returns parsed `lsof` output.
- Frontend renders a flat list of detected ports.
- Cheap socket scan diffs against previous snapshot.
- Acceptance: starting `python3 -m http.server 8000` shows up within one
  poll interval; closing it removes it within one poll interval.

## M2 — Process enrichment

- For each new PID resolve process name, full command line, and cwd.
- Cache by `(pid, started_at)` to survive PID reuse.
- Acceptance: each detected port shows command and cwd; enrichment never
  runs for already-cached PIDs.

## M3 — Save detected service + SQLite

- "Save" dialog persists `services` rows with label, command, cwd,
  expected ports.
- Saved services appear in the Pinned section.
- Acceptance: closing and reopening the app preserves saved services.

## M4 — Run saved service + logs

- Tauri command spawns the saved command in its `cwd` under a fresh
  process group.
- Stdout and stderr are written to a per-run log file.
- Acceptance: clicking Run starts the service and the log file streams
  into the UI.

## M5 — Stop / restart

- Graceful SIGTERM, then SIGKILL after a configurable timeout.
- Restart = stop + run with the same params.
- Killing a detected (non-managed) process requires full confirmation.
- Acceptance: stop returns control within the timeout window; restart
  produces a new `service_runs` row.

## M6 — Project scanner

- Manual, deterministic scan of a directory.
- Parses `package.json` scripts, `docker-compose.yml` services, `Procfile`,
  `Makefile` targets, common framework configs.
- Acceptance: scanning a known fixture project produces the expected list
  of inferred services.

## M7 — Optional local AI CLI

- Detect installed AI CLIs (Claude, Codex, Cursor, Ollama) by binary
  presence.
- User can invoke a provider on an allow-listed metadata bundle.
- Output is review-only; never auto-applied.
- Acceptance: with no CLI installed, the feature is hidden; with one
  installed, suggestions appear in a review pane only.

## M8 — Linux adapter

- `ss -ltnp` plus `/proc/<pid>/{cmdline,cwd,comm,stat}` parsing.
- Same trait surface as macOS.
- Acceptance: parity test suite for the adapter passes on a Linux runner.
