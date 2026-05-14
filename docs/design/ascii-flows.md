# ASCII Flows

Compact mockups of the primary panel and key dialogs. Pixel-accurate
positioning is not the point — these capture structure and copy.

## Main panel — populated

```
+--------------------------------------------------+
|  DevDock                       [theme] [settings] |
+--------------------------------------------------+
|  Running                                          |
|  +--------------------------------------------+  |
|  | * web      :3000   pnpm dev      [Stop] [>]| |
|  | * api      :8080   cargo run     [Stop] [>]| |
|  +--------------------------------------------+  |
|  Pinned                                           |
|  +--------------------------------------------+  |
|  | o worker   :6379   redis-server  [Run]  [>]| |
|  +--------------------------------------------+  |
|  Recently Seen                                    |
|  +--------------------------------------------+  |
|  | o ?        :5173   vite          [Save] [x]| |
|  +--------------------------------------------+  |
+--------------------------------------------------+
|  Last scan 0.4s ago     [Scan project] [Refresh] |
+--------------------------------------------------+
```

## Main panel — empty

```
+--------------------------------------------------+
|  DevDock                       [theme] [settings] |
+--------------------------------------------------+
|                                                  |
|               Nothing on localhost.              |
|                                                  |
|        Start a dev server in any terminal —      |
|        DevDock will pick it up automatically.    |
|                                                  |
|              [Scan a project to begin]           |
|                                                  |
+--------------------------------------------------+
|  Idle                              [Refresh now] |
+--------------------------------------------------+
```

## Save dialog

```
+----------- Save service ------------+
| Label    : [ web                  ] |
| Command  : [ pnpm dev             ] |
| Cwd      : [ ~/code/acme/web      ] |
| Ports    : [ 3000                 ] |
| Project  : [ acme              v ]  |
| [ ] Pin to top                      |
|                                     |
|             [ Cancel ]  [ Save ]    |
+-------------------------------------+
```

## Run dialog

```
+--------- Run service ---------------+
|  Label   : web                      |
|  Command : pnpm dev                 |
|  Cwd     : ~/code/acme/web          |
|  Ports   : 3000                     |
|                                     |
|  Logs will be saved to:             |
|    .../logs/<run-id>.log            |
|                                     |
|             [ Cancel ]   [ Run ]    |
+-------------------------------------+
```

## Stop dialog (managed)

```
+--------- Stop service --------------+
|  Label   : web                      |
|  PID     : 48211 (pgid 48211)       |
|  Command : pnpm dev                 |
|  Cwd     : ~/code/acme/web          |
|                                     |
|  SIGTERM, then SIGKILL after 5s.    |
|                                     |
|             [ Cancel ]   [ Stop ]   |
+-------------------------------------+
```

## Kill external dialog (detected)

```
+----- Kill external process ---------+
|  This was NOT started by DevDock.   |
|                                     |
|  PID     : 73019                    |
|  Command : python3 -m http.server   |
|  Cwd     : ~/scratch                |
|  Port    : 8000                     |
|                                     |
|  [ ] Kill the whole process tree    |
|                                     |
|             [ Cancel ]   [ Kill ]   |
+-------------------------------------+
```

## Project scan dialog

```
+--------- Scan project --------------+
|  Root : ~/code/acme                 |
|                                     |
|  Found:                             |
|   [x] web   - pnpm dev    (3000)    |
|   [x] api   - cargo run   (8080)    |
|   [ ] db    - docker compose up     |
|                                     |
|  AI: [ Suggest with Ollama ]        |
|                                     |
|         [ Cancel ]   [ Save 2 ]     |
+-------------------------------------+
```
