//! Writes stdout/stderr from managed services to log files. M4 implementation.

use std::path::PathBuf;

pub struct LogWriter {
    pub path: PathBuf,
}

impl LogWriter {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
