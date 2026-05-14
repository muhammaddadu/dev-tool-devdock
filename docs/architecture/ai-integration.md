# AI Integration

AI in DevDock is **manual, optional, and local-first**. There is no
cloud account, no API key collection, and no background invocation. If
no local provider is installed, the AI surface is hidden entirely.

## Providers

Detected by presence of a CLI binary on `PATH`:

| Provider | Binary       | Notes                                  |
| -------- | ------------ | -------------------------------------- |
| Claude   | `claude`     | Anthropic Claude CLI                   |
| Codex    | `codex`      | OpenAI Codex CLI                       |
| Cursor   | `cursor`     | Cursor agent CLI                       |
| Ollama   | `ollama`     | Local model runner                     |

Detection runs once at startup and on user request (`detect_ai_providers`).
DevDock does not install, configure, or update any of these tools.

## Provider interface

Internally each provider implements a small Rust trait:

```rust
pub trait AiProvider {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn analyze(&self, bundle: &MetadataBundle) -> AnalysisResult;
}
```

`MetadataBundle` is constructed by the core (not by the provider) and
contains only allow-listed files.

## Allow-list

Only the following file types may be included in a metadata bundle:

- `package.json`, `pnpm-workspace.yaml`, `lerna.json`
- `Cargo.toml`, `pyproject.toml`, `requirements.txt`, `Pipfile`
- `go.mod`, `composer.json`, `Gemfile`
- `Procfile`, `Makefile`, `justfile`
- `docker-compose.yml`, `docker-compose.yaml`
- `README.md` (first N kilobytes only)
- `.tool-versions`, `.nvmrc`

No source code. No `.env`. No `node_modules`. No secrets, ever.

## Rules

- **Manual.** AI runs only on explicit user action (a button in the
  scan dialog).
- **Review-only.** Output is presented as a list of suggested services
  the user can edit, save, or discard. DevDock never spawns a process
  from AI output without the user clicking Run on a saved service.
- **No background.** AI is never invoked by the watcher, the scheduler,
  or any startup hook.
- **No silent file reads.** The bundle is shown to the user before
  sending, and the user can deselect files.
- **Caching.** Results are cached in `ai_suggestions` keyed by
  `input_hash` to avoid repeat invocations on identical bundles.
