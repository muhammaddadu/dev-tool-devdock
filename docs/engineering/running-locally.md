# Running Locally

## Dev loop

```bash
pnpm tauri dev
```

- Frontend (Vite) on `http://localhost:1420`.
- Tauri window auto-reloads on frontend edits.
- Rust code reloads on save (a rebuild is triggered; expect a few
  seconds).

## Useful commands

| Command                                  | What it does                       |
| ---------------------------------------- | ---------------------------------- |
| `pnpm tauri dev`                         | Run app in dev mode                |
| `pnpm dev`                               | Run Vite only (no Tauri)           |
| `pnpm test`                              | Run frontend tests (Vitest)        |
| `cd src-tauri && cargo test`             | Run Rust tests                     |
| `pnpm lint`                              | ESLint + Prettier check            |
| `cd src-tauri && cargo clippy`           | Lint Rust                          |
| `pnpm format`                            | Format frontend                    |
| `cd src-tauri && cargo fmt`              | Format Rust                        |

## Adding a Tauri command

1. Write a function in `src-tauri/src/commands/<area>.rs` annotated with
   `#[tauri::command]`. Keep it thin: validate input, delegate to a core
   engine, return a DTO.
2. Register it in `src-tauri/src/lib.rs` inside
   `tauri::generate_handler![...]`.
3. Add a corresponding wrapper in the frontend API layer
   (`src/lib/api/`) typed against the DTO.
4. Add a Rust test that mocks the platform adapter and exercises the
   command path.
5. Update the relevant doc under `docs/architecture/` if behavior
   changes.

## Logs

| What                 | Where                                                 |
| -------------------- | ----------------------------------------------------- |
| App tracing output   | stderr in dev, or `~/Library/Application Support/DevDock/logs/devdock.log` |
| Per-run service logs | `~/Library/Application Support/DevDock/logs/<run_id>.log` |
| Vite dev logs        | Terminal where `pnpm tauri dev` is running            |

On Linux logs live under `~/.local/share/devdock/logs/`.
