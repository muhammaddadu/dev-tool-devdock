//! Windows stub. Returns NotSupported for all operations.

use crate::models::port::ListeningSocket;
use crate::models::process::ProcessInfo;
use crate::platform::traits::{PlatformAdapter, PlatformError, PlatformResult};

pub struct WindowsStubAdapter {}

impl WindowsStubAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WindowsStubAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformAdapter for WindowsStubAdapter {
    fn list_listening_sockets(&self) -> PlatformResult<Vec<ListeningSocket>> {
        Err(PlatformError::NotSupported)
    }

    fn get_process_infos(
        &self,
        _pids: &[u32],
    ) -> PlatformResult<std::collections::HashMap<u32, ProcessInfo>> {
        Err(PlatformError::NotSupported)
    }

    fn get_process_info(&self, _pid: u32) -> PlatformResult<Option<ProcessInfo>> {
        Err(PlatformError::NotSupported)
    }

    fn list_processes(&self) -> PlatformResult<Vec<ProcessInfo>> {
        Err(PlatformError::NotSupported)
    }

    fn terminate_process(&self, _pid: u32) -> PlatformResult<()> {
        Err(PlatformError::NotSupported)
    }

    fn terminate_process_tree(&self, _pid: u32) -> PlatformResult<()> {
        Err(PlatformError::NotSupported)
    }
}
