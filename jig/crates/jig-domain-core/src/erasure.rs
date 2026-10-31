//! The planner of an ended task's whole erasure
//! (`temper/docs/design/draft/playground-reset.md`, 5;
//! `jig/docs/design/protocol/store.md`, 11.1).
#![cfg_attr(not(test), expect(dead_code, reason = "project deletion will call the erasure planner"))]

use crate::{CoreKey, Key, ProposalDecisionRecord, Record, forget, indexes};
use jig_domain_people::Proposer;
use jig_domain_tasks as tasks;
use skein_lib::List;

/// The most keys one record or row can plan, to size `out`.
pub(crate) const MOST_KEYS: u32 = 5;

/// The keys an ended task's record alone names: its party and project
/// indexes and its requester's edge, then `Ended` itself, last.
pub(crate) fn record_keys(record: &tasks::TaskRecord, out: &mut List<Key>) -> Option<()> {
    match record.requester {
        tasks::Party::Person(party) => {
            if record.result_position != 0 {
                let at = record.ended_at?;
                out.push(Key::Tasks(tasks::Key::Result { party, position: record.result_position })).ok()?;
                out.push(Key::Tasks(forget::result_update(record.number, party, at).key())).ok()?;
                out.push(Key::Tasks(forget::ended_by_party(record.number, party, record.label, at).key())).ok()?;
            }
        }
        tasks::Party::Task(requester) => {
            out.push(Key::Tasks(tasks::Key::EndedDelegate { requester, task: record.number })).ok()?;
            out.push(Key::Tasks(tasks::Key::Delegate { requester, task: record.number })).ok()?;
        }
        tasks::Party::Deployment { .. } => {}
    }
    if record.flags.listed {
        let index = forget::ended_by_project(record.number, record.project, record.label, record.ended_at?);
        out.push(Key::Tasks(index.key())).ok()?;
    }
    out.push(Key::Tasks(tasks::Key::Ended(record.number))).ok()
}

/// A row a `TaskRows` read gave: its own key, then any index key the
/// core derived from it when it was saved. `requester` is the
/// requester of the task being erased (its `Ended` record's), `None`
/// for a row read outside a task's erasure.
pub(crate) fn row_keys(row: &Record, requester: Option<tasks::Party>, out: &mut List<Key>) -> Option<()> {
    out.push(row.key()).ok()?;
    match row {
        Record::Tasks(tasks::Stored::Activity(line)) => {
            if let Some(update) = indexes::activity_update(line, requester) {
                out.push(Key::Tasks(update.key())).ok()?;
            }
        }
        Record::Tasks(
            tasks::Stored::PlacedHold(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::EndedByParty(_)
            | tasks::Stored::EndedByProject(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::DecidedProposal(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_),
        )
        | Record::Core(_)
        | Record::People(_)
        | Record::Notes(_)
        | Record::Accounts(_) => {}
    }
    Some(())
}

/// A decided person proposal a `DecidedProposals` page gave, with the
/// decision record the page joined to it (`None` when none was saved):
/// its own key, the decision record's key, then the proposer's
/// `Update` the core derived from the decision when it was saved.
pub(crate) fn proposal_keys(
    row: &tasks::PersonProposal,
    decision: Option<&ProposalDecisionRecord>,
    out: &mut List<Key>,
) -> Option<()> {
    out.push(Key::Tasks(tasks::Key::DecidedProposal(row.number))).ok()?;
    out.push(Key::Core(CoreKey::ProposalDecision { proposer: Proposer::Party(row.proposer), proposal: row.number }))
        .ok()?;
    if let Some(decision) = decision {
        out.push(Key::Tasks(indexes::proposal_update(row, decision.at).key())).ok()?;
    }
    Some(())
}
