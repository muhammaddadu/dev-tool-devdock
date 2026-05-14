# Port Detection

Detection is a layered pipeline designed to do as little work as possible
in the background.

## Pipeline

```
cheap socket scan  ->  diff vs previous snapshot
                              |
                              v
                  enrich only new / changed PIDs
                              |
                              v
                cache by (pid, started_at)
                              |
                              v
              infer project root + likely command
                              |
                              v
                  persist + emit Tauri event
```

1. **Cheap socket scan.** Adapter returns `Vec<ListeningSocket>` of
   `{port, host, pid?}` only. No process names, no cwd, no command line.
2. **Diff.** Compare to the previous snapshot keyed by
   `(port, host, pid)`. Drop unchanged entries.
3. **Enrich.** For each new PID, call `get_process_info` to resolve
   process name, full command line, cwd, and start time.
4. **Cache.** Store enrichment results in memory keyed by
   `(pid, started_at)` to defeat PID reuse (see
   [`process-management.md`](./process-management.md)).
5. **Infer.** Walk up from cwd to find a project root (presence of
   `.git`, `package.json`, `Cargo.toml`, `pyproject.toml`, etc.) and
   guess the originating command if it matches a known framework.
6. **Persist + emit.** Upsert into `detected_ports`; emit an event for
   the frontend.

## Platform commands

| Platform | Command                                          | Notes                              |
| -------- | ------------------------------------------------ | ---------------------------------- |
| macOS    | `lsof -nP -iTCP -sTCP:LISTEN`                    | Single shell-out per tick          |
| Linux    | `ss -ltnp` + `/proc/<pid>/{cmdline,cwd,comm,stat}` | `cwd` requires read on `/proc`     |

The macOS parser must handle multi-PID lines and IPv4/IPv6 columns. The
Linux parser must tolerate `ss` output with no `users:` column (no
permission to see PIDs) and degrade to "unknown PID".

## Polling intervals

| State                | Interval        | Rationale                            |
| -------------------- | --------------- | ------------------------------------ |
| Panel open           | 1–2 s           | User is watching, responsiveness     |
| Background idle      | 10–15 s         | Negligible CPU footprint             |
| Post-launch (≤ 10 s) | 250–500 ms      | Catch the newly-bound port quickly   |
| After post-launch    | Backoff to 30 s | Idle if nothing changed              |

Background watcher must never run enrichment, project inference, or AI on
its own clock — those only run when the cheap scan reports a diff.
