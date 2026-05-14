//! Linux platform adapter. Wired in M8.

use crate::models::port::ListeningSocket;
use crate::models::process::ProcessInfo;
use crate::platform::traits::{PlatformAdapter, PlatformError, PlatformResult};

pub struct LinuxAdapter {}

impl LinuxAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for LinuxAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformAdapter for LinuxAdapter {
    fn list_listening_sockets(&self) -> PlatformResult<Vec<ListeningSocket>> {
        Ok(Vec::new())
    }

    fn get_process_infos(
        &self,
        _pids: &[u32],
    ) -> PlatformResult<std::collections::HashMap<u32, ProcessInfo>> {
        Ok(std::collections::HashMap::new())
    }

    fn get_process_info(&self, _pid: u32) -> PlatformResult<Option<ProcessInfo>> {
        Ok(None)
    }

    fn list_processes(&self) -> PlatformResult<Vec<ProcessInfo>> {
        Ok(Vec::new())
    }

    fn terminate_process(&self, _pid: u32) -> PlatformResult<()> {
        Err(PlatformError::CommandFailed("not yet implemented".into()))
    }

    fn terminate_process_tree(&self, _pid: u32) -> PlatformResult<()> {
        Err(PlatformError::CommandFailed("not yet implemented".into()))
    }
}

#[allow(dead_code)]
pub fn parse_ss_listening_ports(_input: &str) -> Vec<ListeningSocket> {
    Vec::new()
}
