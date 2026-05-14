# Components

Frontend components and their responsibilities. Keep components small;
push state into the Zustand store and data into API wrappers.

## TrayPanel

Root of the menu bar window. Owns the section ordering and the footer.
Subscribes to the services store; does not call `invoke` directly.

## ServiceSection

Titled group of services (Running, Pinned, Recently Seen). Handles
empty-section state and the section header. Renders a list of
ServiceCards. Knows nothing about individual services beyond their
status.

## ServiceCard

A single row. Shows status dot, label, port, command preview, primary
action, and an overflow menu. Action handlers are passed in as props
so cards stay dumb. Memoized on `(serviceId, status, port, command)`.

## EmptyState

Centered message + illustration + a single call-to-action. Used by
TrayPanel when no services or ports exist. Accepts `title`,
`description`, and `action`.

## StatusDot

Small colored circle with an `aria-label`. Status values:
`idle`, `running`, `starting`, `stopping`, `error`. Color comes from
`--success`, `--warning`, `--muted`, `--destructive`.

## Button

shadcn-based primitive. Variants: `default`, `secondary`, `ghost`,
`destructive`. Sizes: `sm`, `md`, `icon`. Always reaches `--ring` for
focus.

## Input

shadcn-based primitive. Used in dialogs (Save, Run, Scan). Renders with
a visible label; placeholder is illustrative, not load-bearing.

## ThemeToggle

Three-state control (System / Light / Dark). Persists via the settings
table. Applies by toggling the `dark` class on `<html>`. Re-reads the
system preference on `prefers-color-scheme` changes when set to
System.

## Where things live

```
src/components/
  TrayPanel.tsx
  ServiceSection.tsx
  ServiceCard.tsx
  EmptyState.tsx
  StatusDot.tsx
  ThemeToggle.tsx
  ui/
    button.tsx
    input.tsx
    dialog.tsx
    dropdown-menu.tsx
```

shadcn primitives go under `components/ui/`. DevDock-specific
components stay at the top level.
