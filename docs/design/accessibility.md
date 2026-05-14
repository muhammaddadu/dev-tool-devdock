# Accessibility

DevDock is a keyboard-first tool. The bar is high: a power user should
be able to operate it without the trackpad.

## Focus

- Every interactive element shows a visible focus ring driven by
  `--ring`.
- Never set `outline: none` without an equivalent ring.
- Tab order follows visual order: header → sections (top to bottom,
  cards then their controls) → footer.
- Dialogs trap focus until closed; restore focus to the originating
  control on close.

## Keyboard

- `Enter` / `Space` activate buttons.
- `Esc` closes dialogs and dismisses transient popovers.
- The TrayPanel responds to `R` for refresh and `S` for scan when no
  text field has focus (subject to user setting).

## ARIA

- Icon-only buttons (overflow menu, theme toggle, refresh) have an
  `aria-label` describing the action.
- StatusDot has an `aria-label` like `"status: running"`.
- Dialogs have an `aria-labelledby` pointing at the dialog title and
  `aria-describedby` pointing at the body summary.
- Sections use `<section aria-label="...">` so screen readers can jump
  between them.

## Color independence

- Color is never the only status signal. Status is also encoded in
  text ("Running", "Stopped") and shape (filled vs hollow dot).
- Destructive actions are labelled with both color and verb (`Kill`,
  `Stop`).

## Motion

- Respect `prefers-reduced-motion`. Transitions collapse to instant
  state changes.

## Contrast

- All text/background pairs meet WCAG AA in both themes.
- Muted foreground is for secondary information only, never for
  interactive labels.
