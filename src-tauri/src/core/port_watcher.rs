//! Background watcher: cheap socket scan + diff. M1+ implementation.

use std::collections::HashSet;

use crate::models::port::ListeningSocket;

/// Computes the symmetric diff between two socket snapshots.
///
/// Returns (added, removed) keyed by (host, port, pid). The watcher only
/// triggers process enrichment for added/changed PIDs.
pub fn diff_snapshots(
    prev: &[ListeningSocket],
    next: &[ListeningSocket],
) -> (Vec<ListeningSocket>, Vec<ListeningSocket>) {
    let prev_set: HashSet<_> = prev.iter().collect();
    let next_set: HashSet<_> = next.iter().collect();

    let added = next_set
        .difference(&prev_set)
        .map(|s| (*s).clone())
        .collect();
    let removed = prev_set
        .difference(&next_set)
        .map(|s| (*s).clone())
        .collect();

    (added, removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::port::Protocol;

    fn sock(port: u16, pid: Option<u32>) -> ListeningSocket {
        ListeningSocket {
            protocol: Protocol::Tcp,
            host: "127.0.0.1".into(),
            port,
            pid,
            process_name: None,
        }
    }

    #[test]
    fn diff_detects_added_and_removed() {
        let prev = vec![sock(3000, Some(1)), sock(8000, Some(2))];
        let next = vec![sock(8000, Some(2)), sock(5173, Some(3))];

        let (added, removed) = diff_snapshots(&prev, &next);
        assert_eq!(added, vec![sock(5173, Some(3))]);
        assert_eq!(removed, vec![sock(3000, Some(1))]);
    }

    #[test]
    fn diff_returns_empty_when_unchanged() {
        let s = vec![sock(3000, Some(1))];
        let (added, removed) = diff_snapshots(&s, &s);
        assert!(added.is_empty());
        assert!(removed.is_empty());
    }
}
