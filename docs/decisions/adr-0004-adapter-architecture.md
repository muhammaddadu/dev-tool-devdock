# ADR-0004: Platform Adapter Architecture

## Status

Accepted.

## Context

DevDock's core job is to ask the OS questions ("what is listening on
localhost", "who owns PID 4711", "kill this group") and act on the
answers. Each OS has a different mechanism for each of those.

If OS calls are scattered across the core, two things go wrong:

- Adding Linux support means a sprawling search for `lsof` / `ps`
  references.
- Tests need a real machine to run; we cannot exercise edge cases
  deterministically.

## Decision

Define a single `PlatformAdapter` trait
(`src-tauri/src/platform/traits.rs`) that hides all OS-specific calls.
Core engines accept an `Arc<dyn PlatformAdapter>` and never reference a
concrete adapter. Implementations live under `platform::macos`,
`platform::linux`, and `platform::windows_stub`.

Tests use a mock adapter that returns canned `ListeningSocket` and
`ProcessInfo` values.

## Consequences

Positive:

- New platforms are a self-contained module change.
- Engine tests run on any machine without shelling out.
- Parser logic is testable against captured fixtures.
- The "no OS calls in core" rule is enforceable by grep / review.

Negative:

- Some indirection — a new feature touches both an adapter method and
  the core engine that uses it.
- Trait surface must stay stable; adding a method affects all
  implementations including the Windows stub.

Alternatives considered:

- **Cfg-gated functions inline in core.** Rejected — pollutes every
  engine with `#[cfg]` blocks and makes testing platform code on the
  wrong OS impossible.
- **Per-platform crates.** Overkill at this size. Revisit if any
  adapter grows past a few hundred lines.
