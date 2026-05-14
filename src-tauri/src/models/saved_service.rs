use serde::{Deserialize, Serialize};

/// A row in the `services` table — a command the user has explicitly asked
/// DevDock to remember.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SavedService {
    pub id: String,
    pub project_id: Option<String>,
    pub label: String,
    pub command: String,
    pub cwd: String,
    /// Ports we expect this service to bind. Stored in the DB as a JSON array
    /// so the schema doesn't need a join table.
    pub expected_ports: Vec<u16>,
    pub pinned: bool,
    /// Why the row exists: `"detected"` (saved from a running service),
    /// `"manual"` (typed into the Add Command form), `"scan"` (project scan).
    pub created_from: String,
    pub last_run_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
