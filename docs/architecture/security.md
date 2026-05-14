# Security

DevDock is local-first and runs commands with the full privilege of the
logged-in user. The threat model is therefore not network — it is
"avoid surprising the user".

## Principles

- **Local-first.** No cloud calls, no telemetry, no remote config.
- **Never silently execute commands.** Every command run by DevDock is
  initiated by an explicit user click. The watcher never runs anything.
- **Never modify the environment.** No edits to shell profiles, no
  installation of dependencies, no auto-`brew install`.
- **Never silently kill process trees.** Killing a tree requires a
  separate confirmation from killing a single PID.
- **Never silently send files to AI tools.** The bundle is shown
  before invocation and is restricted to the allow-list in
  [`ai-integration.md`](./ai-integration.md).

## Destructive action confirmations

Any UI action that can stop or kill a process must show:

- the PID
- the full command line
- the resolved cwd
- a clearly destructive button styled with `--destructive`

The dialog must not be dismissable by clicking outside if it would
otherwise default to "OK".

## AI safety

- AI suggestions are review-only and never auto-applied.
- No source files are sent to providers — only allow-listed manifest
  files.
- The user can disable all AI features in settings; with no provider
  CLI installed the surface is hidden.

## Telemetry

There is no telemetry. No usage reporting. No crash reporting beyond
local log files.

## Logs

Per-run logs live in the app data directory and contain only stdout and
stderr of services the user explicitly launched. DevDock never reads
logs back to the network.
