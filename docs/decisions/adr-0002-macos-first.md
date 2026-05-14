# ADR-0002: macOS First

## Status

Accepted.

## Context

DevDock targets developers running services on their workstation. The
team's primary workstations are macOS. macOS also has a well-defined
and stable menu bar story; Linux tray support varies by desktop
environment (GNOME ships extensions, KDE has SNI, others differ).

We need to ship something useful end-to-end before broadening the
target list, and the menu bar is central to the product.

## Decision

Treat **macOS as the primary target** through MVP. Maintain Linux as an
**adapter-ready** path — interfaces and tests exist from M0 — but
defer the Linux implementation to M8. Windows is a **stub only** with
no implementation planned for MVP.

## Consequences

Positive:

- Faster feedback loop on the team's daily machines.
- One menu bar implementation to design, test, and polish.
- Clear definition of done: macOS scope only.

Negative:

- Linux users wait until M8 for a working build.
- We may discover macOS-specific assumptions during M8 that need
  refactoring (mitigated by the adapter trait).

Alternatives considered:

- **Cross-platform from day one.** Rejected — too much surface area for
  too little user feedback.
- **macOS only forever.** Rejected — Linux is a meaningful share of the
  target user base; we keep the door open via the adapter trait.
