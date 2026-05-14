# Storage

DevDock stores state in a single SQLite database accessed via SQLx from
the Rust core. The frontend never writes to disk; it only reads via
Tauri commands.

## On-disk locations

| Platform | App data directory                            |
| -------- | --------------------------------------------- |
| macOS    | `~/Library/Application Support/DevDock/`      |
| Linux    | `~/.local/share/devdock/`                     |

Inside that directory:

- `devdock.sqlite` — primary database.
- `logs/` — per-run log files, one per `service_runs` row.
- `cache/` — enrichment and scanner caches (regeneratable).

## Schema

The canonical schema lives in
`src-tauri/migrations/0001_initial.sql`. Reproduced here:

```sql
CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS services (
  id TEXT PRIMARY KEY,
  project_id TEXT,
  label TEXT NOT NULL,
  command TEXT NOT NULL,
  cwd TEXT NOT NULL,
  expected_ports TEXT NOT NULL,
  pinned INTEGER NOT NULL DEFAULT 0,
  created_from TEXT NOT NULL,
  last_run_at TEXT,
  last_seen_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS service_runs (
  id TEXT PRIMARY KEY,
  service_id TEXT NOT NULL,
  root_pid INTEGER,
  process_group_id INTEGER,
  status TEXT NOT NULL,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  exit_code INTEGER,
  log_path TEXT,
  FOREIGN KEY (service_id) REFERENCES services(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS detected_ports (
  id TEXT PRIMARY KEY,
  port INTEGER NOT NULL,
  host TEXT NOT NULL,
  pid INTEGER,
  process_name TEXT,
  command_line TEXT,
  cwd TEXT,
  project_root TEXT,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ignored_ports (
  id TEXT PRIMARY KEY,
  port INTEGER NOT NULL,
  host TEXT,
  reason TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_suggestions (
  id TEXT PRIMARY KEY,
  project_id TEXT,
  provider TEXT NOT NULL,
  input_hash TEXT NOT NULL,
  output_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_services_pinned ON services(pinned);
CREATE INDEX IF NOT EXISTS idx_service_runs_service ON service_runs(service_id);
CREATE INDEX IF NOT EXISTS idx_detected_ports_port ON detected_ports(port);
```

## Table purposes

- **projects** — project roots the user has saved or scanned. Acts as a
  grouping anchor for services.
- **services** — user-saved runnable services. `expected_ports` is a
  JSON-encoded array of integers. `created_from` records origin
  (`detected`, `scan`, `manual`).
- **service_runs** — one row per launch attempt. Holds PID/PGID for the
  live process, plus `log_path` to the per-run log file.
- **detected_ports** — last-seen state of every listener observed by the
  watcher; pruned on a schedule.
- **ignored_ports** — entries the user dismissed from the panel; the
  watcher keeps observing but the UI hides them.
- **settings** — key/value runtime settings (poll intervals, theme,
  stop timeout).
- **ai_suggestions** — cached output from AI providers keyed by an
  `input_hash` so identical scans do not re-query.

## Migrations

Migrations are SQL files under `src-tauri/migrations/`, applied in
filename order at startup. Add a new file rather than editing an
existing one.
