//! The single index serving a task query's ended group (protocol/store/loads.md, 3).
use crate::{
    Key,
    documents::{PhaseClass, TaskQuery},
};
use jig_domain_tasks as tasks;

/// The index selected by the query's keys, in priority order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EndedPart {
    ByParty { party: u64, label: Option<u32> },
    Delegates { requester: u64 },
    ByProject { project: u32, label: Option<u32> },
    None,
}
/// Choose one ended index; a live-only query has no ended part.
#[must_use]
pub fn ended_part(query: &TaskQuery) -> EndedPart {
    if let Some(phase) = query.phase
        && phase != PhaseClass::Ended
    {
        return EndedPart::None;
    }
    match (query.requester, query.parent) {
        (Some(tasks::Party::Person(party)), _) => EndedPart::ByParty { party, label: query.label },
        (Some(tasks::Party::Task(task)), _) | (None, Some(task)) => EndedPart::Delegates { requester: task },
        (Some(tasks::Party::Deployment { .. }), _) => EndedPart::None,
        (None, None) => match query.project {
            Some(project) => EndedPart::ByProject { project, label: query.label },
            None => EndedPart::None,
        },
    }
}
/// Validate an examined position against the chosen prefix.
#[must_use]
pub fn position(query: &TaskQuery, key: &Key) -> bool {
    match (ended_part(query), key) {
        (
            EndedPart::ByParty { party, label },
            Key::Tasks(tasks::Key::EndedByParty { party: owner, label: value, .. }),
        ) => party == *owner && (label.is_none() || label == Some(*value)),
        (EndedPart::Delegates { requester }, Key::Tasks(tasks::Key::EndedDelegate { requester: owner, .. })) => {
            requester == *owner
        }
        (
            EndedPart::ByProject { project, label },
            Key::Tasks(tasks::Key::EndedByProject { project: owner, label: value, .. }),
        ) => project == *owner && (label.is_none() || label == Some(*value)),
        _ => false,
    }
}
