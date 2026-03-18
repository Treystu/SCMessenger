//! Hierarchical Timeout Budgeting for DHT Peer Discovery
//!
//! Instead of fixed 500ms timeouts per phase, uses a total time budget with
//! progressive fallback through discovery phases. This reduces worst-case
//! latency from 2000ms (4 phases × 500ms) to 500ms total budget.
//!
//! # Design Principles
//!
//! 1. **Budget-based**: Total time budget for entire discovery, not per-phase
//! 2. **Early termination**: Stop as soon as a route is found
//! 3. **Progressive fallback**: Cheaper methods first, expensive last
//! 4. **Deterministic**: Same inputs produce same phase transitions

use std::time::{Duration, Instant};

/// Discovery phases in order of increasing cost
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryPhase {
    /// Phase 1: Local cache lookup (cheapest, ~1ms)
    LocalCache,
    /// Phase 2: Neighborhood gossip query (~10-50ms)
    NeighborhoodQuery,
    /// Phase 3: Targeted delegate query (~50-200ms)
    DelegateQuery,
    /// Phase 4: Full DHT walk (most expensive, ~200-500ms)
    FullDhtWalk,
}

impl DiscoveryPhase {
    /// Get the estimated duration for this phase
    pub fn estimated_duration(&self) -> Duration {
        match self {
            DiscoveryPhase::LocalCache => Duration::from_millis(10),
            DiscoveryPhase::NeighborhoodQuery => Duration::from_millis(50),
            DiscoveryPhase::DelegateQuery => Duration::from_millis(200),
            DiscoveryPhase::FullDhtWalk => Duration::from_millis(500),
        }
    }

    /// Get the next phase, if any
    pub fn next(&self) -> Option<DiscoveryPhase> {
        match self {
            DiscoveryPhase::LocalCache => Some(DiscoveryPhase::NeighborhoodQuery),
            DiscoveryPhase::NeighborhoodQuery => Some(DiscoveryPhase::DelegateQuery),
            DiscoveryPhase::DelegateQuery => Some(DiscoveryPhase::FullDhtWalk),
            DiscoveryPhase::FullDhtWalk => None,
        }
    }
}

/// Budget tracker for hierarchical discovery
///
/// Manages a total time budget across all discovery phases, ensuring
/// the entire discovery completes within the budget even if some phases
/// timeout.
#[derive(Debug, Clone)]
pub struct TimeoutBudget {
    /// Total time budget for this discovery
    total_budget: Duration,
    /// When this budget was created
    start_time: Instant,
    /// Current phase
    phase: DiscoveryPhase,
    /// Time spent in current phase
    phase_start: Instant,
    /// Whether budget is exhausted
    exhausted: bool,
}

impl TimeoutBudget {
    /// Create a new timeout budget with the given total duration
    pub fn new(total_budget: Duration) -> Self {
        let now = Instant::now();
        TimeoutBudget {
            total_budget,
            start_time: now,
            phase: DiscoveryPhase::LocalCache,
            phase_start: now,
            exhausted: false,
        }
    }

    /// Create a budget with default 500ms total
    pub fn default_500ms() -> Self {
        Self::new(Duration::from_millis(500))
    }

    /// Get the current phase
    pub fn current_phase(&self) -> DiscoveryPhase {
        self.phase
    }

    /// Get time elapsed since budget creation
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get time remaining in the budget
    pub fn remaining(&self) -> Duration {
        self.total_budget.saturating_sub(self.elapsed())
    }

    /// Get time spent in current phase
    pub fn phase_elapsed(&self) -> Duration {
        self.phase_start.elapsed()
    }

    /// Check if the budget is exhausted
    pub fn is_exhausted(&self) -> bool {
        self.exhausted || self.remaining().is_zero()
    }

    /// Check if we should advance to the next phase
    ///
    /// Returns true if:
    /// - Current phase has timed out (no result yet)
    /// - There's enough budget remaining for the next phase
    pub fn should_advance(&self) -> bool {
        if self.exhausted {
            return false;
        }

        let remaining = self.remaining();

        // Check if there's enough budget for at least one more phase
        match self.phase.next() {
            Some(next_phase) => {
                // Need at least 10ms for the next phase to be meaningful
                remaining.as_millis() >= 10
            }
            None => false, // Already at last phase
        }
    }

    /// Advance to the next phase
    ///
    /// Returns the new phase, or None if budget is exhausted
    pub fn advance(&mut self) -> Option<DiscoveryPhase> {
        if self.exhausted {
            return None;
        }

        match self.phase.next() {
            Some(next_phase) => {
                let remaining = self.remaining();

                // Check if we have enough budget for this phase
                let phase_estimate = next_phase.estimated_duration();
                if remaining < phase_estimate / 2 {
                    // Less than half the estimated time remaining
                    // Mark as exhausted but still allow the attempt
                    self.exhausted = true;
                }

                self.phase = next_phase;
                self.phase_start = Instant::now();
                Some(next_phase)
            }
            None => {
                self.exhausted = true;
                None
            }
        }
    }

    /// Mark the budget as exhausted (e.g., route found, stop searching)
    pub fn complete(&mut self) {
        self.exhausted = true;
    }

    /// Get the total budget duration
    pub fn total_budget(&self) -> Duration {
        self.total_budget
    }

    /// Check if we're in the final phase
    pub fn is_final_phase(&self) -> bool {
        self.phase.next().is_none()
    }

    /// Get a summary of budget usage
    pub fn summary(&self) -> BudgetSummary {
        BudgetSummary {
            total_budget: self.total_budget,
            elapsed: self.elapsed(),
            remaining: self.remaining(),
            current_phase: self.phase,
            phase_elapsed: self.phase_elapsed(),
            is_exhausted: self.exhausted,
        }
    }
}

/// Summary of budget usage for logging/metrics
#[derive(Debug, Clone)]
pub struct BudgetSummary {
    pub total_budget: Duration,
    pub elapsed: Duration,
    pub remaining: Duration,
    pub current_phase: DiscoveryPhase,
    pub phase_elapsed: Duration,
    pub is_exhausted: bool,
}

impl std::fmt::Display for BudgetSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Budget: {:?} elapsed, {:?} remaining, phase: {:?} ({:?} in phase), exhausted: {}",
            self.elapsed, self.remaining, self.current_phase, self.phase_elapsed, self.is_exhausted
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_creation() {
        let budget = TimeoutBudget::new(Duration::from_millis(500));
        assert_eq!(budget.current_phase(), DiscoveryPhase::LocalCache);
        assert!(!budget.is_exhausted());
        assert!(!budget.is_final_phase());
    }

    #[test]
    fn test_phase_progression() {
        let mut budget = TimeoutBudget::new(Duration::from_millis(500));

        assert_eq!(budget.current_phase(), DiscoveryPhase::LocalCache);

        let next = budget.advance();
        assert_eq!(next, Some(DiscoveryPhase::NeighborhoodQuery));
        assert_eq!(budget.current_phase(), DiscoveryPhase::NeighborhoodQuery);

        let next = budget.advance();
        assert_eq!(next, Some(DiscoveryPhase::DelegateQuery));

        let next = budget.advance();
        assert_eq!(next, Some(DiscoveryPhase::FullDhtWalk));

        let next = budget.advance();
        assert_eq!(next, None);
        assert!(budget.is_exhausted());
    }

    #[test]
    fn test_budget_completion() {
        let mut budget = TimeoutBudget::new(Duration::from_millis(500));
        budget.complete();
        assert!(budget.is_exhausted());
        assert_eq!(budget.advance(), None);
    }

    #[test]
    fn test_budget_summary() {
        let budget = TimeoutBudget::new(Duration::from_millis(500));
        let summary = budget.summary();
        assert_eq!(summary.total_budget, Duration::from_millis(500));
        assert_eq!(summary.current_phase, DiscoveryPhase::LocalCache);
        assert!(!summary.is_exhausted);
    }

    #[test]
    fn test_default_500ms() {
        let budget = TimeoutBudget::default_500ms();
        assert_eq!(budget.total_budget(), Duration::from_millis(500));
    }

    #[test]
    fn test_phase_estimated_durations() {
        assert_eq!(
            DiscoveryPhase::LocalCache.estimated_duration(),
            Duration::from_millis(10)
        );
        assert_eq!(
            DiscoveryPhase::NeighborhoodQuery.estimated_duration(),
            Duration::from_millis(50)
        );
        assert_eq!(
            DiscoveryPhase::DelegateQuery.estimated_duration(),
            Duration::from_millis(200)
        );
        assert_eq!(
            DiscoveryPhase::FullDhtWalk.estimated_duration(),
            Duration::from_millis(500)
        );
    }
}
