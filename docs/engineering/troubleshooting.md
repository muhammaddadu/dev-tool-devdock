# Troubleshooting

## `lsof: permission denied`

Symptom: macOS adapter returns `PermissionDenied` or rows have no PID.

Cause: `lsof` cannot see processes owned by other users (including
some system daemons).

Fix: DevDock is expected to only manage the logged-in user's processes.
If a port belongs to another user it will show with PID `unknown`. Do
not run DevDock as root to "fix" this — it would break the threat
model.

## A port is open but does not appear in the panel

Checks, in order:

1. Is the watcher actually running? Open the panel and look at the
   footer's last-scan timestamp.
2. Is the socket bound to a non-localhost address? DevDock filters to
   localhost listeners by default.
3. Is the port in `ignored_ports`? Open settings and clear the
   ignore list.
4. Is the adapter parser failing on an exotic `lsof` row? Check
   `devdock.log` for `Parse` errors and capture the line as a fixture.

## A managed service will not stop

Checks, in order:

1. The stop sequence is SIGTERM → wait → SIGKILL. If the process is
   stuck in uninterruptible I/O, SIGKILL may not return immediately.
2. Confirm the `service_runs` row has a `process_group_id`. If null,
   the child was not spawned in a new process group — file a bug.
3. As a last resort, kill from a terminal with the PID shown on the
   card and report what DevDock did wrong.

## AI provider not detected

Checks:

1. Is the binary on `PATH` for the user running DevDock? Run
   `which claude` (or equivalent) in the same shell that launches
   DevDock.
2. macOS GUI apps inherit a minimal `PATH`. If the binary lives in
   `~/.local/bin` or a custom location, add it to a launchd `plist` or
   `/etc/paths.d/` so DevDock can see it.
3. Click "Re-detect providers" in settings to force a rescan.

## Frontend won't connect to Vite

Symptom: blank window in `pnpm tauri dev`.

Cause: Vite usually runs on `1420`; if the port is taken it picks the
next free one and Tauri loses the reference.

Fix: free port `1420` (DevDock can help here) and restart
`pnpm tauri dev`.
