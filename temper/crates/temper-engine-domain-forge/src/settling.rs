//! A release into a train waits for the change's first decision (temper/docs/design/domain/landing-turn.md, 2).
use crate::{ChangeRow, Domain, Request, Stored};
use alloc::boxed::Box;
use skein_forge::api::{Commit, Repository};
use skein_lib::{List, Queue};
use temper_engine_domain_forge_change as change;

/// The checked head at which a change can settle into its train.
pub(crate) fn settling_head(landing: Option<u64>, row: &ChangeRow) -> Option<Commit> {
    if row.change.way != change::Way::Turn || landing.is_some() || row.frozen() {
        return None;
    }
    match row.change.state {
        change::State::Gating { head, .. } => {
            if row.change.last_head == Some(head) {
                Some(head)
            } else {
                None
            }
        }
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::CancellingGates { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Landed { .. }
        | change::State::NoChange { .. }
        | change::State::Ended
        | change::State::Held { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::Judging { .. } => None,
    }
}

/// The blocking gates at the checked head have passed, or the last one's
/// terminal has arrived. The terminal leaves its verdict to the change step.
#[must_use]
pub fn gates_settled(d: &Domain, row: &ChangeRow, returned: Option<(u64, Commit)>, capacity: u32) -> bool {
    let Some(head) = settling_head(d.landing_of(row.task), row) else { return false };
    let reports = crate::follow_check::reports(row, &row.verdicts, capacity);
    let Some((number, at)) = returned else {
        return change::gates_pass(&row.change, &reports, head, row.spec_revision);
    };
    if at != head || !blocking(row, number) {
        return false;
    }
    let mut settling = List::with_capacity(capacity);
    for report in &reports {
        if report.number != number {
            settling.push(*report).expect("bounded retained gate reports");
        }
    }
    // A terminal settles this gate for one step, even if the step will reject it.
    if settling
        .push(change::GateReport { number, head: at, revision: row.gate_revision, status: change::Status::Passed })
        .is_err()
    {
        return false;
    }
    change::gates_pass(&row.change, settling.as_slice(), head, row.spec_revision)
}

fn blocking(row: &ChangeRow, number: u64) -> bool {
    for gate in &row.change.gates {
        if gate.number == number {
            return gate.blocking;
        }
    }
    false
}

impl Domain {
    /// Settling changes into one train, in task order.
    #[must_use]
    pub fn settling_changes(&self, repository: Repository, branch: &[u8], most: u32) -> Box<[u64]> {
        let mut changes = List::with_capacity(most.min(self.changes.len()));
        for (task, row) in &self.changes {
            if changes.room() == 0 {
                break;
            }
            if row.settling && row.repository == repository && row.base.as_ref() == branch {
                changes.push(*task).expect("room checked");
            }
        }
        changes.into_boxed()
    }

    /// Store the release or its end in the decision that takes the fact.
    pub fn set_settling(&mut self, task: u64, settling: bool, out: &mut Queue<Request>) {
        let Some(row) = self.changes.get(&task) else { return };
        if row.settling == settling || (settling && settling_head(self.landing_of(task), row).is_none()) {
            return;
        }
        let row = self.changes.get_mut(&task).expect("known change");
        row.settling = settling;
        let row = row.clone();
        mark_train_waits(self, &row);
        let (_, hot) = row.split();
        out.push(Request::Save { record: Stored::ChangeHot(Box::new(hot)), release: None, read_afresh: None });
    }
}

pub(crate) fn mark_train_waits(d: &mut Domain, row: &ChangeRow) {
    for (task, other) in &d.changes {
        if other.repository == row.repository && other.base == row.base && crate::landing::waiting(d, other) {
            d.wait_reports.changed(*task);
        }
    }
}

pub(crate) fn head_seen(d: &mut Domain, task: u64, head: Option<Commit>, out: &mut Queue<Request>) {
    if let Some(row) = d.change(task)
        && row.settling
        && row.change.last_head != head
    {
        d.set_settling(task, false, out);
    }
}
