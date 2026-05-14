# UI Overview

DevDock's primary surface is a single fixed-width panel anchored to the
menu bar icon. Everything the user needs in a normal day fits there.

## TrayPanel layout

```
+--------------------------------------------------+
|  DevDock                       [theme] [settings] |  <- header
+--------------------------------------------------+
|  Running                                          |  <- section
|  +--------------------------------------------+  |
|  | * web      :3000   pnpm dev      [Stop] [>]| |  <- ServiceCard
|  | * api      :8080   cargo run     [Stop] [>]| |
|  +--------------------------------------------+  |
|                                                  |
|  Pinned                                           |
|  +--------------------------------------------+  |
|  | o worker   :6379   redis-server  [Run]  [>]| |
|  +--------------------------------------------+  |
|                                                  |
|  Recently Seen                                    |
|  +--------------------------------------------+  |
|  | o ?        :5173   vite          [Save] [x]| |
|  +--------------------------------------------+  |
+--------------------------------------------------+
|  Last scan 0.4s ago     [Scan project] [Refresh] |  <- footer
+--------------------------------------------------+
```

## Sections

- **Running.** Services with a live `service_runs` row, plus detected
  ports owned by managed services. Always visible (with empty state) if
  the user has saved at least one service.
- **Pinned.** Saved services flagged `pinned=1`. Always visible if any
  pinned services exist.
- **Recently Seen.** Detected ports observed today that are not owned
  by a managed service. Auto-prunes after a configurable window.

## ServiceCard anatomy

```
[status-dot]  [label]   [:port]   [command preview]   [primary] [more]
```

- **Status dot.** Idle / running / starting / stopping / error. Color
  plus `aria-label`.
- **Label.** User-given name or inferred from the project.
- **Port.** First expected port; additional ports shown on expand.
- **Command preview.** Truncated; full command in tooltip and detail.
- **Primary action.** Context-sensitive (Run / Stop / Save / Kill).
- **More.** Menu with Restart, Open URL, Copy URL, Edit, Forget.

## Footer

- Last scan timestamp.
- Scan project (file picker for a project root).
- Manual refresh (forces a watcher tick).
