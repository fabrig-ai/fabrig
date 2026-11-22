//! Borrowed inspection of the task child's own tables and armed deadlines.
use crate::{Domain, Name, PoolSlots, ResourceReport, TaskRecord};
use core::ops::Bound::{Excluded, Unbounded};
use skein_lib::{Deadlines, Map, Set, Token};

impl Domain {
    /// Borrow the next named live row without building a projection for the world's adapter.
    /// `None` starts in name order; otherwise the row follows `after` strictly.
    #[must_use]
    pub fn inspection_task_after(&self, after: Option<u64>) -> Option<&TaskRecord> {
        let (_, id) = match after {
            None => self.names.first(),
            Some(number) => self.names.range((Excluded(number), Unbounded)).next(),
        }?;
        Some(&self.tasks.get(*id).expect("named live task").record)
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_subscription_capacity(&self) -> u32 {
        self.subscription_capacity
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_reports(&self) -> &Map<Name, ResourceReport> {
        &self.resources
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_report_pins(&self) -> &Map<Token, Set<Name>> {
        &self.report_pins
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_pools(&self) -> &Map<Name, PoolSlots> {
        &self.pools
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_holders(&self) -> &Map<(Name, u64), ()> {
        &self.waits.holders
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_waiters(&self) -> &Map<(Name, u64), ()> {
        &self.waits.waiters
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_alarms(&self) -> &Deadlines<u64> {
        &self.alarms
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_grace_alarms(&self) -> &Deadlines<u64> {
        &self.grace_alarms
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_timers(&self) -> &Deadlines<u64> {
        &self.timers
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_wakes(&self) -> &Deadlines<u64> {
        &self.wakes
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_proposal_alarms(&self) -> &Deadlines<u64> {
        &self.proposal_alarms
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_escalation_alarms(&self) -> &Deadlines<u64> {
        &self.escalation_alarms
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_hold_alarms(&self) -> &Deadlines<u64> {
        &self.hold_alarms
    }
}
