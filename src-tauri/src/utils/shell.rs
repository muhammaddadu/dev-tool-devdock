//! Shell helpers for launching user commands. M4 implementation.
//!
//! Commands are run via `sh -c` on macOS/Linux. Future work: respect the user's
//! preferred login shell so PATH and shell aliases match terminal behavior.
