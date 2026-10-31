//! Core-owned task indexes, saved in the owner's decision (store.md 7.4).
use crate::{Core, Record, Request, Write};
use jig_domain_tasks as tasks;
use skein_lib::Queue;

pub(crate) fn save(core: &Core, record: &tasks::Stored, at: skein_lib::Wall, out: &mut Queue<Request>) {
    let update = match record {
        tasks::Stored::Ended(task) => {
            match task.requester {
                tasks::Party::Person(party) => {
                    let row = tasks::ResultRow { party, position: task.result_position, task: task.number };
                    out.push(Request::Write(Write::Save(Record::Tasks(tasks::Stored::Result(row)))));
                }
                tasks::Party::Task(_) | tasks::Party::Deployment { .. } => {}
            }
            match task.requester {
                tasks::Party::Person(party) => Some(tasks::Update {
                    party,
                    at: task.ended_at.expect("ended task has a time"),
                    task: task.number,
                    object: tasks::UpdateObject::Result,
                }),
                tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
            }
        }
        tasks::Stored::Activity(line) => {
            #[expect(clippy::manual_map, reason = "step code has no closures (programming-model.md, 10)")]
            let requester = match core.tasks.task(line.task) {
                Some(task) => Some(task.requester),
                None => None,
            };
            activity_update(line, requester)
        }
        tasks::Stored::DecidedProposal(row) => {
            let Some(decision) = crate::proposals::decision_record(record, at) else { return };
            Some(proposal_update(row, decision.at))
        }
        tasks::Stored::Result(_)
        | tasks::Stored::EndedByParty(_)
        | tasks::Stored::EndedByProject(_)
        | tasks::Stored::EndedDelegate(_)
        | tasks::Stored::Delegate(_)
        | tasks::Stored::Update(_)
        | tasks::Stored::Live(_)
        | tasks::Stored::History(_)
        | tasks::Stored::Milestone(_)
        | tasks::Stored::PersonProposal(_)
        | tasks::Stored::PlacedHold(_)
        | tasks::Stored::Stub(_)
        | tasks::Stored::Ledger(_)
        | tasks::Stored::Writer(_)
        | tasks::Stored::Pool(_)
        | tasks::Stored::ClosedLedger(_) => None,
    };
    if let Some(update) = update {
        out.push(Request::Write(Write::Save(Record::Tasks(tasks::Stored::Update(update)))));
    }
}

/// The reply update indexed from an activity line and its task's requester.
pub(crate) fn activity_update(line: &tasks::ActivityLine, requester: Option<tasks::Party>) -> Option<tasks::Update> {
    let addressed = match &line.line {
        tasks::Line::Turn { reply, .. } => match requester {
            Some(tasks::Party::Person(party)) => Some((party, *reply)),
            Some(tasks::Party::Task(_) | tasks::Party::Deployment { .. }) | None => None,
        },
        tasks::Line::Notified { recipient: tasks::Party::Person(party), word } => Some((*party, word.number)),
        tasks::Line::Notified { .. }
        | tasks::Line::Words { .. }
        | tasks::Line::Heard(_)
        | tasks::Line::RunBegan { .. }
        | tasks::Line::RunEnded { .. }
        | tasks::Line::Decided { .. }
        | tasks::Line::Kept { .. }
        | tasks::Line::Milestone { .. } => None,
    };
    let (party, message) = addressed?;
    Some(tasks::Update {
        party,
        at: line.at,
        task: line.task,
        object: tasks::UpdateObject::Reply {
            message: tasks::Newest::of(message),
            activity: tasks::Newest::of(line.position),
        },
    })
}

/// The proposal update indexed at its decision's time.
pub(crate) fn proposal_update(row: &tasks::PersonProposal, at: skein_lib::Wall) -> tasks::Update {
    tasks::Update {
        party: row.proposer,
        at,
        task: row.first(),
        object: tasks::UpdateObject::Proposal { proposal: tasks::Newest::of(row.number) },
    }
}
