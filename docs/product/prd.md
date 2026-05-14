# Product Requirements

## One-liner

DevDock is a menu bar app that tells developers what is running on their
machine and lets them run, stop, and remember local development services.

## Target user

A working software developer who runs multiple local services per day
(web app, API, database, worker) across several projects, and who currently
loses time hunting down ports with `lsof`, dead terminals, and forgotten
commands.

## Problems we solve

- "What is on port 3000?" — instant answer with PID, command, and cwd.
- "How did I start this thing yesterday?" — remembered, named, one click.
- "Why is this port stuck?" — confirm-and-kill with full context.
- "How do I start everything for project X?" — project group, run all.
- "How do I avoid Electron-class memory bloat?" — Tauri + Rust.

## MVP scope

| Included                                      | Excluded                       |
| --------------------------------------------- | ------------------------------ |
| macOS menu bar app                            | Windows implementation         |
| localhost port discovery                      | Cloud sync                     |
| PID / process / command / cwd enrichment      | Accounts                       |
| Save detected service                         | Team sharing                   |
| Run saved service                             | Remote environments            |
| Stop / restart managed service                | Default shell hooks            |
| Basic log capture for launched services       | Background AI                  |
| Recently detected + pinned + project grouping | Auto-run anything              |
| Deterministic project scanner                 | Background project scans       |
| Optional local AI CLI suggestions (review)    | Auto-applied AI suggestions    |
| Dark / light mode                             | Multi-window dashboards        |

## Platform support

| Platform | Status                       | Notes                            |
| -------- | ---------------------------- | -------------------------------- |
| macOS    | Primary, supported           | Menu bar, lsof, signposted SDK   |
| Linux    | Beta after M8                | Adapter-ready, `ss` + `/proc`    |
| Windows  | Stub only, future            | Trait stubbed, no implementation |

## Success criteria

- A developer can identify any localhost listener in under two seconds.
- A saved service can be re-run tomorrow with one click.
- Background CPU footprint is negligible while the panel is closed.
- No command runs without explicit user intent.
