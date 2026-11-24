//! Authenticated reads and their one durable answer (protocol/client/README.md, 4).
use super::{cursor, objects, pages, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::reads;
use jig_domain_notes as notes;
use jig_domain_people as people;
use skein_lib::{Reader, Writer};
pub(crate) fn read(value: w::Read) -> Option<reads::Read> {
    Some(match value {
        w::Read::Settings => reads::Read::Settings,
        w::Read::Setting(value) => read_setting(value),
        w::Read::Revision(value) => reads::Read::Revision { number: value.number },
        w::Read::Revisions(value) => reads::Read::Revisions { before: value.before },
        w::Read::Tasks(value) => reads::Read::Tasks {
            query: super::queries::query(value.query)?,
            live: decode_cursor(value.live)?,
            ended: decode_cursor(value.ended)?,
        },
        w::Read::Turn(value) => reads::Read::Turn { task: value.task, attempt: value.attempt, turn: value.turn },
        w::Read::TaskText(value) => read_task_text(value),
        w::Read::ActivityBeside(value) => reads::Read::ActivityBeside {
            task: value.task,
            position: value.position,
            side: mapped::activity_side_to_domain(value.side),
        },
        w::Read::Me => reads::Read::Frame,
        w::Read::Task(value) => reads::Read::Task { task: value.task },
        w::Read::Proposal(value) => reads::Read::Proposal {
            proposer: mapped::proposer_to_domain(value.proposal.proposer),
            proposal: value.proposal.number,
        },
        w::Read::Escalation(value) => reads::Read::Escalation { task: value.task },
        w::Read::Question(value) => reads::Read::Question { task: value.task, message: value.message },
        w::Read::PersonTask(value) => reads::Read::PersonTask { task: value.task },
        w::Read::Result(value) => reads::Read::Result { task: value.task },
        w::Read::Ended(value) => read_ended(value)?,
        w::Read::History(value) => read_history(value)?,
        w::Read::Activity(value) => read_activity(value)?,
        w::Read::Inbox(value) => read_inbox(value)?,
        w::Read::Spend(value) => {
            let parts = value;
            let from = match parts.after {
                Some(value) => Some(cursor::decode(value.value.as_ref())?),
                None => None,
            };
            reads::Read::Spend { project: parts.project, from }
        }
        w::Read::Project(value) => reads::Read::Project { project: value.project },
        w::Read::Notes(value) => read_notes(value)?,
        w::Read::Deployment => reads::Read::Deployment,
    })
}
fn group(parent: w::InboxReadGroup, kind: Option<w::InboxGroup>) -> Option<reads::Group> {
    let (expected, group) = match kind {
        None => {
            return Some(match parent {
                w::InboxReadGroup::Needs => reads::Group::Needs,
                w::InboxReadGroup::Updates => reads::Group::Updates,
            });
        }
        Some(w::InboxGroup::Proposals) => (w::InboxReadGroup::Needs, reads::Group::Proposals),
        Some(w::InboxGroup::Escalations) => (w::InboxReadGroup::Needs, reads::Group::Escalations),
        Some(w::InboxGroup::Questions) => (w::InboxReadGroup::Needs, reads::Group::Questions),
        Some(w::InboxGroup::PersonTasks) => (w::InboxReadGroup::Needs, reads::Group::PersonTasks),
        Some(w::InboxGroup::Results) => (w::InboxReadGroup::Updates, reads::Group::Results),
        Some(w::InboxGroup::Replies) => (w::InboxReadGroup::Updates, reads::Group::Replies),
    };
    if parent != expected {
        return None;
    }
    Some(group)
}
fn note_scope(value: w::NoteScope) -> Option<notes::Scope> {
    Some(match value {
        w::NoteScope::Tasks(_) => return None,
        w::NoteScope::Party(value) => notes::Scope::Party { party: value.party },
        w::NoteScope::Deployment => notes::Scope::Deployment,
        w::NoteScope::Project(value) => notes::Scope::Project { project: value.value },
        w::NoteScope::Task(value) => notes::Scope::Goal { project: value.project, goal: value.task },
        w::NoteScope::Resource(value) => {
            let parts = value;
            let pattern = parts.pattern;
            let last = match pattern.last {
                w::PatternEnd::Exact(value) => notes::Last::Exact(value.value),
                w::PatternEnd::Open(value) => notes::Last::Open(value.value),
            };
            notes::Scope::Resources {
                project: parts.project,
                connector: parts.connector,
                pattern: notes::Pattern { segments: pattern.segments.into_boxed(), last },
            }
        }
    })
}
pub(crate) fn none(why: reads::Why) -> w::ReadAnswer {
    let value = match why {
        reads::Why::TooLarge { bytes } => w::ReadNone::TooLarge(w::TurnTooLarge { bytes }),
        reads::Why::TooLinked { joins } => w::ReadNone::TooLinked(w::ReadTooLinked { joins }),
        reads::Why::Malformed => w::ReadNone::Malformed,
        reads::Why::Unknown => w::ReadNone::Unknown,
        reads::Why::Busy => w::ReadNone::Busy,
        reads::Why::Gone => w::ReadNone::Gone,
        reads::Why::SignedOut => w::ReadNone::SignedOut,
    };
    w::ReadAnswer::None(w::ReadAnswerNone { value })
}
pub(crate) fn answer(
    read: &reads::Read,
    answer: &reads::Answer,
    models: &[crate::Model],
    private: &crate::Private,
    settings: &crate::Settings,
    limits: &w::Limits,
) -> Result<w::ReadAnswer, w::Problem> {
    let value = match answer {
        reads::Answer::None(why) => return Ok(none(*why)),
        reads::Answer::Setting(value) => {
            w::ReadValue::Setting(super::settings::setting_document(value, models, limits)?)
        }
        reads::Answer::Revision(value) => {
            w::ReadValue::Revision(w::ReadRevisionAnswer { value: super::settings::revision(value, models, limits)? })
        }
        reads::Answer::Revisions(value) => w::ReadValue::Revisions(super::settings::revisions(value)),
        reads::Answer::TaskText(value) => w::ReadValue::TaskText(task_text(value)),
        reads::Answer::Tasks(page) => {
            let reads::Read::Tasks { query, .. } = read else {
                return Err(super::problem(skein_codec::Reason::Tag));
            };
            let slots = super::parts::plan(&query.parts, settings);
            pages::tasks(page, query, &slots, limits)?
        }
        reads::Answer::Page(page) => pages::page(models, read, page, private, limits)?,
        reads::Answer::Document(_) | reads::Answer::Ended(_) => objects::document(answer, models, private, limits)?,
    };
    let mut out = Writer::new(skein_lib::length::usize_of(value.measure(limits)?));
    value.write(limits, &mut out).expect("measured value");
    Ok(owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish()))
}
pub(crate) fn owned(owner: w::Owner, version: u32, record: Box<[u8]>) -> w::ReadAnswer {
    w::ReadAnswer::Value(w::Owned { owner, version, record })
}
pub(crate) fn decode(bytes: &[u8], limits: &w::Limits) -> Option<reads::Read> {
    let mut reader = Reader::new(bytes);
    let value = w::Read::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    read(value)
}

#[expect(clippy::option_option, reason = "outer absence is malformed, inner absence is the initial cursor")]
fn decode_cursor(value: Option<w::Cursor>) -> Option<Option<reads::Cursor>> {
    match value {
        Some(value) => Some(Some(cursor::decode(value.value.as_ref())?)),
        None => Some(None),
    }
}

fn task_text(value: &reads::TaskText) -> w::TaskText {
    w::TaskText {
        task: value.task,
        field: match value.field {
            reads::TaskTextField::Specification => w::TaskTextField::Specification,
            reads::TaskTextField::Instructions => w::TaskTextField::Instructions,
            reads::TaskTextField::Result => w::TaskTextField::Result,
        },
        from: value.from,
        total: value.total,
        next: value.next,
        words: skein_lib::bytes::copy_of(&value.bytes),
    }
}

fn read_setting(value: w::ReadSetting) -> reads::Read {
    {
        let mut proposal = None;
        if let Some(key) = &value.proposal {
            proposal = Some((
                match &key.proposer {
                    w::Proposer::Party(party) => people::Proposer::Party(party.value),
                    w::Proposer::Task(task) => people::Proposer::Task(task.value),
                },
                key.number,
            ));
        }
        reads::Read::Setting { setting: super::settings::setting(&value.setting), revision: value.revision, proposal }
    }
}

fn read_task_text(value: w::ReadTaskText) -> reads::Read {
    reads::Read::TaskText {
        task: value.task,
        from: value.from,
        field: match &value.field {
            w::TaskTextField::Specification => reads::TaskTextField::Specification,
            w::TaskTextField::Instructions => reads::TaskTextField::Instructions,
            w::TaskTextField::Result => reads::TaskTextField::Result,
        },
    }
}

fn read_ended(value: w::ReadEnded) -> Option<reads::Read> {
    Some({
        let parts = value;
        let from = match parts.after {
            Some(value) => Some(cursor::decode(value.value.as_ref())?),
            None => None,
        };
        reads::Read::EndedDelegates { task: parts.task, from }
    })
}

fn read_history(value: w::ReadHistory) -> Option<reads::Read> {
    Some({
        let parts = value;
        let from = match parts.after {
            Some(value) => Some(cursor::decode(value.value.as_ref())?),
            None => None,
        };
        reads::Read::History { task: parts.task, subtree: parts.subtree, from }
    })
}

fn read_activity(value: w::ReadActivity) -> Option<reads::Read> {
    Some({
        let parts = value;
        let before = match parts.before {
            Some(value) => Some(cursor::decode(value.value.as_ref())?),
            None => None,
        };
        reads::Read::Activity { task: parts.task, before }
    })
}

fn read_inbox(value: w::ReadInbox) -> Option<reads::Read> {
    Some({
        let parts = value;
        let group = group(parts.group, parts.kind)?;
        let from = match parts.after {
            Some(value) => Some(cursor::decode(value.value.as_ref())?),
            None => None,
        };
        reads::Read::Inbox { group, filter: reads::Filter { project: parts.project, goal: parts.goal }, from }
    })
}

fn read_notes(value: w::ReadNotes) -> Option<reads::Read> {
    Some({
        let parts = value;
        let from = match parts.after {
            Some(value) => Some(cursor::decode(value.value.as_ref())?),
            None => None,
        };
        match parts.scope {
            w::NoteScope::Tasks(value) => {
                if parts.description.is_some() {
                    return None;
                }
                reads::Read::Notes { project: value.project, from }
            }
            scope @ (w::NoteScope::Deployment
            | w::NoteScope::Project(_)
            | w::NoteScope::Resource(_)
            | w::NoteScope::Task(_)
            | w::NoteScope::Party(_)) => {
                reads::Read::ScopedNotes { scope: note_scope(scope)?, description: parts.description, from }
            }
        }
    })
}
