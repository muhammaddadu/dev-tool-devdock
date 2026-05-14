use crate::models::port::ListeningSocket;
use crate::models::process::ProcessInfo;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("not supported on this platform")]
    NotSupported,
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("command failed: {0}")]
    CommandFailed(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type PlatformResult<T> = Result<T, PlatformError>;

/// Adapter trait abstracting OS-specific behavior away from the core domain.
///
/// All OS-specific code must live behind this trait. The core layer must not
/// shell out to OS tools directly.
pub trait PlatformAdapter: Send + Sync {
    fn list_listening_sockets(&self) -> PlatformResult<Vec<ListeningSocket>>;

    /// Returns full process metadata for the requested PIDs in a single
    /// batch. Implementations should use the cheapest reliable mix of OS
    /// calls — on macOS that's sysinfo plus `ps` / `lsof` fallbacks for
    /// fields libproc leaves empty.
    fn get_process_infos(
        &self,
        pids: &[u32],
    ) -> PlatformResult<std::collections::HashMap<u32, ProcessInfo>>;

    fn get_process_info(&self, pid: u32) -> PlatformResult<Option<ProcessInfo>>;
    fn list_processes(&self) -> PlatformResult<Vec<ProcessInfo>>;
    fn terminate_process(&self, pid: u32) -> PlatformResult<()>;
    fn terminate_process_tree(&self, pid: u32) -> PlatformResult<()>;
}
