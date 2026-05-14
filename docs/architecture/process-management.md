# Process Management

DevDock tracks two classes of processes.

## Managed vs detected

| Class    | Source                              | Lifecycle DevDock owns |
| -------- | ----------------------------------- | ---------------------- |
| Managed  | Spawned by DevDock via `run_service` | Yes                    |
| Detected | Found by the port watcher           | No                     |

The two have different rules. DevDock can freely stop, restart, and
recreate **managed** services. For **detected** processes DevDock can
observe them and, on explicit confirmation, send a signal — but it never
kills a process tree it did not spawn.

## Process groups

Every managed service is spawned in its own process group (`setsid` on
Unix). Stopping the service sends the signal to the group, not the
parent PID, so child workers do not survive.

```
run_service(service_id)
  -> spawn(command, cwd, setsid)
  -> persist service_runs row (root_pid, process_group_id, started_at)
  -> attach stdout/stderr to log file
```

## Graceful stop

```
stop_service(service_id)
  -> SIGTERM to -pgid
  -> wait up to N seconds (default 5)
  -> if still alive: SIGKILL to -pgid
  -> mark service_runs.status = "stopped", set ended_at + exit_code
```

The timeout is configurable via `settings`. The frontend shows a
"stopping..." state until the row updates.

## Killing detected processes

`kill_detected_process` requires a confirmation dialog that shows PID,
command, and cwd. The default action is **SIGTERM to the PID only**, not
the tree, to avoid catastrophic kills of, say, a parent shell.
Tree-kill is only available behind a separate, clearly-labeled action.

## PID reuse safety

PIDs are reused. Naively caching enrichment by `pid` is unsafe — the
process at PID 4711 today is not the process at PID 4711 in an hour.

The cache key is `(pid, started_at)`. `started_at` comes from
`/proc/<pid>/stat` field 22 on Linux and `ps -o lstart=` on macOS.

When the watcher sees a PID with a different `started_at` than cached,
the entry is invalidated and re-enriched.

## Restart

`restart_service` = `stop_service` + `run_service` with the same
params. It produces a new `service_runs` row so logs and exit codes are
not lost.
