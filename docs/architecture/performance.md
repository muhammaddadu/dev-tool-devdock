# Performance

DevDock runs in a developer's menu bar all day. The background cost
must be negligible.

## Watcher rules

- Only cheap socket scans run on a timer.
- Enrichment runs only when the cheap scan reports a diff.
- Project inference runs only on enrichment of a new PID.
- AI providers never run in the background.
- Project scanner never runs in the background.
- No filesystem traversal in the background beyond what the cheap scan
  needs.

## Polling intervals

| Context              | Interval        | Notes                                    |
| -------------------- | --------------- | ---------------------------------------- |
| Panel open           | 1–2 s           | User is watching                         |
| Background idle      | 10–15 s         | Cheap scan only                          |
| Post-launch (≤ 10 s) | 250–500 ms      | Catch newly-bound ports                  |
| After post-launch    | Backoff to 30 s | When no diff for several ticks           |
| AI                   | Never           | Only on explicit user action             |
| Project scan         | Never           | Only on explicit user action             |

## Cache keys

| Cache                       | Key                          | Reason                          |
| --------------------------- | ---------------------------- | ------------------------------- |
| Process enrichment          | `(pid, started_at)`          | Defeat PID reuse                |
| Project root inference      | `cwd` (resolved real path)   | Reuse across PIDs in same dir   |
| AI suggestions              | `input_hash` of bundle       | Stable across identical scans   |
| Listening sockets snapshot  | Previous tick's set          | Diff against current scan       |

## What the watcher must not do

- Read process memory.
- Resolve cwd for every PID on every tick — only for new PIDs.
- Walk project trees recursively.
- Hash files for AI bundles.
- Open log files for inspection.
- Make network requests.

## Frontend performance

- Render from in-memory store on panel open; do not block on a
  round-trip.
- Use Tauri events for live updates, not polling from the frontend.
- Memoize ServiceCard rows by service id + last-known status.
