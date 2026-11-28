//! Held engine cursors, with owner keys carried by the store codec (jig/docs/design/protocol/client/README.md, 4).
use crate::cursor_v2::{self as c, CEILINGS as C};
use crate::generated::cursor as map;
use alloc::boxed::Box;
use c::Record as _;
use documents::{ExecutorKind, PhaseClass, QueryOrder};
use jig_domain_core_vocabulary::{Key, documents, reads};
use jig_domain_tasks::Party;
use skein_codec::Reason;
use skein_lib::{Reader, Writer};

/// The cursor family version checked at the client boundary.
pub const CURSOR_VERSION: u16 = 2;

pub fn encode(value: &reads::Cursor, limit: u32) -> Option<Box<[u8]>> {
    let cursor = map::cursor_to_record(value).ok()?;
    let owner = c::Owner { cursor };
    write(c::Form::Owner(owner), limit)
}
pub(crate) fn encode_tree(value: &documents::Cursor, task: u64, window: u32, limit: u32) -> Option<Box<[u8]>> {
    let tree = c::Tree { task, window, after: map::document_cursor_to_record(value) };
    write(c::Form::Tree(tree), limit)
}
/// Encode an application's document position for a read or watch window's `after`.
/// Returns none if the held cursor does not fit `limit` bytes.
#[must_use]
pub fn encode_window(after: &documents::Cursor, limit: u32) -> Option<Box<[u8]>> {
    let window = c::Window { after: map::document_cursor_to_record(after) };
    write(c::Form::Window(window), limit)
}
fn write(form: c::Form, limit: u32) -> Option<Box<[u8]>> {
    let held = c::Held { layout: c::LAYOUT, form };
    let measured = held.measure(&C).ok()?;
    if measured > limit {
        return None;
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(measured));
    held.write(&C, &mut writer).ok()?;
    Some(writer.finish())
}
pub fn decode(bytes: &[u8]) -> Option<reads::Cursor> {
    match read_form(bytes)? {
        c::Form::Owner(owner) => map::cursor_to_domain(owner.cursor).ok(),
        c::Form::Tree(_) | c::Form::Window(_) => None,
    }
}
/// Decode one document continuation carried by a family window.
pub(crate) fn decode_window(bytes: &[u8]) -> Option<documents::Cursor> {
    match read_form(bytes)? {
        c::Form::Window(window) => Some(map::document_cursor_to_domain(window.after)),
        c::Form::Owner(_) | c::Form::Tree(_) => None,
    }
}
fn read_form(bytes: &[u8]) -> Option<c::Form> {
    if bytes.len() > skein_lib::length::usize_of(super::w::CEILINGS.cursor_value) {
        return None;
    }
    // Judge the fixed prefix before decoding the layout-specific form.
    let mut prefix = Reader::new(bytes);
    if prefix.u16()? != CURSOR_VERSION || prefix.bytes(32)? != c::LAYOUT {
        return None;
    }
    let held = c::Held::decode(&C, &mut Reader::new(bytes)).ok()?;
    Some(held.form)
}

pub(crate) fn scope_to_record(value: &reads::Scope) -> Result<c::Scope, c::Problem> {
    Ok(match value {
        reads::Scope::TaskText { .. }
        | reads::Scope::EndedTasks
        | reads::Scope::TaskSummaries
        | reads::Scope::Retention { .. }
        | reads::Scope::ErasureKeys { .. }
        | reads::Scope::ProposalRemains { .. }
        | reads::Scope::TaskOwnRows { .. }
        | reads::Scope::TaskRows { .. }
        | reads::Scope::SettingsHistory { .. } => return Err(c::paths::SCOPE_TAG.problem(Reason::Tag)),
        reads::Scope::Task { task } => c::Scope::Task(c::Number { value: *task }),
        reads::Scope::Proposal { proposer, proposal } => {
            c::Scope::Proposal(c::Proposal { proposer: map::proposer_to_record(proposer), number: *proposal })
        }
        reads::Scope::Escalation { task } => c::Scope::Escalation(c::Number { value: *task }),
        reads::Scope::Question { task, message } => c::Scope::Question(c::Question { task: *task, message: *message }),
        reads::Scope::PersonTask { task } => c::Scope::PersonTask(c::Number { value: *task }),
        reads::Scope::Result { task } => c::Scope::Result(c::Number { value: *task }),
        reads::Scope::Reply { task, activity } => c::Scope::Reply(c::Reply { task: *task, activity: *activity }),
        reads::Scope::Turn { task, attempt, turn } => {
            c::Scope::Turn(c::Turn { task: *task, attempt: *attempt, number: *turn })
        }
        reads::Scope::EndedDelegates { task } => c::Scope::EndedDelegates(c::Number { value: *task }),
        reads::Scope::History { task, subtree } => c::Scope::History(c::History { task: *task, subtree: *subtree }),
        reads::Scope::Activity { task } => c::Scope::Activity(c::Number { value: *task }),
        reads::Scope::Inbox { group, filter } => {
            c::Scope::Inbox(c::Inbox { group: map::group_to_record(group), filter: map::filter_to_record(filter) })
        }
        reads::Scope::Notes { project } => c::Scope::Notes(c::Project { value: *project }),
        reads::Scope::ScopedNotes { scope, description } => c::Scope::ScopedNotes(c::ScopedNotes {
            scope: map::note_scope_to_record(scope)?,
            description: description.clone(),
        }),
        reads::Scope::Spend { project } => c::Scope::Spend(c::Project { value: *project }),
        reads::Scope::Tasks { query } => c::Scope::Tasks(map::query_to_record(query)?),
    })
}
pub(crate) fn scope_to_domain(value: c::Scope) -> Result<reads::Scope, c::Problem> {
    Ok(match value {
        c::Scope::Task(v) => reads::Scope::Task { task: v.value },
        c::Scope::Proposal(v) => {
            reads::Scope::Proposal { proposer: map::proposer_to_domain(v.proposer), proposal: v.number }
        }
        c::Scope::Escalation(v) => reads::Scope::Escalation { task: v.value },
        c::Scope::Question(v) => reads::Scope::Question { task: v.task, message: v.message },
        c::Scope::PersonTask(v) => reads::Scope::PersonTask { task: v.value },
        c::Scope::Result(v) => reads::Scope::Result { task: v.value },
        c::Scope::Reply(v) => reads::Scope::Reply { task: v.task, activity: v.activity },
        c::Scope::Turn(v) => reads::Scope::Turn { task: v.task, attempt: v.attempt, turn: v.number },
        c::Scope::EndedDelegates(v) => reads::Scope::EndedDelegates { task: v.value },
        c::Scope::History(v) => reads::Scope::History { task: v.task, subtree: v.subtree },
        c::Scope::Activity(v) => reads::Scope::Activity { task: v.value },
        c::Scope::Inbox(v) => {
            reads::Scope::Inbox { group: map::group_to_domain(v.group), filter: map::filter_to_domain(v.filter) }
        }
        c::Scope::Notes(v) => reads::Scope::Notes { project: v.value },
        c::Scope::ScopedNotes(v) => {
            let c::ScopedNotes { scope, description } = v;
            reads::Scope::ScopedNotes { scope: map::note_scope_to_domain(scope), description }
        }
        c::Scope::Spend(v) => reads::Scope::Spend { project: v.value },
        c::Scope::Tasks(v) => reads::Scope::Tasks { query: map::query_to_domain(v)? },
    })
}
pub(crate) fn owner_key_to_record(value: &Key) -> Result<c::OwnerKey, c::Problem> {
    let value = jig_protocol_store::keys::encode(value, super::w::CEILINGS.cursor_value)
        .ok_or(c::paths::OWNER_KEY_VALUE.problem(Reason::Bound))?;
    Ok(c::OwnerKey { value })
}
pub(crate) fn owner_key_to_domain(value: c::OwnerKey) -> Result<Key, c::Problem> {
    jig_protocol_store::keys::decode(&value.value).ok_or(c::paths::OWNER_KEY_VALUE.problem(Reason::Malformed))
}
#[expect(clippy::ref_option, reason = "the by-hand maps signature borrows the entire requester field")]
pub(crate) fn requester_to_record(value: &Option<Party>) -> Result<c::Requester, c::Problem> {
    Ok(match value {
        None => c::Requester::None,
        Some(Party::Person(v)) => c::Requester::Person(c::Number { value: *v }),
        Some(Party::Task(v)) => c::Requester::Task(c::Number { value: *v }),
        Some(Party::Deployment { .. }) => return Err(c::paths::REQUESTER_TAG.problem(Reason::Tag)),
    })
}
pub(crate) fn requester_to_domain(value: c::Requester) -> Option<Party> {
    match value {
        c::Requester::None => None,
        c::Requester::Person(v) => Some(Party::Person(v.value)),
        c::Requester::Task(v) => Some(Party::Task(v.value)),
    }
}
#[expect(clippy::trivially_copy_pass_by_ref, reason = "maps converters borrow domain values uniformly")]
pub(crate) fn phase_to_record(value: &PhaseClass) -> u32 {
    match value {
        PhaseClass::Working => 0,
        PhaseClass::Waiting => 1,
        PhaseClass::Held => 2,
        PhaseClass::Closing => 3,
        PhaseClass::Ended => 4,
    }
}
pub(crate) fn phase_to_domain(value: u32) -> Result<PhaseClass, Reason> {
    match value {
        0 => Ok(PhaseClass::Working),
        1 => Ok(PhaseClass::Waiting),
        2 => Ok(PhaseClass::Held),
        3 => Ok(PhaseClass::Closing),
        4 => Ok(PhaseClass::Ended),
        _ => Err(Reason::Tag),
    }
}
#[expect(clippy::trivially_copy_pass_by_ref, reason = "maps converters borrow domain values uniformly")]
pub(crate) fn executor_to_record(value: &ExecutorKind) -> u32 {
    match value {
        ExecutorKind::Agent => 0,
        ExecutorKind::Procedure => 1,
        ExecutorKind::Person => 2,
        ExecutorKind::Job => 3,
    }
}
pub(crate) fn executor_to_domain(value: u32) -> Result<ExecutorKind, Reason> {
    match value {
        0 => Ok(ExecutorKind::Agent),
        1 => Ok(ExecutorKind::Procedure),
        2 => Ok(ExecutorKind::Person),
        3 => Ok(ExecutorKind::Job),
        _ => Err(Reason::Tag),
    }
}
#[expect(clippy::trivially_copy_pass_by_ref, reason = "maps converters borrow domain values uniformly")]
pub(crate) fn tracked_to_record(value: &bool) -> u32 {
    u32::from(*value)
}
pub(crate) fn tracked_to_domain(value: u32) -> Result<bool, Reason> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Reason::Bool),
    }
}
#[expect(clippy::trivially_copy_pass_by_ref, reason = "maps converters borrow domain values uniformly")]
pub(crate) fn order_to_record(value: &QueryOrder) -> u32 {
    match value {
        QueryOrder::Number => 0,
        QueryOrder::Changed => 1,
        QueryOrder::Activity => 2,
        QueryOrder::Priority => 3,
        QueryOrder::Ended => 4,
    }
}
pub(crate) fn order_to_domain(value: u32) -> Result<QueryOrder, Reason> {
    match value {
        0 => Ok(QueryOrder::Number),
        1 => Ok(QueryOrder::Changed),
        2 => Ok(QueryOrder::Activity),
        3 => Ok(QueryOrder::Priority),
        4 => Ok(QueryOrder::Ended),
        _ => Err(Reason::Tag),
    }
}

#[cfg(test)]
mod tests;
