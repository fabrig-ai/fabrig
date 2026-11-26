//! Serving a point load from the engine's store port (jig/docs/design/protocol/store/README.md, 3).
use crate::{Get, Request, Row};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{Key, Record, engine::store};
use jig_domain_tasks as tasks;
use skein_lib::Token;

/// A proposal lookup first reads its live owner, then the named history entry.
#[derive(Debug)]
pub enum Pending {
    Record,
    ProposalOwner { task: u64, proposal: u64 },
}

/// The next get of the same load, or its terminal.
#[derive(Debug)]
pub enum Step {
    Get(Request),
    Answer(store::Answer),
}

#[must_use]
pub fn start(owner: Token, range: store::Range) -> Option<(Request, Pending)> {
    let (key, pending) = match range {
        store::Range::Record(key) => (key, Pending::Record),
        store::Range::Proposal { task, proposal } => {
            (Key::Tasks(tasks::Key::Live(task)), Pending::ProposalOwner { task, proposal })
        }
        store::Range::Read(_) | store::Range::Window { .. } => return None,
    };
    Some((get(owner, key), pending))
}

#[must_use]
pub fn got(owner: Token, pending: Pending, row: Option<Row>) -> Step {
    let row = match row {
        Some(Row::Jig(row)) => Some(row),
        None => None,
        Some(Row::Application { .. }) => return Step::Answer(store::Answer::Failed(store::Failure::Failed)),
    };
    match pending {
        Pending::Record => Step::Answer(records(row)),
        Pending::ProposalOwner { task, proposal } => {
            if let Some(Record::Tasks(tasks::Stored::Live(live))) = row
                && live.number == task
                && let Some(key) = crate::reads::proposal_key(&live, proposal)
            {
                return Step::Get(get(owner, key));
            }
            Step::Answer(records(None))
        }
    }
}

#[must_use]
pub fn records(row: Option<Record>) -> store::Answer {
    let records: Box<[Record]> = match row {
        Some(row) => Box::new([row]),
        None => Box::new([]),
    };
    store::Answer::Rows(store::Rows::Records { records, more: false })
}

fn get(owner: Token, key: Key) -> Request {
    Request::Get { owner, key: Get::Jig(key) }
}
