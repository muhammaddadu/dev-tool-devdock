use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
}

/// A listening TCP socket as observed in a single cheap OS call (e.g. `lsof`,
/// `ss`). Anything that requires further inspection of the owning process
/// (command line, cwd, project root) belongs in `ProcessInfo` and is added by
/// the process enricher in M2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ListeningSocket {
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub pid: Option<u32>,
    /// Process name as reported by the OS in the same listing call. Truncated
    /// on macOS (lsof's COMMAND is 9 chars by default). Use `ProcessInfo` for
    /// the full command line.
    pub process_name: Option<String>,
}
