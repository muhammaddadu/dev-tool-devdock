use std::path::PathBuf;

/// Resolve the directory where DevDock stores its SQLite DB, logs, and cache.
///
/// macOS: `~/Library/Application Support/DevDock`
/// Linux: `$XDG_DATA_HOME/devdock` (falling back to `~/.local/share/devdock`)
pub fn app_data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        #[cfg(target_os = "macos")]
        {
            return home.join("Library/Application Support/DevDock");
        }
        #[cfg(target_os = "linux")]
        {
            if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
                return PathBuf::from(xdg).join("devdock");
            }
            return home.join(".local/share/devdock");
        }
        #[allow(unreachable_code)]
        {
            return home.join(".devdock");
        }
    }
    PathBuf::from(".devdock")
}
