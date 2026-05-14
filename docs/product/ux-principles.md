# UX Principles

DevDock is a utility, not a product showcase. Keep the surface area small
and the feedback loop tight.

## Compact

- Menu bar panel is fixed-width and tall enough to show three sections
  without scroll on a typical setup.
- One service per row. No double-stacked metadata blocks.
- Truncate long commands; full text is available on hover or in detail.

## Calm

- No motion that draws the eye without reason.
- No toasts for routine actions (a card already updates).
- Status changes animate by color/dot, not by movement.

## Fast

- Panel open should render the cached snapshot immediately, then refresh.
- Optimistic UI is allowed for non-destructive actions.
- Polling intervals respect the rules in
  [`architecture/performance.md`](../architecture/performance.md).

## Trustworthy

- Never run a command the user did not click.
- Destructive actions always show PID, command, cwd, and a confirmation
  step.
- Errors are surfaced inline on the affected card, not in a modal.

## Keyboard-friendly

- Every interactive control is reachable by Tab.
- Focus ring uses `--ring`; never removed.
- Common actions have shortcuts (Run, Stop, Open URL).

## Dark / light parity

- All states (idle, running, error, confirming) are designed in both
  themes.
- Color is never the only signal — every status dot has an `aria-label`.
