# ADR-0001: Tauri v2 + Rust + React

## Status

Accepted.

## Context

DevDock is a desktop utility that runs in the menu bar all day. The
core work — listing sockets, inspecting processes, spawning and
signalling services, parsing OS output — is naturally a systems job. We
considered Electron, Tauri, and a pure-native (Swift) approach.

Constraints:

- Must feel light on the system. A menu bar tool that costs hundreds of
  megabytes of RAM is a non-starter.
- Must be cross-platform-friendly. macOS first, but Linux must be a
  realistic follow-up.
- Must have first-class access to OS-level APIs (signals, process
  groups, `lsof`/`ss`, filesystem).
- Should let us iterate quickly on UI.

## Decision

Build DevDock on **Tauri v2** with a **Rust** backend and a
**React + TypeScript** frontend (Vite, Tailwind v4, shadcn/ui).

## Consequences

Positive:

- Small binary and low memory footprint vs Electron.
- Rust core gives us a strong type system and ecosystem for process,
  filesystem, and parsing work.
- React + shadcn lets us iterate on the panel UI quickly.
- Tauri v2's tray support is mature on macOS and progressing on Linux.

Negative:

- Two languages, two build tools.
- Tauri command surface is a serialization boundary; we need typed
  wrappers on both sides (see `coding-standards.md`).
- Some platform-specific work (e.g. accurate cwd resolution) still
  requires shelling out.

Alternatives considered:

- **Electron.** Rejected for footprint and the desire for a Rust core.
- **Swift / AppKit.** Rejected because it would require a separate
  Linux port from scratch and a different UI stack.
