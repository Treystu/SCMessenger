//! Structural-mismatch quarantine (#684 part B).
//!
//! A dial candidate can be *structurally* impossible rather than merely
//! unlucky: the address answers as a peer that is not the peer the request
//! names, so no amount of retrying can ever satisfy the request. #679 measured
//! exactly that on device -- the same message reached `attempt=630 pass=629`
//! against a loopback address that answered as a different peer id.
//!
//! Retrying it is pure cost, and on device it was not a bounded cost: the retry
//! cycle restarted, the log had no cap, and one stuck message wrote 4.19 GB over
//! a month.
//!
//! This module is the store for those verdicts. It is deliberately small and
//! clock-injected: callers pass the current [`Instant`], so it is testable
//! without sleeping and it never reads a clock of its own.
//!
//! Contract:
//! * Keyed by `(requested peer, address key)`, so the SAME address is still
//!   usable for a different peer and the same peer is still dialable at a
//!   different address. The mismatch is a property of the pair.
//! * Every entry expires. A quarantine with no expiry would be a permanent peer
//!   ban by another name, which this project forbids: "never give up" is the
//!   dial-policy doctrine, and the cure here is "stop retrying an impossible
//!   pair until the evidence is re-tested", not "block the peer".

use std::collections::HashMap;
use web_time::{Duration, Instant};

/// How long a structurally impossible pair stays out of candidate selection.
///
/// Long enough to stop a 506-restart loop cold, short enough that a device
/// which later shares that address with the right peer (DHCP reuse, relay port
/// reuse, a peer re-binding after a network change) re-tests it within the same
/// session rather than after a reinstall.
pub const DEFAULT_STRUCTURAL_QUARANTINE_TTL: Duration = Duration::from_secs(1800);

/// Store of `(requested peer, address)` pairs proven structurally impossible.
#[derive(Debug, Clone)]
pub struct StructuralQuarantine {
    ttl: Duration,
    entries: HashMap<(String, String), Instant>,
}

impl Default for StructuralQuarantine {
    fn default() -> Self {
        Self::new(DEFAULT_STRUCTURAL_QUARANTINE_TTL)
    }
}

impl StructuralQuarantine {
    /// Create a store whose verdicts live for `ttl`.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: HashMap::new(),
        }
    }

    /// The TTL this store applies, for logging and tests.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Record that dialing `addr_key` for `requested` cannot satisfy the
    /// request. Returns true when this is a new verdict, false when the pair was
    /// already quarantined (in which case the deadline is extended, so repeated
    /// evidence keeps the pair out without duplicating entries).
    pub fn mark_mismatch(&mut self, requested: &str, addr_key: &str, now: Instant) -> bool {
        let newly = !self.is_quarantined(requested, addr_key, now);
        self.entries.insert(
            (requested.to_string(), addr_key.to_string()),
            now + self.ttl,
        );
        newly
    }

    /// Whether the pair is currently quarantined. An expired verdict is not
    /// quarantined and is not required to have been pruned first.
    pub fn is_quarantined(&self, requested: &str, addr_key: &str, now: Instant) -> bool {
        match self
            .entries
            .get(&(requested.to_string(), addr_key.to_string()))
        {
            Some(until) => *until > now,
            None => false,
        }
    }

    /// Drop every expired verdict. Returns how many entries were removed.
    pub fn prune(&mut self, now: Instant) -> usize {
        let before = self.entries.len();
        self.entries.retain(|_pair, until| *until > now);
        before - self.entries.len()
    }

    /// Number of entries being tracked, expired ones included until pruned.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is tracked.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PEER: &str = "12D3KooWrequestedPeer";
    const OTHER_PEER: &str = "12D3KooWsomeOtherPeer";
    const ADDR: &str = "127.0.0.1:4001";
    const OTHER_ADDR: &str = "127.0.0.1:4002";

    fn now() -> Instant {
        Instant::now()
    }

    #[test]
    fn a_marked_pair_is_quarantined_and_re_marking_extends_without_duplicating() {
        let mut quarantine = StructuralQuarantine::default();
        let t0 = now();

        assert!(
            quarantine.mark_mismatch(PEER, ADDR, t0),
            "first verdict is new"
        );
        assert!(quarantine.is_quarantined(PEER, ADDR, t0));
        assert_eq!(quarantine.len(), 1);

        assert!(
            !quarantine.mark_mismatch(PEER, ADDR, t0 + Duration::from_secs(60)),
            "a repeated verdict for a live pair is not new"
        );
        assert_eq!(quarantine.len(), 1, "no duplicate entry");
        assert!(
            quarantine.is_quarantined(
                PEER,
                ADDR,
                t0 + Duration::from_secs(60) + quarantine.ttl() - Duration::from_secs(1)
            ),
            "the extended deadline is in force"
        );
    }

    #[test]
    fn a_verdict_expires_instead_of_banning_the_peer() {
        let mut quarantine = StructuralQuarantine::new(Duration::from_secs(30));
        let t0 = now();

        assert!(quarantine.mark_mismatch(PEER, ADDR, t0));
        assert!(quarantine.is_quarantined(PEER, ADDR, t0 + Duration::from_secs(29)));
        assert!(
            !quarantine.is_quarantined(PEER, ADDR, t0 + Duration::from_secs(30)),
            "the pair is dialable again once the evidence is stale"
        );
    }

    #[test]
    fn the_verdict_is_scoped_to_the_pair_not_to_either_half() {
        let mut quarantine = StructuralQuarantine::default();
        let t0 = now();
        assert!(quarantine.mark_mismatch(PEER, ADDR, t0));

        assert!(
            !quarantine.is_quarantined(PEER, OTHER_ADDR, t0),
            "the same peer remains dialable at another address"
        );
        assert!(
            !quarantine.is_quarantined(OTHER_PEER, ADDR, t0),
            "the same address remains usable for another peer"
        );
    }

    #[test]
    fn prune_removes_only_expired_entries() {
        let mut quarantine = StructuralQuarantine::new(Duration::from_secs(10));
        let t0 = now();
        quarantine.mark_mismatch(PEER, ADDR, t0);
        quarantine.mark_mismatch(PEER, OTHER_ADDR, t0 + Duration::from_secs(8));
        assert_eq!(quarantine.len(), 2);
        assert!(quarantine.is_empty() == false);

        let removed = quarantine.prune(t0 + Duration::from_secs(12));
        assert_eq!(removed, 1, "the older verdict expired");
        assert_eq!(quarantine.len(), 1);
        assert!(quarantine.is_quarantined(PEER, OTHER_ADDR, t0 + Duration::from_secs(12)));
    }
}
