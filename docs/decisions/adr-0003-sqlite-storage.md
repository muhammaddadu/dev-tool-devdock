# ADR-0003: SQLite Storage

## Status

Accepted.

## Context

DevDock stores saved services, detected ports, run history, settings,
and cached AI suggestions. The data is per-user, local, and modest in
size. We considered flat JSON files, a key-value store (sled), and an
embedded SQL database (SQLite).

Requirements:

- Local-first; no server.
- Survive crashes and concurrent reads from background watcher + UI.
- Easy to migrate as the schema grows.
- Queryable for things like "services in this project" and "runs for
  this service in the last 24 hours".

## Decision

Use **SQLite** via the **SQLx** crate, accessed only from the Rust
backend. Migrations live in `src-tauri/migrations/`. The frontend never
touches the database directly — it goes through Tauri commands.

## Consequences

Positive:

- Mature, embedded, well-understood. No daemon to manage.
- SQL is a good fit for the cross-table queries we need.
- SQLx provides compile-time-checked queries and async I/O.
- Single backup target: one file.

Negative:

- Schema changes require migration files; rolling back requires care.
- SQLite write concurrency is single-writer — fine here because all
  writes go through the Rust core, not the frontend.

Alternatives considered:

- **JSON files.** Rejected — concurrent writes, partial corruption
  risk, and querying gets ugly fast.
- **sled / redb.** Rejected — relational queries are useful for the
  service/run/project graph; SQL is the path of least surprise.
