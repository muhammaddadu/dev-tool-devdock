# AGENTS.md

## Project

DevDock is a macOS-first Tauri desktop app for managing local development services.

It discovers localhost ports, identifies the owning process, remembers commands, and lets users run, stop, restart, open, organize, and inspect local services from the menu bar.

## Primary stack

- Tauri v2
- Rust backend
- React + TypeScript frontend
- Vite
- Tailwind CSS v4
- shadcn/ui
- SQLite
- macOS first
- Linux adapter-ready
- Windows stub only

## Non-negotiable product principles

1. Local-first.
2. No cloud account.
3. No background AI.
4. No commands run without explicit user action.
5. No AI-suggested command is run without review.
6. Background watcher must be lightweight.
7. Expensive process/project inspection only happens on diffs or manual actions.
8. Docs must be updated in the same PR as behavior changes.
9. UI must support dark and light mode.
10. Destructive actions require confirmation.

## CHANGELOG policy

`CHANGELOG.md` is part of the contract — it must stay current.

**On every user-visible change** (new feature, behavior change, bug fix,
removed capability, security fix) add a bullet to the `[Unreleased]` section
of `CHANGELOG.md` in the same change. Categorise under one of:

- `### Added` — new capability the user can use
- `### Changed` — modified existing behavior the user will notice
- `### Deprecated` — feature that's still present but going away
- `### Removed` — feature that's gone
- `### Fixed` — bug fix
- `### Security` — vulnerability fix

Internal-only changes (refactors with no user-visible effect, test additions,
doc-only updates, code-style fixes) do **not** require a CHANGELOG entry. The
goal is what shipped to users, not what we did to get there.

**On version bump** (modifying `version` in `package.json` and
`src-tauri/tauri.conf.json`):

1. Rename the `[Unreleased]` section to `[X.Y.Z] - YYYY-MM-DD` using the
   release date.
2. Add a fresh empty `[Unreleased]` section above it with the
   "_Nothing yet_" placeholder.
3. Update the compare-link footnote at the bottom: the previous
   `[Unreleased]` link becomes `[X.Y.Z]` pointing at the tag, and a new
   `[Unreleased]` compare link is added pointing at `vX.Y.Z...HEAD`.

The same agent that bumps the version performs this rename — never leave it
for "later".

## Docs policy

All user-facing or architectural changes must update docs.

Required docs locations:

- Product changes: `docs/product/`
- Architecture changes: `docs/architecture/`
- Setup/testing changes: `docs/engineering/`
- UI/UX changes: `docs/design/`
- Important technical decisions: `docs/decisions/`

When changing behavior, update:

- relevant docs file
- README if needed
- tests
- type definitions
- ASCII UI examples if UI flow changes

## Current MVP scope

Included:

- macOS menu bar app
- localhost port discovery
- PID/process/command/cwd detection
- save detected service
- run saved service
- stop/restart managed service
- basic logs
- recently detected services
- pinned services
- project grouping
- deterministic project scan
- optional local AI CLI suggestions
- dark/light mode

Excluded:

- Windows implementation
- cloud sync
- accounts
- team sharing
- remote environments
- shell hooks by default
- background AI

## Architecture rules

The core domain layer must not call OS-specific commands directly.

Use platform adapters:

- `platform::traits`
- `platform::macos`
- `platform::linux`
- `platform::windows_stub`

All OS-specific code belongs inside platform adapters.

The frontend must call backend behavior through Tauri commands only.

The backend owns:

- port detection
- process inspection
- command execution
- service lifecycle
- SQLite persistence
- log writing
- project scanning

The frontend owns:

- presentation
- interactions
- theme
- layout
- optimistic UI where safe

## Performance rules

Background watcher:

- Cheap socket scan only.
- Diff against previous snapshot.
- Enrich only new or changed PIDs.
- Cache process metadata by PID plus start time when available.
- Do not run project scans in background.
- Do not run AI in background.

Polling:

- panel open: 1–2 seconds
- background idle: 10–15 seconds
- post-launch fast polling: 250–500ms for max 10 seconds
- backoff after no changes

## Security rules

Never silently:

- execute commands
- modify shell profiles
- install dependencies
- kill process trees
- send files to AI tools
- alter project files

For destructive actions:

- show PID
- show command
- show cwd
- require confirmation

## UI rules

UI should be compact, beautiful, and practical.

Use:

- clear hierarchy
- rounded cards
- keyboard-accessible controls
- readable spacing
- dark/light mode
- status indicators
- clear destructive action styling
- empty states
- loading states
- error states

Do not overdecorate. This is a developer utility.

## Testing requirements

Rust:

- unit tests for parsers
- unit tests for command inference
- unit tests for project scanner
- integration tests for service repository
- mocked platform adapter tests

Frontend:

- component tests
- state tests
- accessibility checks where practical

E2E:

- basic app launch
- theme toggle
- mocked service list render
- save service flow
- run confirmation flow

## Definition of done

A task is complete only when:

- feature works
- tests pass
- docs updated
- errors handled
- no unnecessary polling added
- UI works in dark and light mode
- destructive actions are confirmed
- no regressions in existing flows

## Commands

Install:
`pnpm install`

Dev:
`pnpm tauri dev`

Frontend tests:
`pnpm test`

Rust tests:
`cd src-tauri && cargo test`

Lint:
`pnpm lint`
`cd src-tauri && cargo clippy -- -D warnings`

Format:
`pnpm format`
`cd src-tauri && cargo fmt`

## Preferred implementation order

1. Docs scaffold.
2. Tauri app shell.
3. Theme system.
4. SQLite schema.
5. Platform adapter traits.
6. macOS port discovery.
7. process enrichment.
8. service list UI.
9. save detected service.
10. run saved service.
11. logs.
12. stop/restart.
13. project scanner.
14. optional AI CLI detection.
15. Linux adapter.
