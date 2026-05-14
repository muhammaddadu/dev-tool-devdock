use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::core::managed_runtime::ManagedRuntime;
use crate::core::process_enricher::ProcessEnricher;
use crate::core::service_engine::ClassifierContext;
use crate::platform::{self, traits::PlatformAdapter};

/// Shared application state held by the Tauri app and injected into commands.
pub struct AppState {
    pub platform: Arc<dyn PlatformAdapter>,
    /// Held behind a `Mutex` because the enricher mutates caches. Commands
    /// lock briefly during the synchronous enrichment phase and release
    /// before awaiting anything else.
    pub enricher: Mutex<ProcessEnricher>,
    pub classifier: ClassifierContext,
    /// Tracks services DevDock has launched. Cloneable (internal `Arc`) so
    /// background wait-tasks can update it when a managed child exits.
    pub runtime: ManagedRuntime,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            platform: platform::default_adapter(),
            enricher: Mutex::new(ProcessEnricher::new()),
            classifier: ClassifierContext::new(resolve_dev_root()),
            runtime: ManagedRuntime::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve the directory that should anchor "this is DevDock itself" checks.
///
/// Only meaningful during `pnpm tauri dev` — the Vite dev server and our
/// webview process live under the repo root and we want to keep them out of
/// the foreground Dev bucket. In a packaged release build the binary lives
/// inside an `.app` bundle and there is no "dev tree" to anchor against, so
/// we return `None` and the check is skipped entirely. Without this guard a
/// packaged build with `current_dir() = "/"` would match every process and
/// route everything to Tooling.
fn resolve_dev_root() -> Option<PathBuf> {
    if is_running_from_app_bundle() {
        tracing::info!("running from .app bundle; self-detection disabled");
        return None;
    }

    let cwd = std::env::current_dir().ok()?;
    let resolved = if cwd.file_name().and_then(|n| n.to_str()) == Some("src-tauri") {
        cwd.parent().map(PathBuf::from).unwrap_or(cwd)
    } else {
        cwd
    };
    tracing::info!(dev_root = %resolved.display(), "resolved DevDock dev root");
    Some(resolved)
}

fn is_running_from_app_bundle() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    exe.to_string_lossy().contains(".app/Contents/")
}
