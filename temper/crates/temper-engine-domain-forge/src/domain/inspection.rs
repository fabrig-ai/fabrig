//! Shared reads for temper's adapter (temper/docs/design/testing/engine-checks.md, 2).
use super::Domain;
use crate::{ChangeRow, Hold, LandingRow, Name, ReleaseRow};
use alloc::boxed::Box;
use skein_lib::{List, Map, Time, Token, Wall};

impl Domain {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_limits(&self) -> &crate::Limits {
        &self.landing.limits
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_landings(&self) -> &Map<u64, LandingRow> {
        &self.landing.rows
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_changes(&self) -> &Map<u64, ChangeRow> {
        &self.changes
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_holds(&self) -> &Map<Name, Hold> {
        &self.holds
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_releases(&self) -> &Map<u64, ReleaseRow> {
        &self.releases
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_client(&self) -> &skein_forge_client::Domain {
        &self.client
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_entries(&self) -> Box<[u64]> {
        let mut keys = List::with_capacity(self.entries.len());
        for (key, _) in &self.entries {
            keys.push(*key).expect("counted entries");
        }
        keys.into_boxed()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_entry(&self, key: u64) -> Option<&skein_forge_client::Entry> {
        Some(&self.entries.get(&key)?.entry)
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_entry_capacity(&self) -> u32 {
        self.entries.capacity()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_landing_due(&self, task: u64) -> Option<Wall> {
        self.landing.due.get(&task).copied()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    /// A turn change has settled its gates, without a delegate terminal guessed by the reader.
    #[must_use]
    pub fn inspection_gates_settled(&self, task: u64) -> bool {
        let Some(row) = self.changes.get(&task) else { return false };
        crate::settling::gates_settled(row, None, self.landing.limits.change_policy.gates)
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_change_due(&self, task: u64) -> Option<Wall> {
        self.change_due.get(&task).copied()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    /// The completed follow check's receipt ends the workflow wait at its head.
    #[must_use]
    pub fn inspection_follow_check_ended(&self, task: u64) -> Option<Wall> {
        let row = self.changes.get(&task)?;
        let check = row.follow_check.as_ref()?;
        let temper_engine_domain_forge_change::State::Gating { head, .. } = row.change.state else { return None };
        check.key?;
        self.repository(row.repository)?;
        if check.head != head || !crate::follow_check::has_verdict(row, head) {
            return None;
        }
        Some(self.receipt(&crate::follow_check::receipt_key(self, row))?.ended)
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_step_retry(&self, task: u64) -> Option<Time> {
        for (_, step) in &self.steps {
            if step.task == task {
                return step.retry_at;
            }
        }
        None
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    /// Writes still answering an owner's effect settlement, including withdrawn writes.
    #[must_use]
    pub fn inspection_settling_entries(&self, task: u64) -> Box<[u64]> {
        let Some(release) = self.releases.get(&task) else { return Box::new([]) };
        if release.effects_settled {
            return Box::new([]);
        }
        let mut entries = List::with_capacity(self.entries.len());
        for (number, row) in &self.entries {
            if row.entry.task == task && !row.held {
                entries.push(*number).expect("counted settling entries");
            }
        }
        entries.into_boxed()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_making_waiters(&self) -> Box<[u64]> {
        let mut tasks = List::with_capacity(self.makings.len());
        for (_, making) in &self.makings {
            if making.outbox_wait.is_some() {
                tasks.push(making.task).expect("counted makings");
            }
        }
        tasks.into_boxed()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_read_task(&self, owner: Token) -> Option<u64> {
        if let Some(step) = self.steps.get(&owner) {
            return Some(step.task);
        }
        if let Some(pending) = self.landing.pending.get(&owner) {
            return Some(pending.task);
        }
        if let Some(lost) = self.lost.get(&owner) {
            return Some(lost.task);
        }
        match self.makings.get(&owner) {
            Some(making) => Some(making.task),
            None => None,
        }
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_access_wait(&self, task: u64) -> Option<&Wall> {
        self.change_connections.as_ref()?.get(&task)
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_step_dependencies(&self, task: u64) -> Box<[u64]> {
        let mut tasks = List::uncapped();
        for (_, step) in &self.steps {
            if step.task == task {
                for pending in &step.pending {
                    tasks.push(pending.task).expect("counted dependencies");
                }
            }
        }
        tasks.into_boxed()
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    /// The owner and queued state of a follow change, where its entry still stands.
    #[must_use]
    pub fn inspection_follow_wait(&self, task: u64) -> Option<(crate::settings::FollowOwner, &crate::FollowState)> {
        for (_, row) in &self.follow_rows {
            for entry in &row.queue {
                if entry.change == Some(task) {
                    let setting = self.follow(row.project, row.follow)?;
                    return Some((setting.owner, &entry.state));
                }
            }
        }
        None
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    /// An owner's start or skip remakes the held tail after this queued ended dependency.
    #[must_use]
    pub fn inspection_follow_restart_owner(&self, task: u64, dependency: u64) -> Option<crate::settings::FollowOwner> {
        for (_, row) in &self.follow_rows {
            let mut ended = false;
            for entry in &row.queue {
                if entry.change == Some(dependency) {
                    ended = match entry.state {
                        crate::FollowState::Ended(_) => !entry.remake,
                        crate::FollowState::NotMade
                        | crate::FollowState::WaitsWord
                        | crate::FollowState::HeldByOwner(_)
                        | crate::FollowState::WaitsFor(_)
                        | crate::FollowState::Producing
                        | crate::FollowState::Held(_) => false,
                    };
                }
                if entry.change == Some(task) {
                    return match entry.state {
                        crate::FollowState::Held(_) if ended => Some(self.follow(row.project, row.follow)?.owner),
                        crate::FollowState::Held(_)
                        | crate::FollowState::NotMade
                        | crate::FollowState::WaitsWord
                        | crate::FollowState::HeldByOwner(_)
                        | crate::FollowState::WaitsFor(_)
                        | crate::FollowState::Producing
                        | crate::FollowState::Ended(_) => None,
                    };
                }
            }
        }
        None
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_lost_read_waiters(&self) -> Box<[u64]> {
        let mut tasks = List::with_capacity(self.lost.len());
        for (_, pending) in &self.lost {
            if pending.state == super::LostReadState::WaitingForRoom {
                tasks.push(pending.task).expect("counted lost readers");
            }
        }
        tasks.into_boxed()
    }
}
