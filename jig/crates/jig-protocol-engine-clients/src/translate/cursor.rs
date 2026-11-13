//! Whole owner cursors, with keys carried by the store codec (jig/docs/design/protocol/client/README.md, 4).
use crate::cursor_v1::{self as c, CEILINGS as C};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{Key, documents, reads};
use jig_domain_notes as notes;
use jig_domain_people as people;
use skein_lib::{Reader, Wall, Writer};

pub fn encode(value: &reads::Cursor, limit: u32) -> Option<Box<[u8]>> {
    let cursor = match value.clone() {
        reads::Cursor::Live { scope, after, store_after } => c::Cursor::Live(
            c::LiveCursor::new(
                &C,
                c::LiveCursorParts {
                    scope: put_scope(scope)?,
                    after: put_document(after),
                    store_after: put_position(store_after)?,
                },
            )
            .ok()?,
        ),
        reads::Cursor::Store { scope, after } => c::Cursor::Store(
            c::StoreCursor::new(&C, c::StoreCursorParts { scope: put_scope(scope)?, after: put_position(after)? })
                .ok()?,
        ),
    };
    let owner = c::Owner::new(&C, c::OwnerParts { cursor }).ok()?;
    write(c::Form::Owner(owner), limit)
}
pub(crate) fn encode_tree(value: &documents::Cursor, task: u64, window: u32, limit: u32) -> Option<Box<[u8]>> {
    let tree = c::Tree::new(&C, c::TreeParts { task, window, after: put_document(*value) }).ok()?;
    write(c::Form::Tree(tree), limit)
}
fn write(form: c::Form, limit: u32) -> Option<Box<[u8]>> {
    let held = c::Held::new(&C, c::HeldParts { form }).ok()?;
    if held.measure() > limit {
        return None;
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(held.measure()));
    held.encode(&mut writer).ok()?;
    Some(writer.finish())
}
pub fn decode(bytes: &[u8]) -> Option<reads::Cursor> {
    if bytes.len() > skein_lib::length::usize_of(super::w::CEILINGS.cursor_value) {
        return None;
    }
    let held = c::Held::decode(&C, &mut Reader::new(bytes)).ok()?;
    let owner = match held.into_parts().form {
        c::Form::Owner(owner) => owner,
        c::Form::Tree(_) => return None,
    };
    Some(match owner.into_parts().cursor {
        c::Cursor::Live(value) => {
            let c::LiveCursorParts { scope, after, store_after } = value.into_parts();
            reads::Cursor::Live {
                scope: take_scope(scope)?,
                after: take_document(after),
                store_after: take_position(store_after)?,
            }
        }
        c::Cursor::Store(value) => {
            let c::StoreCursorParts { scope, after } = value.into_parts();
            reads::Cursor::Store { scope: take_scope(scope)?, after: take_position(after)? }
        }
    })
}
/// Decode one document continuation carried by a family window.
pub(crate) fn decode_window(bytes: &[u8]) -> Option<documents::Cursor> {
    Some(take_document(c::DocumentCursor::decode(&C, &mut Reader::new(bytes)).ok()?))
}
fn number(value: u64) -> c::Number {
    c::Number::new(&C, c::NumberParts { value }).expect("unbounded number")
}
fn project(value: u32) -> c::Project {
    c::Project::new(&C, c::ProjectParts { value }).expect("unbounded project")
}
fn put_scope(value: reads::Scope) -> Option<c::Scope> {
    Some(match value {
        reads::Scope::TaskText { .. }
        | reads::Scope::EndedTasks
        | reads::Scope::TaskRows { .. }
        | reads::Scope::SettingsHistory { .. } => return None,
        reads::Scope::Task { task } => c::Scope::Task(number(task)),
        reads::Scope::Proposal { proposer, proposal } => c::Scope::Proposal(put_proposal(proposer, proposal)),
        reads::Scope::Escalation { task } => c::Scope::Escalation(number(task)),
        reads::Scope::Question { task, message } => {
            c::Scope::Question(c::Question::new(&C, c::QuestionParts { task, message }).ok()?)
        }
        reads::Scope::PersonTask { task } => c::Scope::PersonTask(number(task)),
        reads::Scope::Result { task } => c::Scope::Result(number(task)),
        reads::Scope::Reply { task, activity } => {
            c::Scope::Reply(c::Reply::new(&C, c::ReplyParts { task, activity }).ok()?)
        }
        reads::Scope::Turn { task, attempt, turn } => {
            c::Scope::Turn(c::Turn::new(&C, c::TurnParts { task, attempt, number: turn }).ok()?)
        }
        reads::Scope::EndedDelegates { task } => c::Scope::EndedDelegates(number(task)),
        reads::Scope::History { task, subtree } => {
            c::Scope::History(c::History::new(&C, c::HistoryParts { task, subtree }).ok()?)
        }
        reads::Scope::Activity { task } => c::Scope::Activity(number(task)),
        reads::Scope::Inbox { group, filter } => {
            let filter = c::Filter::new(&C, c::FilterParts { project: filter.project, goal: filter.goal }).ok()?;
            c::Scope::Inbox(c::Inbox::new(&C, c::InboxParts { group: put_group(group), filter }).ok()?)
        }
        reads::Scope::Notes { project: value } => c::Scope::Notes(project(value)),
        reads::Scope::ScopedNotes { scope, description } => c::Scope::ScopedNotes(
            c::ScopedNotes::new(&C, c::ScopedNotesParts { scope: put_note_scope(scope)?, description }).ok()?,
        ),
        reads::Scope::Spend { project: value } => c::Scope::Spend(project(value)),
        reads::Scope::Tasks { query } => c::Scope::Tasks(put_query(query)?),
    })
}
fn take_scope(value: c::Scope) -> Option<reads::Scope> {
    Some(match value {
        c::Scope::Task(v) => reads::Scope::Task { task: v.value() },
        c::Scope::Proposal(v) => {
            reads::Scope::Proposal { proposer: take_proposer(v.proposer().clone()), proposal: v.number() }
        }
        c::Scope::Escalation(v) => reads::Scope::Escalation { task: v.value() },
        c::Scope::Question(v) => reads::Scope::Question { task: v.task(), message: v.message() },
        c::Scope::PersonTask(v) => reads::Scope::PersonTask { task: v.value() },
        c::Scope::Result(v) => reads::Scope::Result { task: v.value() },
        c::Scope::Reply(v) => reads::Scope::Reply { task: v.task(), activity: v.activity() },
        c::Scope::Turn(v) => reads::Scope::Turn { task: v.task(), attempt: v.attempt(), turn: v.number() },
        c::Scope::EndedDelegates(v) => reads::Scope::EndedDelegates { task: v.value() },
        c::Scope::History(v) => reads::Scope::History { task: v.task(), subtree: v.subtree() },
        c::Scope::Activity(v) => reads::Scope::Activity { task: v.value() },
        c::Scope::Inbox(v) => reads::Scope::Inbox {
            group: take_group(v.group().clone()),
            filter: reads::Filter { project: *v.filter().project(), goal: *v.filter().goal() },
        },
        c::Scope::Notes(v) => reads::Scope::Notes { project: v.value() },
        c::Scope::ScopedNotes(v) => {
            let c::ScopedNotesParts { scope, description } = v.into_parts();
            reads::Scope::ScopedNotes { scope: take_note_scope(scope), description }
        }
        c::Scope::Spend(v) => reads::Scope::Spend { project: v.value() },
        c::Scope::Tasks(v) => reads::Scope::Tasks { query: take_query(v)? },
    })
}
#[expect(clippy::option_option, reason = "outer absence is malformed; inner absence is an owner's absent field")]
fn put_position(value: Option<reads::Position>) -> Option<Option<c::Position>> {
    Some(match value {
        None => None,
        Some(reads::Position::Owner(key)) => Some(c::Position::Owner(put_key(&key)?)),
        Some(reads::Position::Inbox { at, task, object, key }) => Some(c::Position::Inbox(
            c::InboxPosition::new(
                &C,
                c::InboxPositionParts { at: wall(at), task, object: put_inbox(object), key: put_key(&key)? },
            )
            .ok()?,
        )),
    })
}
#[expect(clippy::option_option, reason = "outer absence is malformed; inner absence is an owner's absent field")]
fn take_position(value: Option<c::Position>) -> Option<Option<reads::Position>> {
    Some(match value {
        None => None,
        Some(c::Position::Owner(key)) => Some(reads::Position::Owner(jig_protocol_store::keys::decode(key.value())?)),
        Some(c::Position::Inbox(v)) => {
            let c::InboxPositionParts { at, task, object, key } = v.into_parts();
            Some(reads::Position::Inbox {
                at: take_wall(at),
                task,
                object: take_inbox(object),
                key: jig_protocol_store::keys::decode(key.value())?,
            })
        }
    })
}
fn put_key(value: &Key) -> Option<c::OwnerKey> {
    let value = jig_protocol_store::keys::encode(value, super::w::CEILINGS.cursor_value)?;
    c::OwnerKey::new(&C, c::OwnerKeyParts { value }).ok()
}
fn wall(value: Option<Wall>) -> Option<u64> {
    match value {
        None => None,
        Some(value) => Some(value.as_nanos()),
    }
}
fn take_wall(value: Option<u64>) -> Option<Wall> {
    match value {
        None => None,
        Some(value) => Some(Wall::from_nanos(value)),
    }
}
fn put_document(value: documents::Cursor) -> c::DocumentCursor {
    match value {
        documents::Cursor::Number { number: value } => c::DocumentCursor::Number(number(value)),
        documents::Cursor::Inbox { task, object, at } => c::DocumentCursor::Inbox(
            c::InboxCursor::new(&C, c::InboxCursorParts { task, object: put_inbox(object), at: wall(at) })
                .expect("unbounded document cursor"),
        ),
    }
}
fn take_document(value: c::DocumentCursor) -> documents::Cursor {
    match value {
        c::DocumentCursor::Number(v) => documents::Cursor::Number { number: v.value() },
        c::DocumentCursor::Inbox(v) => {
            let c::InboxCursorParts { task, object, at } = v.into_parts();
            documents::Cursor::Inbox { task, object: take_inbox(object), at: take_wall(at) }
        }
    }
}
fn put_proposal(proposer: people::Proposer, value: u64) -> c::Proposal {
    let proposer = match proposer {
        people::Proposer::Task(v) => c::Proposer::Task(number(v)),
        people::Proposer::Party(v) => c::Proposer::Party(number(v)),
    };
    c::Proposal::new(&C, c::ProposalParts { proposer, number: value }).expect("unbounded proposal")
}
fn take_proposer(value: c::Proposer) -> people::Proposer {
    match value {
        c::Proposer::Task(v) => people::Proposer::Task(v.value()),
        c::Proposer::Party(v) => people::Proposer::Party(v.value()),
    }
}
fn put_inbox(value: documents::InboxObject) -> c::InboxObject {
    match value {
        documents::InboxObject::Question { message } => c::InboxObject::Question(number(message)),
        documents::InboxObject::Proposal { proposer, number } => {
            c::InboxObject::Proposal(put_proposal(proposer, number))
        }
        documents::InboxObject::Escalation { revision } => c::InboxObject::Escalation(number(revision)),
        documents::InboxObject::PersonTask => c::InboxObject::PersonTask,
        documents::InboxObject::Reply { message } => c::InboxObject::Reply(number(message)),
        documents::InboxObject::Result { position } => c::InboxObject::Result(number(position)),
    }
}
fn take_inbox(value: c::InboxObject) -> documents::InboxObject {
    match value {
        c::InboxObject::Question(v) => documents::InboxObject::Question { message: v.value() },
        c::InboxObject::Proposal(v) => {
            documents::InboxObject::Proposal { proposer: take_proposer(v.proposer().clone()), number: v.number() }
        }
        c::InboxObject::Escalation(v) => documents::InboxObject::Escalation { revision: v.value() },
        c::InboxObject::PersonTask => documents::InboxObject::PersonTask,
        c::InboxObject::Reply(v) => documents::InboxObject::Reply { message: v.value() },
        c::InboxObject::Result(v) => documents::InboxObject::Result { position: v.value() },
    }
}
fn put_group(value: reads::Group) -> c::Group {
    match value {
        reads::Group::Needs => c::Group::Needs,
        reads::Group::Updates => c::Group::Updates,
        reads::Group::Proposals => c::Group::Proposals,
        reads::Group::Escalations => c::Group::Escalations,
        reads::Group::Questions => c::Group::Questions,
        reads::Group::PersonTasks => c::Group::PersonTasks,
        reads::Group::Results => c::Group::Results,
        reads::Group::Replies => c::Group::Replies,
    }
}
fn take_group(value: c::Group) -> reads::Group {
    match value {
        c::Group::Needs => reads::Group::Needs,
        c::Group::Updates => reads::Group::Updates,
        c::Group::Proposals => reads::Group::Proposals,
        c::Group::Escalations => reads::Group::Escalations,
        c::Group::Questions => reads::Group::Questions,
        c::Group::PersonTasks => reads::Group::PersonTasks,
        c::Group::Results => reads::Group::Results,
        c::Group::Replies => reads::Group::Replies,
    }
}
fn put_note_scope(value: notes::Scope) -> Option<c::NoteScope> {
    Some(match value {
        notes::Scope::Party { party } => {
            c::NoteScope::Party(c::NoteScopeParty::new(&C, c::NoteScopePartyParts { party }).ok()?)
        }
        notes::Scope::Deployment => c::NoteScope::Deployment,
        notes::Scope::Project { project: v } => c::NoteScope::Project(project(v)),
        notes::Scope::Goal { project, goal } => {
            c::NoteScope::Goal(c::Goal::new(&C, c::GoalParts { project, goal }).ok()?)
        }
        notes::Scope::Resources { project, connector, pattern } => {
            let last = match pattern.last {
                notes::Last::Exact(value) => c::Last::Exact(c::Blob::new(&C, c::BlobParts { value }).ok()?),
                notes::Last::Open(value) => c::Last::Open(c::Blob::new(&C, c::BlobParts { value }).ok()?),
            };
            let segments = skein_lib::List::from_boxed(pattern.segments);
            let pattern = c::Pattern::new(&C, c::PatternParts { segments, last }).ok()?;
            c::NoteScope::Resources(c::Resources::new(&C, c::ResourcesParts { project, connector, pattern }).ok()?)
        }
    })
}
fn take_note_scope(value: c::NoteScope) -> notes::Scope {
    match value {
        c::NoteScope::Party(value) => notes::Scope::Party { party: value.party() },
        c::NoteScope::Deployment => notes::Scope::Deployment,
        c::NoteScope::Project(v) => notes::Scope::Project { project: v.value() },
        c::NoteScope::Goal(v) => notes::Scope::Goal { project: v.project(), goal: v.goal() },
        c::NoteScope::Resources(v) => {
            let c::ResourcesParts { project, connector, pattern } = v.into_parts();
            let c::PatternParts { segments, last } = pattern.into_parts();
            let last = match last {
                c::Last::Exact(v) => notes::Last::Exact(v.into_parts().value),
                c::Last::Open(v) => notes::Last::Open(v.into_parts().value),
            };
            notes::Scope::Resources {
                project,
                connector,
                pattern: notes::Pattern { segments: segments.into_boxed(), last },
            }
        }
    }
}
fn put_query(query: documents::TaskQuery) -> Option<c::Query> {
    use documents::{ExecutorKind as E, PhaseClass as P, QueryOrder as O};
    let requester = match query.requester {
        None => c::Requester::None,
        Some(jig_domain_tasks::Party::Person(v)) => c::Requester::Person(number(v)),
        Some(jig_domain_tasks::Party::Task(v)) => c::Requester::Task(number(v)),
        Some(jig_domain_tasks::Party::Deployment { .. }) => return None,
    };
    let phase = match query.phase {
        None => None,
        Some(v) => Some(match v {
            P::Working => 0,
            P::Waiting => 1,
            P::Held => 2,
            P::Closing => 3,
            P::Ended => 4,
        }),
    };
    let executor = match query.executor {
        None => None,
        Some(v) => Some(match v {
            E::Agent => 0,
            E::Procedure => 1,
            E::Person => 2,
            E::Job => 3,
        }),
    };
    let tracked = match query.tracked {
        None => None,
        Some(v) => Some(u32::from(v)),
    };
    let order = match query.order {
        O::Number => 0,
        O::Changed => 1,
        O::Activity => 2,
        O::Priority => 3,
        O::Ended => 4,
    };
    c::Query::new(
        &C,
        c::QueryParts {
            project: query.project,
            requester,
            root: query.root,
            parent: query.parent,
            label: query.label,
            phase,
            executor,
            tracked,
            order,
        },
    )
    .ok()
}
fn take_query(value: c::Query) -> Option<documents::TaskQuery> {
    use documents::{ExecutorKind as E, PhaseClass as P, QueryOrder as O};
    let c::QueryParts { project, requester, root, parent, label, phase, executor, tracked, order } = value.into_parts();
    let requester = match requester {
        c::Requester::None => None,
        c::Requester::Person(v) => Some(jig_domain_tasks::Party::Person(v.value())),
        c::Requester::Task(v) => Some(jig_domain_tasks::Party::Task(v.value())),
    };
    let phase = match phase {
        None => None,
        Some(0) => Some(P::Working),
        Some(1) => Some(P::Waiting),
        Some(2) => Some(P::Held),
        Some(3) => Some(P::Closing),
        Some(4) => Some(P::Ended),
        Some(_) => return None,
    };
    let executor = match executor {
        None => None,
        Some(0) => Some(E::Agent),
        Some(1) => Some(E::Procedure),
        Some(2) => Some(E::Person),
        Some(3) => Some(E::Job),
        Some(_) => return None,
    };
    let tracked = match tracked {
        None => None,
        Some(0) => Some(false),
        Some(1) => Some(true),
        Some(_) => return None,
    };
    let order = match order {
        0 => O::Number,
        1 => O::Changed,
        2 => O::Activity,
        3 => O::Priority,
        4 => O::Ended,
        _ => return None,
    };
    Some(documents::TaskQuery { project, requester, root, parent, label, phase, executor, tracked, order })
}
