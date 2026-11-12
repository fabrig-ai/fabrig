//! Physical keys of joined read rows (protocol/store/loads.md, 3).
use super::{Position, Row};
use crate::{CoreKey, CoreRecord, Key, Record, documents};
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;

/// Stable address supplied by the owning record, never by a client row number.
#[must_use]
pub fn row_key(row: &Row) -> Option<Position> {
    if let Some(position) = super::indexed::proposal_position(row) {
        return Some(position);
    }
    let key = match &row.value {
        Record::Tasks(value) => Some(Key::Tasks(value.key())),
        Record::Notes(notes::Record::Entry(value)) => {
            Some(Key::Notes(notes::Key::Line { scope: value.scope.clone(), name: value.name }))
        }
        Record::Core(CoreRecord::Turn(value)) => {
            Some(Key::Core(CoreKey::Turn { task: value.task, attempt: value.attempt, turn: value.turn }))
        }
        Record::Core(CoreRecord::EscalationDecision(value)) => {
            Some(Key::Core(CoreKey::EscalationDecision { task: value.task, revision: value.revision }))
        }
        Record::Core(CoreRecord::ProposalDecision(value)) => {
            Some(Key::Core(CoreKey::ProposalDecision { proposer: value.proposer, proposal: value.proposal }))
        }
        Record::Core(CoreRecord::Terminal(value)) => {
            Some(Key::Core(CoreKey::Terminal { task: value.task, attempt: value.attempt }))
        }
        Record::Core(
            CoreRecord::Settings(_)
            | CoreRecord::Deployment(_)
            | CoreRecord::RunProof(_)
            | CoreRecord::Call(_)
            | CoreRecord::Projection(_),
        )
        | Record::People(_)
        | Record::Notes(notes::Record::Line(_))
        | Record::Accounts(_) => None,
    }?;
    if let Record::Tasks(tasks::Stored::Update(_)) = &row.value {
        return Some(Position::Owner(key));
    }
    match row.inbox {
        Some(documents::Cursor::Inbox { task, object, at }) => Some(Position::Inbox { at, task, object, key }),
        Some(documents::Cursor::Number { .. }) => None,
        None => Some(Position::Owner(key)),
    }
}
