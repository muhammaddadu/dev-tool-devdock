# Theme

Tokens live in `src/styles/globals.css`. Everything in the UI reads from
these — no hardcoded colors or radii in components.

## Light tokens

```
--background:          0 0% 100%
--foreground:        240 10% 3.9%
--card:                0 0% 100%
--card-foreground:   240 10% 3.9%
--popover:             0 0% 100%
--popover-foreground:240 10% 3.9%
--primary:           240 5.9% 10%
--primary-foreground:  0 0% 98%
--secondary:         240 4.8% 95.9%
--secondary-foreground: 240 5.9% 10%
--muted:             240 4.8% 95.9%
--muted-foreground:  240 3.8% 46.1%
--accent:            240 4.8% 95.9%
--accent-foreground: 240 5.9% 10%
--destructive:         0 84.2% 60.2%
--destructive-foreground: 0 0% 98%
--border:            240 5.9% 90%
--input:             240 5.9% 90%
--ring:              240 5.9% 10%
--success:           142 71% 45%
--warning:            38 92% 50%
--radius:            0.75rem
```

## Dark tokens

```
--background:        240 10% 3.9%
--foreground:          0 0% 98%
--card:              240 10% 5.5%
--card-foreground:     0 0% 98%
--popover:           240 10% 5.5%
--popover-foreground:  0 0% 98%
--primary:             0 0% 98%
--primary-foreground:240 5.9% 10%
--secondary:         240 3.7% 15.9%
--secondary-foreground: 0 0% 98%
--muted:             240 3.7% 15.9%
--muted-foreground:  240 5% 64.9%
--accent:            240 3.7% 15.9%
--accent-foreground:   0 0% 98%
--destructive:         0 62.8% 50%
--destructive-foreground: 0 0% 98%
--border:            240 3.7% 15.9%
--input:             240 3.7% 15.9%
--ring:              240 4.9% 83.9%
--success:           142 64% 50%
--warning:            38 92% 55%
```

The dark theme activates by adding the `dark` class to `<html>`. The
custom variant in `globals.css` is:

```
@custom-variant dark (&:is(.dark *));
```

## Radius scale

| Token         | Value                       | Usage                |
| ------------- | --------------------------- | -------------------- |
| `--radius`    | `0.75rem`                   | Base radius          |
| `--radius-lg` | `var(--radius)`             | Cards, dialogs       |
| `--radius-md` | `calc(var(--radius) - 2px)` | Buttons, inputs      |
| `--radius-sm` | `calc(var(--radius) - 4px)` | Tags, status pills   |
| `--radius-xl` | `calc(var(--radius) + 4px)` | The TrayPanel itself |

## Font stacks

```
--font-sans: ui-sans-serif, system-ui, -apple-system, "Segoe UI",
             Roboto, sans-serif;
--font-mono: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
```

Sans for everything except commands, paths, PIDs, and ports — those use
mono.

## Theme rules

- Always use `hsl(var(--token))`; never hex literals in components.
- New tokens are added in pairs (light + dark).
- Components must render in both modes — every PR touching UI is
  responsible for visual parity.
