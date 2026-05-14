use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Running,
    Starting,
    Stopped,
    Crashed,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceSource {
    Detected,
    Managed,
}

/// Where a service should be displayed in the panel.
///
/// * `Dev` — front-of-mind. The developer's actual work.
/// * `Tooling` — IDE helpers, language servers, debug bridges, and DevDock
///   itself. Things the user runs but doesn't think of as "their service."
/// * `System` — macOS daemons and consumer apps (Dropbox, Slack, etc.) that
///   happen to bind ports.
///
/// Tooling and System are both collapsed by default; keeping them separate
/// preserves an honest taxonomy and lets each section behave independently.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceBucket {
    Dev,
    Tooling,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceView {
    pub id: String,
    pub label: String,
    pub status: ServiceStatus,
    pub source: ServiceSource,
    pub bucket: ServiceBucket,
    pub port: Option<u16>,
    pub host: Option<String>,
    pub url: Option<String>,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub project_name: Option<String>,
    pub project_root: Option<String>,
    /// Process start time as Unix seconds. The frontend formats this into a
    /// relative "X minutes ago" label; keeping it numeric avoids locking the
    /// engine to a presentation format.
    pub started_at_unix: Option<u64>,
    /// When set, this view corresponds to a row in the `services` table.
    /// Drives Save/Unsave/Pin button state in the frontend.
    pub saved_id: Option<String>,
    /// Filesystem path to the stdout/stderr capture for the currently-managed
    /// run. Only populated when DevDock launched the service. The frontend
    /// uses this to surface the "View Logs" menu entry.
    pub log_path: Option<String>,
    pub pinned: bool,
    pub can_open: bool,
    pub can_save: bool,
    pub can_run: bool,
    pub can_stop: bool,
    pub can_restart: bool,
    pub can_kill: bool,
}
