//! Detect installed code editors and launch them.
//!
//! Two signals each editor can be present by:
//!   * a CLI on `$PATH` (e.g. `code`, `cursor`, `subl`) — preferred because it
//!     can reuse an existing editor window
//!   * a macOS application bundle in `/Applications` (or `~/Applications`)
//!
//! Either signal is sufficient. When opening a path, we try the CLI first and
//! fall back to `open -a "<Bundle Name>"`. That keeps the action working
//! whether or not the user has run the editor's "install command on PATH"
//! one-time setup.

use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EditorInfo {
    /// Stable identifier used by the frontend when invoking `open_in_editor`.
    pub id: String,
    pub display_name: String,
    /// Absolute path to the CLI if found on `$PATH`, else `None`.
    pub cli_path: Option<String>,
    /// macOS bundle name (without `.app`) if found in an Applications dir.
    pub bundle_name: Option<String>,
}

#[derive(Debug)]
struct KnownEditor {
    id: &'static str,
    display_name: &'static str,
    cli: &'static str,
    bundle: &'static str,
}

/// Priority order: editors detected earlier in this list show up earlier in
/// the panel menu. VS Code first because it's by far the most common.
const KNOWN_EDITORS: &[KnownEditor] = &[
    KnownEditor {
        id: "vscode",
        display_name: "VS Code",
        cli: "code",
        bundle: "Visual Studio Code",
    },
    KnownEditor {
        id: "cursor",
        display_name: "Cursor",
        cli: "cursor",
        bundle: "Cursor",
    },
    KnownEditor {
        id: "windsurf",
        display_name: "Windsurf",
        cli: "windsurf",
        bundle: "Windsurf",
    },
    KnownEditor {
        id: "zed",
        display_name: "Zed",
        cli: "zed",
        bundle: "Zed",
    },
    KnownEditor {
        id: "sublime",
        display_name: "Sublime Text",
        cli: "subl",
        bundle: "Sublime Text",
    },
    KnownEditor {
        id: "fleet",
        display_name: "Fleet",
        cli: "fleet",
        bundle: "Fleet",
    },
];

pub fn detect_editors() -> Vec<EditorInfo> {
    KNOWN_EDITORS
        .iter()
        .filter_map(|e| {
            let cli_path = which(e.cli);
            let bundle = if is_app_installed(e.bundle) {
                Some(e.bundle.to_string())
            } else {
                None
            };
            if cli_path.is_none() && bundle.is_none() {
                return None;
            }
            Some(EditorInfo {
                id: e.id.to_string(),
                display_name: e.display_name.to_string(),
                cli_path,
                bundle_name: bundle,
            })
        })
        .collect()
}

/// Open `path` in the editor identified by `editor_id`. Returns Err with a
/// human-readable reason when the editor isn't known or can't be launched.
pub fn open_in(editor_id: &str, path: &str) -> Result<(), String> {
    let editor = KNOWN_EDITORS
        .iter()
        .find(|e| e.id == editor_id)
        .ok_or_else(|| format!("unknown editor: {editor_id}"))?;

    // Try the CLI first — when the editor is already running, opening via
    // `code <path>` etc. reuses the existing window. That matches what a user
    // expects when they ask to "open in <editor>".
    if which(editor.cli).is_some()
        && Command::new(editor.cli).arg(path).spawn().is_ok()
    {
        return Ok(());
    }

    // Fall back to `open -a` on macOS for users who haven't installed the
    // CLI to PATH yet (a common state after a fresh editor install).
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", editor.bundle])
            .arg(path)
            .spawn()
            .map_err(|e| format!("failed to launch {}: {e}", editor.display_name))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(format!(
            "{} CLI not on PATH and bundle launch isn't supported on this OS",
            editor.display_name
        ))
    }
}

fn which(cmd: &str) -> Option<String> {
    let output = Command::new("which").arg(cmd).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn is_app_installed(bundle_name: &str) -> bool {
    let home = std::env::var("HOME").ok();
    let candidates: Vec<String> = [
        Some(format!("/Applications/{bundle_name}.app")),
        Some(format!("/System/Applications/{bundle_name}.app")),
        home.as_deref()
            .map(|h| format!("{h}/Applications/{bundle_name}.app")),
    ]
    .into_iter()
    .flatten()
    .collect();
    candidates.iter().any(|p| Path::new(p).exists())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_editor_returns_err() {
        let err = open_in("does-not-exist", "/tmp").unwrap_err();
        assert!(err.contains("unknown editor"));
    }

    #[test]
    fn detect_returns_at_most_one_per_known_editor() {
        // Whatever the CI environment looks like, we should never report
        // duplicates: each known editor appears 0 or 1 times.
        let detected = detect_editors();
        let mut ids: Vec<&String> = detected.iter().map(|e| &e.id).collect();
        ids.sort();
        let len_before = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), len_before, "duplicate editor ids: {ids:?}");
    }
}
