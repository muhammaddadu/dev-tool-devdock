//! macOS platform adapter.
//!
//! Listening sockets come from `lsof -nP -iTCP -sTCP:LISTEN`. The parser is
//! kept pure (no I/O) so it can be unit-tested from fixtures in
//! `src/tests/fixtures/`. The adapter shells to `lsof` and forwards the output
//! to the parser. Process enrichment (command line, cwd, project root) is M2.

use std::collections::{HashMap, HashSet};
use std::process::Command;

use crate::models::port::{ListeningSocket, Protocol};
use crate::models::process::ProcessInfo;
use crate::platform::traits::{PlatformAdapter, PlatformError, PlatformResult};

const LSOF_ARGS: &[&str] = &["-nP", "-iTCP", "-sTCP:LISTEN"];

pub struct MacOsAdapter {}

impl MacOsAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MacOsAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformAdapter for MacOsAdapter {
    fn list_listening_sockets(&self) -> PlatformResult<Vec<ListeningSocket>> {
        let output = Command::new("lsof").args(LSOF_ARGS).output().map_err(|e| {
            PlatformError::CommandFailed(format!("failed to spawn lsof: {e}"))
        })?;

        // lsof exits with 1 when there are no matching sockets. That's an
        // empty-but-valid result, not a failure. Only treat it as an error if
        // both stdout is empty and there's stderr text we should surface.
        if !output.status.success() && output.stdout.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.trim().is_empty() {
                return Err(PlatformError::CommandFailed(format!(
                    "lsof failed: {}",
                    stderr.trim()
                )));
            }
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_lsof_listening_ports(&stdout))
    }

    fn get_process_infos(&self, pids: &[u32]) -> PlatformResult<HashMap<u32, ProcessInfo>> {
        if pids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut sys = sysinfo::System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, false);

        let mut result = HashMap::with_capacity(pids.len());
        for pid in pids {
            let target = sysinfo::Pid::from_u32(*pid);
            let Some(p) = sys.process(target) else {
                continue;
            };
            let mut info = process_info_from_sysinfo(p);

            // libproc on macOS sometimes returns an empty cmd / no cwd even
            // for healthy user processes. Backfill from `ps` and `lsof` —
            // both small, cheap, and exactly the calls Activity Monitor uses.
            if info.command_line.as_ref().map_or(true, |c| c.is_empty()) {
                if let Some(cmd) = ps_command_line(*pid) {
                    info.command_line = Some(cmd);
                }
            }
            if info.cwd.is_none() {
                if let Some(cwd) = lsof_process_cwd(*pid) {
                    info.cwd = Some(cwd);
                }
            }

            result.insert(*pid, info);
        }
        Ok(result)
    }

    fn get_process_info(&self, pid: u32) -> PlatformResult<Option<ProcessInfo>> {
        Ok(self.get_process_infos(&[pid])?.remove(&pid))
    }

    fn list_processes(&self) -> PlatformResult<Vec<ProcessInfo>> {
        let mut sys = sysinfo::System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, false);
        Ok(sys.processes().values().map(process_info_from_sysinfo).collect())
    }

    fn terminate_process(&self, pid: u32) -> PlatformResult<()> {
        // SIGTERM is what Ctrl-C does in the terminal — gives the process a
        // chance to clean up listeners and child processes. Tree-kill is a
        // separate, more dangerous operation guarded by its own confirmation.
        send_signal(pid, libc::SIGTERM)
    }

    fn terminate_process_tree(&self, pid: u32) -> PlatformResult<()> {
        // Negative pid → entire process group on Unix. Only useful for
        // processes that became group leaders; for arbitrary external
        // processes we may not know the group, but SIGTERM to -pid is still
        // a best-effort tree kill that we won't escalate past.
        send_signal_to_group(pid, libc::SIGTERM)
    }
}

fn send_signal(pid: u32, sig: libc::c_int) -> PlatformResult<()> {
    // SAFETY: `kill` with a real syscall; we never pass a 0 pid (we'd refuse
    // upstream). Bad pids return -1 / ESRCH which we translate below.
    let result = unsafe { libc::kill(pid as i32, sig) };
    if result == 0 {
        return Ok(());
    }
    let err = std::io::Error::last_os_error();
    Err(PlatformError::CommandFailed(format!(
        "kill({pid}, {sig}) failed: {err}"
    )))
}

fn send_signal_to_group(pid: u32, sig: libc::c_int) -> PlatformResult<()> {
    let result = unsafe { libc::kill(-(pid as i32), sig) };
    if result == 0 {
        return Ok(());
    }
    let err = std::io::Error::last_os_error();
    Err(PlatformError::CommandFailed(format!(
        "kill(-{pid}, {sig}) failed: {err}"
    )))
}

/// Fetch the full command line for a PID via `ps`. sysinfo's macOS backend
/// reads `KERN_PROCARGS2` which can return empty silently; `ps -o command=`
/// is what Activity Monitor uses internally and works on every user process.
fn ps_command_line(pid: u32) -> Option<Vec<String>> {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if line.is_empty() {
        return None;
    }
    // `ps -o command=` joins the original argv with single spaces. We split
    // back into a Vec so the engine's display-shortener sees individual args
    // (it can collapse paths to basenames per-arg). This loses quoting but
    // typical dev commands don't depend on it.
    Some(line.split_whitespace().map(String::from).collect())
}

/// Fetch a PID's current working directory via `lsof -d cwd -Fn`.
///
/// Output looks like:
///   p12345
///   fcwd
///   n/Users/me/code/web
///
/// We scan for the line starting with `n` and return its remainder.
fn lsof_process_cwd(pid: u32) -> Option<String> {
    let output = Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
        .ok()?;
    // lsof exits non-zero with an empty stdout when the process has no
    // matching fd entry — treat that as "unknown", not as a failure.
    if output.stdout.is_empty() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            let path = path.trim();
            if !path.is_empty() {
                return Some(path.to_string());
            }
        }
    }
    None
}

/// Translate a sysinfo Process into our internal ProcessInfo. The fields
/// follow sysinfo's macOS-libproc-backed accessors. `start_time()` returns
/// Unix seconds, which we serialize as a decimal string so the cache key
/// stays human-debuggable.
fn process_info_from_sysinfo(p: &sysinfo::Process) -> ProcessInfo {
    let cmd: Vec<String> = p
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    let cmd = if cmd.is_empty() { None } else { Some(cmd) };

    ProcessInfo {
        pid: p.pid().as_u32(),
        parent_pid: p.parent().map(|pp| pp.as_u32()),
        process_name: p.name().to_string_lossy().to_string(),
        executable_path: p.exe().map(|path| path.to_string_lossy().to_string()),
        command_line: cmd,
        cwd: p.cwd().map(|path| path.to_string_lossy().to_string()),
        started_at: Some(p.start_time().to_string()),
        user: None,
    }
}

/// Parses the output of `lsof -nP -iTCP -sTCP:LISTEN`.
///
/// lsof emits one row per (process, FD). A single listener typically shows up
/// twice (IPv4 and IPv6); we dedupe by (pid, port) so the panel sees one entry
/// per logical service.
///
/// Non-listening rows (e.g. ESTABLISHED connections) are skipped — lsof can be
/// invoked without the filter, and we want the parser to be robust to that.
pub fn parse_lsof_listening_ports(input: &str) -> Vec<ListeningSocket> {
    let mut out = Vec::new();
    let mut seen: HashSet<(u16, Option<u32>)> = HashSet::new();

    for line in input.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with("COMMAND") {
            continue;
        }

        let Some(parsed) = parse_lsof_line(line) else {
            continue;
        };

        let key = (parsed.port, parsed.pid);
        if !seen.insert(key) {
            continue;
        }
        out.push(parsed);
    }

    out
}

fn parse_lsof_line(line: &str) -> Option<ListeningSocket> {
    // Columns: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME...
    // NAME can contain spaces (e.g. "*:3000 (LISTEN)"), so split on whitespace
    // and reconstruct NAME from the 9th token onward.
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 9 {
        return None;
    }

    let command = tokens[0];
    let pid: u32 = tokens[1].parse().ok()?;

    let name_part = tokens[8..].join(" ");
    let addr = name_part.strip_suffix("(LISTEN)")?.trim_end();
    let addr = addr.trim();

    let (host, port) = split_host_port(addr)?;

    Some(ListeningSocket {
        protocol: Protocol::Tcp,
        host: normalize_host(&host),
        port,
        pid: Some(pid),
        process_name: Some(command.to_string()),
    })
}

/// Splits an lsof address token into (host, port). Handles:
///   - "*:3000"
///   - "127.0.0.1:8000"
///   - "[::1]:5173"
///   - "[::]:3000"
fn split_host_port(addr: &str) -> Option<(String, u16)> {
    if let Some(rest) = addr.strip_prefix('[') {
        // IPv6 in brackets: "[host]:port"
        let end = rest.find(']')?;
        let host = &rest[..end];
        let port_str = rest.get(end + 1..)?.strip_prefix(':')?;
        let port: u16 = port_str.parse().ok()?;
        return Some((host.to_string(), port));
    }
    let idx = addr.rfind(':')?;
    let host = &addr[..idx];
    let port: u16 = addr[idx + 1..].parse().ok()?;
    Some((host.to_string(), port))
}

fn normalize_host(host: &str) -> String {
    match host {
        "*" => "0.0.0.0".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../tests/fixtures/macos_lsof_listen.txt");

    #[test]
    fn empty_input_yields_no_sockets() {
        assert!(parse_lsof_listening_ports("").is_empty());
    }

    #[test]
    fn parses_full_fixture() {
        let sockets = parse_lsof_listening_ports(FIXTURE);

        // 6 raw rows: 5 LISTEN + 1 ESTABLISHED. After dedupe of the two
        // node/12345 entries (IPv4 + IPv6 on :3000), we expect 4 entries.
        assert_eq!(sockets.len(), 4, "got: {sockets:#?}");

        let ports: Vec<u16> = sockets.iter().map(|s| s.port).collect();
        assert_eq!(ports, vec![3000, 8000, 5173, 4567]);
    }

    #[test]
    fn dedupes_ipv4_and_ipv6_on_same_pid_port() {
        let sockets = parse_lsof_listening_ports(FIXTURE);
        let on_3000: Vec<_> = sockets.iter().filter(|s| s.port == 3000).collect();
        assert_eq!(on_3000.len(), 1, "expected one entry for :3000 after dedupe");
        assert_eq!(on_3000[0].pid, Some(12345));
    }

    #[test]
    fn normalizes_wildcard_host() {
        let sockets = parse_lsof_listening_ports(FIXTURE);
        let svc = sockets.iter().find(|s| s.port == 3000).unwrap();
        assert_eq!(svc.host, "0.0.0.0");
    }

    #[test]
    fn parses_bracketed_ipv6_host() {
        let sockets = parse_lsof_listening_ports(FIXTURE);
        let svc = sockets.iter().find(|s| s.port == 5173).unwrap();
        assert_eq!(svc.host, "::1");
        assert_eq!(svc.pid, Some(67890));
    }

    #[test]
    fn preserves_loopback_host() {
        let sockets = parse_lsof_listening_ports(FIXTURE);
        let svc = sockets.iter().find(|s| s.port == 8000).unwrap();
        assert_eq!(svc.host, "127.0.0.1");
    }

    #[test]
    fn skips_non_listen_rows() {
        let sockets = parse_lsof_listening_ports(FIXTURE);
        assert!(
            sockets.iter().all(|s| s.port != 22),
            "ESTABLISHED row leaked into parser output"
        );
    }

    #[test]
    fn captures_process_name() {
        let sockets = parse_lsof_listening_ports(FIXTURE);
        let svc = sockets.iter().find(|s| s.port == 5173).unwrap();
        assert_eq!(svc.process_name.as_deref(), Some("node"));
    }

    #[test]
    fn split_host_port_handles_all_forms() {
        assert_eq!(
            split_host_port("*:3000"),
            Some(("*".to_string(), 3000))
        );
        assert_eq!(
            split_host_port("127.0.0.1:8000"),
            Some(("127.0.0.1".to_string(), 8000))
        );
        assert_eq!(
            split_host_port("[::1]:5173"),
            Some(("::1".to_string(), 5173))
        );
        assert_eq!(split_host_port("[::]:3000"), Some(("::".to_string(), 3000)));
        assert_eq!(split_host_port("not-an-address"), None);
    }
}
