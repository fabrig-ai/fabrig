//! One readiness step can hold a train, unstored (temper/docs/design/domain/landing-turn.md, 2).
use crate::{ChangeRow, Domain};
use skein_forge::api::Commit;
use skein_lib::List;
use temper_engine_domain_forge_change as change;

/// The checked head at which a change can settle into its train.
fn settling_head(landing: Option<u64>, row: &ChangeRow) -> Option<Commit> {
    if row.change.way != change::Way::Turn || landing.is_some() || row.terminal() {
        return None;
    }
    match row.change.state {
        change::State::Gating { head, .. } => Some(head),
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

/// Gates passed by the reports a change step would build, or a delegate's
/// terminal is back for that step to consume. This does not grant readiness.
#[must_use]
pub fn gates_settled(d: &Domain, row: &ChangeRow, returned: Option<(u64, Commit)>, capacity: u32) -> bool {
    let Some(head) = settling_head(d.landing_of(row.task), row) else { return false };
    let reports = crate::follow_check::reports(row, &row.verdicts, capacity);
    if change::gates_pass(&row.change, &reports, head, row.spec_revision) {
        return true;
    }
    let Some((number, at)) = returned else { return false };
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

impl Domain {
    /// Spend the hold only once at this gating head; repeat judgements of the
    /// same due step still hold until it ends.
    pub fn hold_forming(&mut self, task: u64, step: u64) -> bool {
        let Some(row) = self.change(task) else { return false };
        let Some(head) = settling_head(self.landing_of(task), row) else { return false };
        if let Some((spent, at)) = self.forming_holds.get(&task)
            && *at == head
        {
            return *spent == step;
        }
        self.forming_holds.insert(task, (step, head)).expect("one forming hold per bounded change");
        true
    }
}

pub(crate) fn forget(d: &mut Domain, row: &ChangeRow) {
    if let Some((_, head)) = d.forming_holds.get(&row.task)
        && settling_head(d.landing_of(row.task), row) != Some(*head)
    {
        d.forming_holds.remove(&row.task);
    }
}
