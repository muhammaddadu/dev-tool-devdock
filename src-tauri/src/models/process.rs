use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub command_line: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub started_at: Option<String>,
    pub user: Option<String>,
}
