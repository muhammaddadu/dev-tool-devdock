# Platform Adapters

All OS-specific behavior is hidden behind the `PlatformAdapter` trait in
`src-tauri/src/platform/traits.rs`. The core layer must never shell out
to OS tools directly.

## Trait

```rust
pub trait PlatformAdapter: Send + Sync {
    fn list_listening_sockets(&self) -> PlatformResult<Vec<ListeningSocket>>;
    fn get_process_info(&self, pid: u32) -> PlatformResult<Option<ProcessInfo>>;
    fn list_processes(&self) -> PlatformResult<Vec<ProcessInfo>>;
    fn terminate_process(&self, pid: u32) -> PlatformResult<()>;
    fn terminate_process_tree(&self, pid: u32) -> PlatformResult<()>;
}
```

Errors are reported via `PlatformError` (`NotSupported`,
`PermissionDenied`, `CommandFailed`, `Parse`, `Io`).

## Implementations

| Platform     | Module                       | Status      | Mechanism                                  |
| ------------ | ---------------------------- | ----------- | ------------------------------------------ |
| macOS        | `platform::macos`            | Supported   | `lsof -nP -iTCP -sTCP:LISTEN`, `ps`, `kill`|
| Linux        | `platform::linux`            | Beta (M8)   | `ss -ltnp`, `/proc/<pid>/{cmdline,cwd,...}`|
| Windows stub | `platform::windows_stub`     | Stub only   | All methods return `PlatformError::NotSupported` |

## Rules

- Core engines must accept an `Arc<dyn PlatformAdapter>` (or equivalent)
  and never reference a concrete adapter.
- Adapter modules must not import from `core::` — they translate OS
  output to neutral models in `models::`.
- Each adapter has a parser unit-tested against fixture output in
  `src-tauri/src/tests/fixtures/`.
- Tests use a mock adapter that returns canned `ListeningSocket` /
  `ProcessInfo` values.

## Adding a platform

1. Create `platform::<name>` with a struct implementing `PlatformAdapter`.
2. Wire selection via `#[cfg(target_os = "...")]` at the `platform` mod
   root.
3. Add fixture-based parser tests.
4. Document any platform quirks here.
