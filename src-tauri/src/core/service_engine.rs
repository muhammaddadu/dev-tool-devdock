//! Service engine.
//!
//! Pure mapping layer: takes an `EnrichedSocket` (raw OS listener + optional
//! process + project metadata) and produces a `ServiceView` for the frontend.
//! No I/O — the adapter does OS calls, the enricher walks the filesystem,
//! and this module just decides how it should appear.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::core::managed_runtime::ManagedProcessRecord;
use crate::core::process_enricher::EnrichedSocket;
use crate::models::saved_service::SavedService;
use crate::models::service::{ServiceBucket, ServiceSource, ServiceStatus, ServiceView};

/// Contextual data the classifier needs but the rest of the engine doesn't
/// carry around. Currently just the directory DevDock is running from, used to
/// recognize "this is DevDock itself" and avoid showing the dev server as a
/// foreground service.
#[derive(Debug, Clone, Default)]
pub struct ClassifierContext {
    pub dev_root: Option<PathBuf>,
}

impl ClassifierContext {
    pub fn new(dev_root: Option<PathBuf>) -> Self {
        Self { dev_root }
    }
}

/// IDE helpers, language servers, debug bridges, and other dev infrastructure
/// that listens on local ports but isn't what the user is actively building.
///
/// Names are matched as **prefixes** because lsof truncates COMMAND to 9 chars.
/// Entries are written as the lsof-displayed form so they work against both
/// truncated and full process names.
const TOOLING_NAME_PREFIXES: &[&str] = &[
    // Editors and their helper processes
    "Code Help",     // VS Code Helper, VS Code Helper (Plugin/Renderer/GPU)
    "Code",          // VS Code main
    "Cursor He",     // Cursor Helper
    "Cursor",
    "Windsurf",
    "Sublime T",     // Sublime Text
    "subl",
    // JetBrains IDEs (truncated forms)
    "idea",
    "WebStorm",
    "PyCharm",
    "IntelliJ",
    "GoLand",
    "RustRover",
    "RubyMine",
    "DataGrip",
    "PhpStorm",
    "AppCode",
    "CLion",
    // Language servers / debug bridges
    "adb",
    "gopls",
    "rust-anal",     // rust-analyzer
    "tsserver",
    "typescript",    // typescript-language-server
    "pyright",
    "clangd",
    "lua-langu",     // lua-language-server
    "metals",        // Scala
    "solargrap",     // solargraph (Ruby)
    // Container & dev environment UIs
    "Docker De",     // Docker Desktop
    "com.docke",     // com.docker.backend
    "OrbStack",
    "Rancher D",     // Rancher Desktop
    "colima",
    // Misc local dev infra
    "ngrok",
    "Tailscale",
];

/// Process-name prefixes that should land in the `System` bucket (collapsed by
/// default). Names are matched as **prefixes** because macOS's `lsof` truncates
/// the COMMAND column (typically to 9 chars), so the stored values must work
/// against both the truncated form ("ControlCe") and the full form
/// ("ControlCenter") when other listers don't truncate.
///
/// Keep this list conservative — when in doubt, default to `Dev` and let the
/// user hide individual entries (M3 wires `ignored_ports`).
const SYSTEM_NAME_PREFIXES: &[&str] = &[
    // Apple system daemons
    "rapportd",
    "ControlCe",     // ControlCenter (truncated)
    "mDNSResp",      // mDNSResponder
    "sharingd",
    "nsurlses",      // nsurlsessiond
    "cloudd",
    "identitys",     // identityservicesd
    "AirPlay",
    "remoted",
    "trustd",
    "apsd",
    "callserv",      // callservicesd
    "secd",
    "syncdefa",      // syncdefaultsd
    "softwareupdate",
    "screensharingd",
    "homed",
    "WiFiAgent",
    "bluetoothd",
    "wirelessproxd",
    // Common consumer apps that listen on local ports but aren't dev work
    "Dropbox",
    "Backblaze",
    "Spotify",
    "Slack",
    "Discord",
    "Google",        // Google Drive / Google Software Update
    "1Password",
    "Loom",
    "Zoom",
    "zoom.us",
    "Postman",
    "Notion",
];

/// Classify a service. Resolution order, most specific first:
///
/// 1. Anything running from DevDock's own dev root is `Tooling` (so DevDock
///    doesn't appear as a foreground service when run via `pnpm tauri dev`).
/// 2. System daemons by process-name prefix.
/// 3. Tooling/IDE/dev-infra by process-name prefix.
/// 4. Everything else (and unknown names) defaults to `Dev`.
pub fn classify(
    process_name: Option<&str>,
    cwd: Option<&str>,
    project_root: Option<&str>,
    ctx: &ClassifierContext,
) -> ServiceBucket {
    if is_devdock_self(cwd, project_root, ctx) {
        return ServiceBucket::Tooling;
    }
    if is_app_bundle_cwd(cwd) {
        // DBeaver, Spotify, Slack helpers, etc. — anything running with cwd
        // inside its own .app bundle is a desktop app's internal process,
        // not the user's foreground work. The user can still Save it (which
        // promotes to Dev) if they want to manage it through DevDock.
        return ServiceBucket::Tooling;
    }

    let Some(name) = process_name else {
        return ServiceBucket::Dev;
    };

    if SYSTEM_NAME_PREFIXES.iter().any(|prefix| name.starts_with(prefix)) {
        return ServiceBucket::System;
    }
    if TOOLING_NAME_PREFIXES.iter().any(|prefix| name.starts_with(prefix)) {
        return ServiceBucket::Tooling;
    }
    ServiceBucket::Dev
}

/// True when the candidate process is part of DevDock's own dev tree.
///
/// Checked two ways for resilience — `cwd` reads can fail on macOS for some
/// processes due to libproc permissions, but the enricher may still have
/// inferred a project root from a sibling signal. Either match is sufficient.
/// True when the process is running with cwd inside any `.app` bundle. Catches
/// DBeaver, Slack, Spotify, etc. without us needing to maintain an
/// ever-growing process-name list.
fn is_app_bundle_cwd(cwd: Option<&str>) -> bool {
    let Some(c) = cwd else {
        return false;
    };
    c.contains(".app/Contents/")
}

fn is_devdock_self(
    cwd: Option<&str>,
    project_root: Option<&str>,
    ctx: &ClassifierContext,
) -> bool {
    let Some(dev_root) = ctx.dev_root.as_deref() else {
        return false;
    };
    if let Some(p) = project_root {
        if Path::new(p) == dev_root {
            return true;
        }
    }
    if let Some(c) = cwd {
        if Path::new(c).starts_with(dev_root) {
            return true;
        }
    }
    false
}

fn display_host(host: &str) -> String {
    match host {
        "0.0.0.0" | "*" | "::" | "" => "127.0.0.1".to_string(),
        other => other.to_string(),
    }
}

fn url_for(host: &str, port: u16) -> String {
    let h = display_host(host);
    if h.contains(':') {
        format!("http://[{h}]:{port}")
    } else {
        format!("http://{h}:{port}")
    }
}

/// Stable ID for a detected service. Includes `started_at` when available so
/// that a PID-reused process doesn't inherit the previous service's identity.
fn detected_id(port: u16, pid: Option<u32>, started_at: Option<&str>) -> String {
    match (pid, started_at) {
        (Some(pid), Some(start)) => format!("detected:{port}:{pid}:{start}"),
        (Some(pid), None) => format!("detected:{port}:{pid}"),
        (None, _) => format!("detected:{port}"),
    }
}

/// Build the user-facing label. Prefers the project name when known, then the
/// full process name (from enrichment), then the truncated lsof name, then a
/// fallback to the port number.
fn label_for(enriched: &EnrichedSocket) -> String {
    if let Some(project) = &enriched.project {
        if !project.name.is_empty() {
            return project.name.clone();
        }
    }
    if let Some(process) = &enriched.process {
        if !process.process_name.is_empty() {
            return process.process_name.clone();
        }
    }
    if let Some(name) = enriched.socket.process_name.as_deref().filter(|s| !s.is_empty()) {
        return name.to_string();
    }
    format!("Port {}", enriched.socket.port)
}

/// Build a compact, human-readable command string.
///
/// Node's `argv` is full of nvm paths and node_modules/.bin lookups that bury
/// the actual script. We:
///
/// * collapse absolute paths to their basename (so `/Users/.../node` → `node`,
///   `/.../node_modules/vite/bin/vite.js` → `vite`)
/// * keep flags and non-path args verbatim
/// * drop `node` when it's followed by an interpreted script (the script is
///   the meaningful command anyway)
fn display_command(cmd: &[String]) -> Option<String> {
    if cmd.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = cmd.iter().map(|arg| compact_arg(arg)).collect();

    // `node /path/to/foo.js` → `foo.js`. The interpreter prefix is noise.
    if parts.len() >= 2 && parts[0] == "node" && !parts[1].starts_with('-') {
        parts.remove(0);
    }
    // Same for python invocations: `python script.py` → `script.py`.
    if parts.len() >= 2
        && (parts[0] == "python" || parts[0] == "python3")
        && !parts[1].starts_with('-')
    {
        parts.remove(0);
    }

    Some(parts.join(" "))
}

/// Reduce a CLI argument to a compact display form: replace absolute paths
/// with their leaf basename. Short args (under 4 chars), things that don't
/// look like paths, and flags are returned unchanged.
fn compact_arg(arg: &str) -> String {
    if arg.starts_with('-') || arg.len() < 4 {
        return arg.to_string();
    }
    if !arg.contains('/') {
        return arg.to_string();
    }
    // Treat tokens like `KEY=/foo/bar` as `KEY=bar`.
    if let Some((key, value)) = arg.split_once('=') {
        if value.contains('/') {
            let base = std::path::Path::new(value)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| value.to_string());
            return format!("{key}={base}");
        }
    }
    std::path::Path::new(arg)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| arg.to_string())
}

pub fn service_view(enriched: &EnrichedSocket, ctx: &ClassifierContext) -> ServiceView {
    let port = enriched.socket.port;
    let host = enriched.socket.host.clone();
    let process_name = enriched
        .process
        .as_ref()
        .map(|p| p.process_name.clone())
        .or_else(|| enriched.socket.process_name.clone());

    let command = enriched
        .process
        .as_ref()
        .and_then(|p| p.command_line.as_deref())
        .and_then(display_command);
    let cwd = enriched.process.as_ref().and_then(|p| p.cwd.clone());

    let started_at = enriched
        .process
        .as_ref()
        .and_then(|p| p.started_at.as_deref());
    let started_at_unix = started_at.and_then(|s| s.parse::<u64>().ok());

    ServiceView {
        id: detected_id(port, enriched.socket.pid, started_at),
        label: label_for(enriched),
        status: ServiceStatus::Running,
        source: ServiceSource::Detected,
        bucket: classify(
            process_name.as_deref(),
            cwd.as_deref(),
            enriched.project.as_ref().map(|p| p.root_path.as_str()),
            ctx,
        ),
        port: Some(port),
        host: Some(display_host(&host)),
        url: Some(url_for(&host, port)),
        pid: enriched.socket.pid,
        process_name,
        command,
        cwd,
        project_name: enriched.project.as_ref().map(|p| p.name.clone()),
        project_root: enriched.project.as_ref().map(|p| p.root_path.clone()),
        started_at_unix,
        saved_id: None,
        log_path: None,
        pinned: false,
        can_open: true,
        // A bare detected service is eligible to save. The merge step turns
        // this off when a matching saved service already exists.
        can_save: true,
        can_run: false,
        can_stop: false,
        can_restart: false,
        can_kill: false,
    }
}

pub fn service_views(enriched: &[EnrichedSocket], ctx: &ClassifierContext) -> Vec<ServiceView> {
    enriched.iter().map(|e| service_view(e, ctx)).collect()
}

/// Merge currently-detected service views with the user's saved services.
///
/// For each detected service whose port appears in some saved service's
/// `expected_ports`, the saved row "claims" it: the user-chosen label takes
/// over, the service is promoted to the Dev bucket (the user explicitly
/// cares about it), and Save is no longer offered. Saved services that
/// aren't currently bound to a port are emitted as separate Stopped views
/// so the user can see them in the Pinned / Recently Seen sections.
///
/// Returns the merged list. Order isn't guaranteed — the frontend's section
/// filter (Running / Pinned / Recently Seen) handles presentation.
pub fn merge_with_saved(
    detected: Vec<ServiceView>,
    saved: &[SavedService],
) -> Vec<ServiceView> {
    merge(detected, saved, &HashMap::new())
}

/// Three-way merge of detected sockets, saved-services rows, and the live
/// managed process registry. The result is what the panel renders.
///
/// Cases handled, by saved-service:
///
/// * Managed by us **and** detected on a port — Running, can Stop/Restart.
/// * Managed by us but no port bound yet — Starting, can Stop. (Spawned
///   process is still warming up; the port will appear on a later refresh.)
/// * Detected on a port but not managed by us — Running externally. Adopt
///   the saved label but don't pretend we can Stop it; the user would need
///   to Kill instead.
/// * Saved but neither managed nor detected — Stopped, can Run.
pub fn merge(
    mut detected: Vec<ServiceView>,
    saved: &[SavedService],
    managed: &HashMap<String, ManagedProcessRecord>,
) -> Vec<ServiceView> {
    let mut claimed: HashSet<String> = HashSet::new();

    for view in &mut detected {
        let Some(port) = view.port else { continue };
        let Some(saved_entry) = saved
            .iter()
            .find(|s| s.expected_ports.contains(&port))
        else {
            continue;
        };
        let managed_entry = managed.get(&saved_entry.id);
        let is_managed = managed_entry.is_some();

        view.label = saved_entry.label.clone();
        view.saved_id = Some(saved_entry.id.clone());
        view.pinned = saved_entry.pinned;
        view.source = ServiceSource::Managed;
        view.bucket = ServiceBucket::Dev;
        view.can_save = false;
        view.can_stop = is_managed;
        view.can_restart = is_managed;
        view.can_run = false;
        // External (not managed by us) saved instance can still be killed —
        // that's how the user reclaims a stuck port to relaunch via DevDock.
        view.can_kill = !is_managed && view.pid.is_some();
        view.log_path = managed_entry.map(|m| m.log_path.clone());
        claimed.insert(saved_entry.id.clone());
    }

    // For any remaining detected services (no saved match) — these are pure
    // external processes. Allow Kill so the user can free a port.
    for view in &mut detected {
        if view.saved_id.is_some() {
            continue; // already handled above
        }
        view.can_kill = view.pid.is_some();
    }

    for s in saved {
        if claimed.contains(&s.id) {
            continue;
        }
        let managed_entry = managed.get(&s.id);
        detected.push(view_for_saved(s, managed_entry));
    }

    detected
}

fn view_for_saved(
    s: &SavedService,
    managed: Option<&ManagedProcessRecord>,
) -> ServiceView {
    let port = s.expected_ports.first().copied();
    let is_starting = managed.is_some();
    ServiceView {
        id: format!("saved:{}", s.id),
        label: s.label.clone(),
        status: if is_starting {
            ServiceStatus::Starting
        } else {
            ServiceStatus::Stopped
        },
        source: ServiceSource::Managed,
        bucket: ServiceBucket::Dev,
        port,
        host: None,
        url: port.map(|p| format!("http://127.0.0.1:{p}")),
        pid: managed.map(|m| m.root_pid),
        process_name: None,
        command: Some(s.command.clone()),
        cwd: Some(s.cwd.clone()),
        project_name: None,
        project_root: None,
        started_at_unix: managed.map(|m| m.started_at_unix),
        saved_id: Some(s.id.clone()),
        log_path: managed.map(|m| m.log_path.clone()),
        pinned: s.pinned,
        can_open: false,
        can_save: false,
        can_run: !is_starting,
        can_stop: is_starting,
        can_restart: false,
        can_kill: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::process_enricher::ProjectHint;
    use crate::models::port::{ListeningSocket, Protocol};
    use crate::models::process::ProcessInfo;

    fn sock(port: u16, pid: Option<u32>, host: &str, name: Option<&str>) -> ListeningSocket {
        ListeningSocket {
            protocol: Protocol::Tcp,
            host: host.to_string(),
            port,
            pid,
            process_name: name.map(|s| s.to_string()),
        }
    }

    fn process(name: &str, command: &[&str], cwd: Option<&str>, started_at: &str) -> ProcessInfo {
        ProcessInfo {
            pid: 123,
            parent_pid: Some(1),
            process_name: name.to_string(),
            executable_path: None,
            command_line: Some(command.iter().map(|s| s.to_string()).collect()),
            cwd: cwd.map(|s| s.to_string()),
            started_at: Some(started_at.to_string()),
            user: None,
        }
    }

    fn project(name: &str, root: &str) -> ProjectHint {
        ProjectHint {
            root_path: root.to_string(),
            name: name.to_string(),
        }
    }

    fn ctx() -> ClassifierContext {
        ClassifierContext::default()
    }

    #[test]
    fn bare_socket_maps_to_minimal_view() {
        let v = service_view(
            &EnrichedSocket::bare(sock(3000, Some(1), "0.0.0.0", Some("node"))),
            &ctx(),
        );
        assert_eq!(v.label, "node");
        assert!(v.command.is_none());
        assert!(v.cwd.is_none());
        assert!(v.project_name.is_none());
        assert_eq!(v.url.as_deref(), Some("http://127.0.0.1:3000"));
    }

    #[test]
    fn enriched_socket_carries_command_and_cwd() {
        let e = EnrichedSocket {
            socket: sock(3000, Some(1), "0.0.0.0", Some("node")),
            process: Some(process(
                "node",
                &["node", "./node_modules/vite/bin/vite.js"],
                Some("/Users/me/code/web"),
                "1700000000",
            )),
            project: Some(project("web", "/Users/me/code/web")),
        };
        let v = service_view(&e, &ctx());
        assert_eq!(v.label, "web");
        // display_command drops the `node` interpreter and reduces the script
        // path to its basename.
        assert_eq!(v.command.as_deref(), Some("vite.js"));
        assert_eq!(v.cwd.as_deref(), Some("/Users/me/code/web"));
        assert_eq!(v.project_name.as_deref(), Some("web"));
        assert_eq!(v.project_root.as_deref(), Some("/Users/me/code/web"));
    }

    #[test]
    fn label_falls_back_through_enrichment_layers() {
        let bare = EnrichedSocket::bare(sock(8000, Some(1), "127.0.0.1", None));
        assert_eq!(service_view(&bare, &ctx()).label, "Port 8000");

        let lsof_only = EnrichedSocket::bare(sock(8000, Some(1), "127.0.0.1", Some("python")));
        assert_eq!(service_view(&lsof_only, &ctx()).label, "python");

        let with_process = EnrichedSocket {
            socket: sock(8000, Some(1), "127.0.0.1", Some("python")),
            process: Some(process("Python", &["python3"], None, "1700000000")),
            project: None,
        };
        assert_eq!(service_view(&with_process, &ctx()).label, "Python");

        let with_project = EnrichedSocket {
            socket: sock(8000, Some(1), "127.0.0.1", Some("python")),
            process: Some(process("Python", &["python3"], None, "1700000000")),
            project: Some(project("api", "/Users/me/code/api")),
        };
        assert_eq!(service_view(&with_project, &ctx()).label, "api");
    }

    #[test]
    fn id_includes_started_at_for_pid_reuse_safety() {
        let a = service_view(
            &EnrichedSocket {
                socket: sock(3000, Some(1), "0.0.0.0", None),
                process: Some(process("node", &["node"], None, "1700000000")),
                project: None,
            },
            &ctx(),
        );
        let b = service_view(
            &EnrichedSocket {
                socket: sock(3000, Some(1), "0.0.0.0", None),
                process: Some(process("node", &["node"], None, "1700009999")),
                project: None,
            },
            &ctx(),
        );
        assert_ne!(a.id, b.id, "different start time should yield different IDs");
    }

    #[test]
    fn classifies_apple_daemons_as_system() {
        let c = ctx();
        assert_eq!(classify(Some("rapportd"), None, None, &c), ServiceBucket::System);
        assert_eq!(classify(Some("ControlCe"), None, None, &c), ServiceBucket::System);
        assert_eq!(classify(Some("ControlCenter"), None, None, &c), ServiceBucket::System);
        assert_eq!(classify(Some("mDNSResponder"), None, None, &c), ServiceBucket::System);
        assert_eq!(classify(Some("sharingd"), None, None, &c), ServiceBucket::System);
    }

    #[test]
    fn classifies_consumer_apps_as_system() {
        let c = ctx();
        assert_eq!(classify(Some("Dropbox"), None, None, &c), ServiceBucket::System);
        assert_eq!(classify(Some("Slack Helper"), None, None, &c), ServiceBucket::System);
        assert_eq!(classify(Some("Loom"), None, None, &c), ServiceBucket::System);
        assert_eq!(classify(Some("zoom.us"), None, None, &c), ServiceBucket::System);
    }

    #[test]
    fn classifies_ide_helpers_as_tooling() {
        let c = ctx();
        assert_eq!(classify(Some("Code Helper"), None, None, &c), ServiceBucket::Tooling);
        assert_eq!(
            classify(Some("Code Helper (Plugin)"), None, None, &c),
            ServiceBucket::Tooling
        );
        assert_eq!(classify(Some("Cursor Helper"), None, None, &c), ServiceBucket::Tooling);
        assert_eq!(classify(Some("WebStorm"), None, None, &c), ServiceBucket::Tooling);
        assert_eq!(classify(Some("idea"), None, None, &c), ServiceBucket::Tooling);
    }

    #[test]
    fn classifies_dev_infra_as_tooling() {
        let c = ctx();
        assert_eq!(classify(Some("adb"), None, None, &c), ServiceBucket::Tooling);
        assert_eq!(classify(Some("gopls"), None, None, &c), ServiceBucket::Tooling);
        assert_eq!(classify(Some("tsserver"), None, None, &c), ServiceBucket::Tooling);
        assert_eq!(classify(Some("rust-analyzer"), None, None, &c), ServiceBucket::Tooling);
        assert_eq!(classify(Some("Docker Desktop"), None, None, &c), ServiceBucket::Tooling);
        assert_eq!(classify(Some("OrbStack"), None, None, &c), ServiceBucket::Tooling);
    }

    #[test]
    fn app_bundle_processes_classify_as_tooling() {
        let c = ctx();
        assert_eq!(
            classify(
                Some("dbeaver"),
                Some("/Applications/DBeaver.app/Contents/MacOS"),
                None,
                &c,
            ),
            ServiceBucket::Tooling
        );
        assert_eq!(
            classify(
                Some("Spotify"),
                Some("/Applications/Spotify.app/Contents/MacOS"),
                None,
                &c,
            ),
            ServiceBucket::Tooling
        );
        // Same process name but cwd outside an app bundle → Dev.
        assert_eq!(
            classify(Some("dbeaver"), Some("/Users/me/code"), None, &c),
            ServiceBucket::Dev
        );
    }

    #[test]
    fn devdock_self_is_tooling_regardless_of_process_name() {
        let c = ClassifierContext::new(Some(PathBuf::from("/Users/me/devdock")));
        assert_eq!(
            classify(Some("node"), Some("/Users/me/devdock"), None, &c),
            ServiceBucket::Tooling
        );
        assert_eq!(
            classify(Some("node"), Some("/Users/me/devdock/src"), None, &c),
            ServiceBucket::Tooling
        );
        // Same process name but different cwd → Dev
        assert_eq!(
            classify(Some("node"), Some("/Users/me/elsewhere"), None, &c),
            ServiceBucket::Dev
        );
    }

    #[test]
    fn devdock_self_detected_via_project_root_when_cwd_missing() {
        // sysinfo sometimes can't read cwd. The enriched project_root still
        // catches it because we walked the FS once and cached the result.
        let c = ClassifierContext::new(Some(PathBuf::from("/Users/me/devdock")));
        assert_eq!(
            classify(Some("node"), None, Some("/Users/me/devdock"), &c),
            ServiceBucket::Tooling
        );
        // A node service in a different project — Dev.
        assert_eq!(
            classify(Some("node"), None, Some("/Users/me/other-project"), &c),
            ServiceBucket::Dev
        );
    }

    #[test]
    fn classifies_dev_servers_as_dev() {
        let c = ctx();
        assert_eq!(classify(Some("node"), None, None, &c), ServiceBucket::Dev);
        assert_eq!(classify(Some("python3.12"), None, None, &c), ServiceBucket::Dev);
        assert_eq!(classify(Some("ruby"), None, None, &c), ServiceBucket::Dev);
        assert_eq!(classify(Some("postgres"), None, None, &c), ServiceBucket::Dev);
        assert_eq!(classify(Some("redis-server"), None, None, &c), ServiceBucket::Dev);
        assert_eq!(classify(Some("cargo"), None, None, &c), ServiceBucket::Dev);
    }

    #[test]
    fn display_command_collapses_node_path_and_script_path() {
        let cmd = vec![
            "/Users/me/.nvm/versions/node/v20/bin/node".to_string(),
            "/Users/me/project/node_modules/.bin/vite".to_string(),
            "--port".to_string(),
            "3000".to_string(),
        ];
        assert_eq!(
            super::display_command(&cmd).as_deref(),
            Some("vite --port 3000"),
        );
    }

    #[test]
    fn display_command_drops_python_interpreter_prefix() {
        let cmd = vec![
            "python3".to_string(),
            "/Users/me/code/api/manage.py".to_string(),
            "runserver".to_string(),
        ];
        assert_eq!(
            super::display_command(&cmd).as_deref(),
            Some("manage.py runserver"),
        );
    }

    #[test]
    fn display_command_keeps_flags_verbatim() {
        let cmd = vec!["cargo".to_string(), "--release".to_string(), "-v".to_string()];
        assert_eq!(
            super::display_command(&cmd).as_deref(),
            Some("cargo --release -v"),
        );
    }

    fn saved(label: &str, ports: &[u16], pinned: bool) -> SavedService {
        SavedService {
            id: format!("svc-{label}"),
            project_id: None,
            label: label.to_string(),
            command: "npm run dev".into(),
            cwd: "/Users/me/code/web".into(),
            expected_ports: ports.to_vec(),
            pinned,
            created_from: "detected".into(),
            last_run_at: None,
            last_seen_at: None,
            created_at: "2026-05-13T00:00:00Z".into(),
            updated_at: "2026-05-13T00:00:00Z".into(),
        }
    }

    #[test]
    fn merge_claims_running_service_with_matching_saved_port() {
        let detected = vec![service_view(
            &EnrichedSocket::bare(sock(3000, Some(1), "0.0.0.0", Some("node"))),
            &ctx(),
        )];
        let merged = super::merge_with_saved(detected, &[saved("Frontend", &[3000], false)]);

        assert_eq!(merged.len(), 1, "no separate stopped row for a claimed saved");
        let v = &merged[0];
        assert_eq!(v.label, "Frontend");
        assert_eq!(v.saved_id.as_deref(), Some("svc-Frontend"));
        assert!(!v.can_save, "already-saved services shouldn't show Save");
        assert!(matches!(v.source, ServiceSource::Managed));
        assert!(matches!(v.bucket, ServiceBucket::Dev));
    }

    #[test]
    fn merge_emits_stopped_view_for_unmatched_saved() {
        let detected: Vec<ServiceView> = Vec::new();
        let merged = super::merge_with_saved(detected, &[saved("API", &[8000], true)]);

        assert_eq!(merged.len(), 1);
        let v = &merged[0];
        assert!(matches!(v.status, ServiceStatus::Stopped));
        assert!(v.pinned);
        assert_eq!(v.port, Some(8000));
        assert_eq!(v.url.as_deref(), Some("http://127.0.0.1:8000"));
    }

    #[test]
    fn detected_service_without_match_keeps_can_save_true() {
        let detected = vec![service_view(
            &EnrichedSocket::bare(sock(3000, Some(1), "0.0.0.0", Some("node"))),
            &ctx(),
        )];
        let merged = super::merge_with_saved(detected, &[]);
        assert!(merged[0].can_save);
        assert!(merged[0].saved_id.is_none());
    }

    fn managed_record(service_id: &str) -> ManagedProcessRecord {
        ManagedProcessRecord {
            run_id: "run-1".into(),
            service_id: service_id.into(),
            root_pid: 9001,
            started_at_unix: 1_700_000_000,
            log_path: "/tmp/devdock.log".into(),
        }
    }

    #[test]
    fn managed_running_service_gets_stop_and_restart() {
        let detected = vec![service_view(
            &EnrichedSocket::bare(sock(3000, Some(1), "0.0.0.0", Some("node"))),
            &ctx(),
        )];
        let saved_svc = saved("Frontend", &[3000], false);
        let mut managed = HashMap::new();
        managed.insert(saved_svc.id.clone(), managed_record(&saved_svc.id));

        let merged = super::merge(detected, &[saved_svc], &managed);
        let v = &merged[0];
        assert!(v.can_stop);
        assert!(v.can_restart);
        assert!(!v.can_run);
    }

    #[test]
    fn externally_running_saved_service_cannot_be_stopped_by_us() {
        // Saved service is bound to its port but DevDock didn't launch it.
        let detected = vec![service_view(
            &EnrichedSocket::bare(sock(3000, Some(1), "0.0.0.0", Some("node"))),
            &ctx(),
        )];
        let saved_svc = saved("Frontend", &[3000], false);
        let managed: HashMap<String, ManagedProcessRecord> = HashMap::new();

        let merged = super::merge(detected, &[saved_svc], &managed);
        let v = &merged[0];
        assert!(!v.can_stop);
        assert!(!v.can_restart);
    }

    #[test]
    fn starting_service_has_starting_status_and_can_stop() {
        // Saved + managed but no port bound yet.
        let saved_svc = saved("API", &[8000], false);
        let mut managed = HashMap::new();
        managed.insert(saved_svc.id.clone(), managed_record(&saved_svc.id));

        let merged = super::merge(Vec::new(), &[saved_svc], &managed);
        assert_eq!(merged.len(), 1);
        let v = &merged[0];
        assert!(matches!(v.status, ServiceStatus::Starting));
        assert!(v.can_stop);
        assert!(!v.can_run);
        assert_eq!(v.pid, Some(9001));
    }

    #[test]
    fn external_detected_service_can_be_killed() {
        let detected = vec![service_view(
            &EnrichedSocket::bare(sock(8000, Some(42), "127.0.0.1", Some("python"))),
            &ctx(),
        )];
        let merged = super::merge(detected, &[], &HashMap::new());
        assert!(merged[0].can_kill);
        assert_eq!(merged[0].pid, Some(42));
    }

    #[test]
    fn managed_service_does_not_offer_kill() {
        let detected = vec![service_view(
            &EnrichedSocket::bare(sock(3000, Some(1), "0.0.0.0", Some("node"))),
            &ctx(),
        )];
        let saved_svc = saved("Frontend", &[3000], false);
        let mut managed = HashMap::new();
        managed.insert(saved_svc.id.clone(), managed_record(&saved_svc.id));

        let merged = super::merge(detected, &[saved_svc], &managed);
        assert!(!merged[0].can_kill, "managed services use Stop, not Kill");
    }

    #[test]
    fn stopped_saved_service_can_be_run() {
        let saved_svc = saved("API", &[8000], false);
        let merged = super::merge(Vec::new(), &[saved_svc], &HashMap::new());
        let v = &merged[0];
        assert!(matches!(v.status, ServiceStatus::Stopped));
        assert!(v.can_run);
        assert!(!v.can_stop);
    }

    #[test]
    fn classifies_unknown_as_dev() {
        let c = ctx();
        assert_eq!(classify(None, None, None, &c), ServiceBucket::Dev);
        assert_eq!(classify(Some(""), None, None, &c), ServiceBucket::Dev);
        assert_eq!(classify(Some("my-custom-server"), None, None, &c), ServiceBucket::Dev);
    }
}
