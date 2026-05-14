//! Maps PIDs to full process metadata and infers project roots.
//!
//! On each refresh we ask the adapter for the full process snapshot in one
//! sweep, then filter down to the PIDs we actually care about. Targeted
//! per-PID lookups via sysinfo's `ProcessesToUpdate::Some` were unreliable in
//! practice (cwd/cmd often came back empty on cold reads), so we trade a
//! slightly larger snapshot for consistently populated fields.
//!
//! The cwd → project mapping is cached because project roots don't move with
//! PIDs and re-walking the filesystem every 1.5s would be wasteful.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::models::port::ListeningSocket;
use crate::models::process::ProcessInfo;
use crate::platform::traits::PlatformAdapter;

/// Files whose presence marks the root of a project. Walking stops at the
/// *first* matching ancestor so monorepo sub-packages are preferred over the
/// monorepo root itself.
const PROJECT_MARKERS: &[&str] = &[
    ".git",
    "package.json",
    "pnpm-workspace.yaml",
    "turbo.json",
    "nx.json",
    "Cargo.toml",
    "pyproject.toml",
    "requirements.txt",
    "manage.py",
    "Gemfile",
    "go.mod",
    "composer.json",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
];

const MAX_WALK_DEPTH: usize = 20;

/// Folder/package names that don't carry meaning on their own — they describe
/// a *role*, not the project. When the immediate basename is one of these, we
/// walk up to find a more specific ancestor.
const GENERIC_NAMES: &[&str] = &[
    "src",
    "app",
    "apps",
    "package",
    "packages",
    "lib",
    "libs",
    "test",
    "tests",
    "web",
    "api",
    "frontend",
    "backend",
    "server",
    "client",
    "service",
    "services",
    "bin",
    "cmd",
    "ui",
    "core",
    "shared",
    "common",
    "internal",
    "infra",
    "infrastructure",
    "main",
    "default",
];

fn is_generic_name(name: &str) -> bool {
    GENERIC_NAMES.iter().any(|g| g.eq_ignore_ascii_case(name))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHint {
    pub root_path: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct EnrichedSocket {
    pub socket: ListeningSocket,
    pub process: Option<ProcessInfo>,
    pub project: Option<ProjectHint>,
}

impl EnrichedSocket {
    pub fn bare(socket: ListeningSocket) -> Self {
        Self {
            socket,
            process: None,
            project: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct ProcessEnricher {
    /// `cwd` → resolved project hint (None means walked and found nothing).
    /// Project roots don't move with PIDs, so this cache safely outlives any
    /// individual process.
    project_cache: HashMap<String, Option<ProjectHint>>,
}

impl ProcessEnricher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enrich a batch of sockets in a single OS sweep, with platform-specific
    /// fallbacks (e.g. `ps`/`lsof` on macOS) for fields the cheap path leaves
    /// empty.
    pub fn enrich(
        &mut self,
        adapter: &dyn PlatformAdapter,
        sockets: &[ListeningSocket],
    ) -> Vec<EnrichedSocket> {
        let unique_pids: HashSet<u32> = sockets.iter().filter_map(|s| s.pid).collect();
        if unique_pids.is_empty() {
            return sockets.iter().cloned().map(EnrichedSocket::bare).collect();
        }

        let pid_vec: Vec<u32> = unique_pids.iter().copied().collect();
        let info_by_pid: HashMap<u32, ProcessInfo> = match adapter.get_process_infos(&pid_vec) {
            Ok(map) => map,
            Err(err) => {
                tracing::warn!(error = %err, "process enrichment failed; sockets will lack metadata");
                HashMap::new()
            }
        };

        sockets
            .iter()
            .map(|s| {
                let process = s.pid.and_then(|pid| info_by_pid.get(&pid).cloned());
                let project = process
                    .as_ref()
                    .and_then(|p| p.cwd.as_deref())
                    .and_then(|cwd| self.project_for(cwd));
                EnrichedSocket {
                    socket: s.clone(),
                    process,
                    project,
                }
            })
            .collect()
    }

    fn project_for(&mut self, cwd: &str) -> Option<ProjectHint> {
        if let Some(cached) = self.project_cache.get(cwd) {
            return cached.clone();
        }
        let hint = infer_project_root(Path::new(cwd));
        self.project_cache.insert(cwd.to_string(), hint.clone());
        hint
    }
}

/// Walk up from `start` looking for the closest directory containing any of
/// the well-known project marker files. Stops at the user's home directory or
/// after `MAX_WALK_DEPTH` levels. Once the project root is found, derives a
/// meaningful display name via [`meaningful_project_name`].
pub fn infer_project_root(start: &Path) -> Option<ProjectHint> {
    let home = std::env::var_os("HOME").map(PathBuf::from);

    for ancestor in start.ancestors().take(MAX_WALK_DEPTH) {
        for marker in PROJECT_MARKERS {
            if ancestor.join(marker).exists() {
                let name = meaningful_project_name(ancestor);
                return Some(ProjectHint {
                    root_path: ancestor.to_string_lossy().to_string(),
                    name,
                });
            }
        }
        if home.as_deref() == Some(ancestor) {
            return None;
        }
    }
    None
}

/// Build the user-facing project name. Resolution order:
///
/// 1. `package.json` / `Cargo.toml` manifest `name` field (the project
///    actually identifies itself).
/// 2. The project root's basename, if it's not generic.
/// 3. The closest non-generic ancestor name. If the basename itself was
///    generic (e.g. `web`), the result is combined as `{ancestor}/{basename}`
///    so the user still knows which app inside the parent.
/// 4. Last resort: the basename even if generic.
fn meaningful_project_name(root: &Path) -> String {
    let basename = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string());

    if let Some(name) = read_manifest_name(root) {
        let leaf = strip_npm_scope(&name);
        if !is_generic_name(leaf) {
            return leaf.to_string();
        }
    }

    if !basename.is_empty() && !is_generic_name(&basename) {
        return basename;
    }

    // Basename is generic — walk up looking for a meaningful ancestor.
    let home = std::env::var_os("HOME").map(PathBuf::from);
    for ancestor in root.ancestors().skip(1).take(MAX_WALK_DEPTH) {
        if home.as_deref() == Some(ancestor) {
            break;
        }
        if let Some(parent_name) = ancestor.file_name().and_then(|n| n.to_str()) {
            if !is_generic_name(parent_name) {
                return if basename.is_empty() {
                    parent_name.to_string()
                } else {
                    format!("{parent_name}/{basename}")
                };
            }
        }
    }

    basename
}

/// Strip an npm scope prefix from a package name, e.g. `@acme/web` → `web`.
/// Returns the input unchanged when there's no scope.
fn strip_npm_scope(name: &str) -> &str {
    name.strip_prefix('@')
        .and_then(|rest| rest.split_once('/'))
        .map(|(_, leaf)| leaf)
        .unwrap_or(name)
}

/// Read a JSON `name` field from package.json, falling back to a minimal
/// regex match against Cargo.toml's `[package] name = "…"`. Returns `None`
/// when no manifest exists or the field is missing/empty.
fn read_manifest_name(dir: &Path) -> Option<String> {
    if let Some(name) = read_package_json_name(dir) {
        return Some(name);
    }
    read_cargo_toml_name(dir)
}

fn read_package_json_name(dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(dir.join("package.json")).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let name = value.get("name")?.as_str()?.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn read_cargo_toml_name(dir: &Path) -> Option<String> {
    let content = std::fs::read_to_string(dir.join("Cargo.toml")).ok()?;
    // Match the `name = "..."` line inside a `[package]` section. Avoids
    // pulling in a TOML parser for this single field. The non-greedy capture
    // tolerates other keys between `[package]` and `name`.
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r#"\[package\][\s\S]*?\bname\s*=\s*"([^"]+)""#).unwrap()
    });
    let caps = re.captures(&content)?;
    let name = caps.get(1)?.as_str().trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn tmp_dir(label: &str) -> PathBuf {
        let n = TMP_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "devdock-test-{}-{}-{}",
            std::process::id(),
            label,
            n
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn finds_project_root_at_package_json() {
        let root = tmp_dir("pkg");
        fs::write(root.join("package.json"), "{}").unwrap();
        let sub = root.join("src").join("api");
        fs::create_dir_all(&sub).unwrap();

        let hint = infer_project_root(&sub).expect("should find a project");
        assert_eq!(hint.root_path, root.to_string_lossy());
    }

    #[test]
    fn returns_none_outside_a_project() {
        let root = tmp_dir("empty");
        assert!(infer_project_root(&root).is_none());
    }

    #[test]
    fn prefers_inner_package_in_a_monorepo() {
        let mono = tmp_dir("monorepo");
        fs::write(mono.join("package.json"), "{}").unwrap();
        fs::write(mono.join("pnpm-workspace.yaml"), "packages:\n  - apps/*").unwrap();
        let inner = mono.join("apps").join("web");
        fs::create_dir_all(&inner).unwrap();
        fs::write(inner.join("package.json"), "{}").unwrap();

        let hint = infer_project_root(&inner).expect("should find inner package");
        assert_eq!(hint.root_path, inner.to_string_lossy());
    }

    #[test]
    fn uses_package_json_name_over_basename() {
        let root = tmp_dir("named");
        fs::write(root.join("package.json"), r#"{"name":"acme-app"}"#).unwrap();
        let hint = infer_project_root(&root).unwrap();
        assert_eq!(hint.name, "acme-app");
    }

    #[test]
    fn strips_npm_scope_from_package_name() {
        let root = tmp_dir("scoped");
        fs::write(root.join("package.json"), r#"{"name":"@acme/web-app"}"#).unwrap();
        let hint = infer_project_root(&root).unwrap();
        assert_eq!(hint.name, "web-app");
    }

    #[test]
    fn falls_back_through_generic_names() {
        let outer = tmp_dir("biz-monorepo");
        let middle = outer.join("apps");
        let inner = middle.join("web");
        fs::create_dir_all(&inner).unwrap();
        fs::write(inner.join("package.json"), r#"{"name":"web"}"#).unwrap();

        let hint = infer_project_root(&inner).unwrap();
        let outer_name = outer.file_name().unwrap().to_string_lossy();
        assert_eq!(hint.name, format!("{outer_name}/web"));
    }

    #[test]
    fn reads_cargo_toml_name() {
        let root = tmp_dir("cargo");
        fs::write(
            root.join("Cargo.toml"),
            r#"[package]
name = "my-crate"
version = "0.1.0"
"#,
        )
        .unwrap();
        let hint = infer_project_root(&root).unwrap();
        assert_eq!(hint.name, "my-crate");
    }

    #[test]
    fn project_cache_avoids_repeated_walks() {
        let root = tmp_dir("cache");
        fs::write(root.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        let mut e = ProcessEnricher::new();

        let cwd = root.to_string_lossy().to_string();
        let a = e.project_for(&cwd);
        let b = e.project_for(&cwd);
        assert_eq!(a, b);
        assert!(a.is_some());
        assert_eq!(e.project_cache.len(), 1);
    }
}
