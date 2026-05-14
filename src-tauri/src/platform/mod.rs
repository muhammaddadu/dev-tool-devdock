//! Platform adapter layer.
//!
//! Core code talks to `traits::PlatformAdapter` only. The concrete adapter is
//! selected at runtime by `default_adapter()` based on the target OS.

use std::sync::Arc;

pub mod traits;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows_stub;

/// Returns the platform adapter for the current OS.
///
/// macOS is the primary target. Linux is architecture-ready (M8). Windows is a
/// stub that returns NotSupported errors.
pub fn default_adapter() -> Arc<dyn traits::PlatformAdapter> {
    #[cfg(target_os = "macos")]
    {
        Arc::new(macos::MacOsAdapter::new())
    }
    #[cfg(target_os = "linux")]
    {
        Arc::new(linux::LinuxAdapter::new())
    }
    #[cfg(target_os = "windows")]
    {
        Arc::new(windows_stub::WindowsStubAdapter::new())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        compile_error!("Unsupported target OS");
    }
}
