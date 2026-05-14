# Testing

## Rust

Run from `src-tauri/`:

```bash
cargo test
```

Test layout:

- `src/tests/` — integration tests.
- `src/tests/fixtures/` — captured `lsof` / `ss` / `/proc` output,
  `package.json` / `Procfile` / etc. for scanner tests.
- `#[cfg(test)] mod tests` blocks live next to the parser they cover.

What each layer covers:

- **Adapter parsers.** Feed a fixture string in, assert the parsed
  `ListeningSocket` / `ProcessInfo` vector.
- **Command inferer.** Given known framework signals (Vite, Next, Cargo
  run, gunicorn), assert it guesses the right command and label.
- **Project scanner.** Given a temp dir mirroring a fixture project,
  assert the list of inferred services.
- **Service repository.** Spin up an in-memory SQLite, run migrations,
  assert CRUD round-trips.
- **Core engines.** Use a mock `PlatformAdapter` that returns canned
  results; assert diffing, enrichment caching, and event emission.

## Mock platform adapter pattern

```rust
struct MockAdapter {
    sockets: Vec<ListeningSocket>,
    processes: HashMap<u32, ProcessInfo>,
}

impl PlatformAdapter for MockAdapter {
    fn list_listening_sockets(&self) -> PlatformResult<Vec<ListeningSocket>> {
        Ok(self.sockets.clone())
    }
    // ... other methods return canned data
}
```

Inject via `Arc<dyn PlatformAdapter>` wherever the engine is
constructed. No code path should rely on platform detection at test
time.

## Frontend

Run from the repo root:

```bash
pnpm test
```

Stack: Vitest + React Testing Library.

What to cover:

- **Components.** ServiceCard renders the right status dot for each
  status, EmptyState renders, StatusDot has an `aria-label`.
- **State.** Zustand store reducers; theme toggle persists.
- **API wrappers.** `invoke` is mocked; assert the wrapper returns the
  typed DTO and surfaces errors.
- **Accessibility.** Smoke checks: focus on Tab, `aria-label` on icon
  buttons, no `outline: none` without a replacement.

## End-to-end (later)

Targeted via `tauri-driver` once stabilized:

- App launches.
- Theme toggle persists across launches.
- Mocked service list renders.
- Save service flow.
- Run confirmation flow.
