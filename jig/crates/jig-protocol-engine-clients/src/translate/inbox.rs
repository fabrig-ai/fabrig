//! Whole live document facts (protocol/client/wire.md, 1, and protocol/client/README.md, 5).
use super::{live, pieces, task, w};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};
pub fn object(value: d::InboxObject, limits: &w::Limits) -> Result<w::InboxObject, w::Problem> {
    Ok(match value {
        d::InboxObject::Question { message } => {
            w::InboxObject::Question(w::InboxObjectQuestion::new(limits, w::InboxObjectQuestionParts { message })?)
        }
        d::InboxObject::Proposal { proposer, number } => w::InboxObject::Proposal(w::InboxObjectProposal::new(
            limits,
            w::InboxObjectProposalParts {
                proposer: match proposer {
                    jig_domain_people::Proposer::Task(value) => {
                        w::Proposer::Task(w::ProposerTask::new(limits, w::ProposerTaskParts { value })?)
                    }
                    jig_domain_people::Proposer::Party(value) => {
                        w::Proposer::Party(w::ProposerParty::new(limits, w::ProposerPartyParts { value })?)
                    }
                },
                number,
            },
        )?),
        d::InboxObject::Escalation { revision } => w::InboxObject::Escalation(w::InboxObjectEscalation::new(
            limits,
            w::InboxObjectEscalationParts { revision },
        )?),
        d::InboxObject::PersonTask => w::InboxObject::PersonTask,
        d::InboxObject::Reply { message } => {
            w::InboxObject::Turn(w::InboxObjectTurn::new(limits, w::InboxObjectTurnParts { message })?)
        }
        d::InboxObject::Result { position } => {
            w::InboxObject::Result(w::InboxObjectResult::new(limits, w::InboxObjectResultParts { position })?)
        }
    })
}
pub fn cursor(value: d::Cursor, limits: &w::Limits) -> Result<w::InboxCursor, w::Problem> {
    Ok(match value {
        d::Cursor::Number { number } => {
            w::InboxCursor::Number(w::InboxCursorNumber::new(limits, w::InboxCursorNumberParts { number })?)
        }
        d::Cursor::Inbox { task, object: value, at } => w::InboxCursor::Inbox(w::InboxCursorInbox::new(
            limits,
            w::InboxCursorInboxParts {
                task,
                object: object(value, limits)?,
                at: match at {
                    Some(item) => Some(item.as_nanos()),
                    None => None,
                },
            },
        )?),
    })
}
pub fn entry(value: &d::InboxEntry, limits: &w::Limits) -> Result<w::InboxEntry, w::Problem> {
    w::InboxEntry::new(
        limits,
        w::InboxEntryParts {
            projects: value.projects.clone(),
            reply: match &value.reply {
                Some(item) => Some(super::queries::line(item, limits)?),
                None => None,
            },
            task: task::link(&value.task, limits)?,
            project: value.project,
            object: object(value.object, limits)?,
            activity: value.activity,
            at: match value.at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            unread: value.unread,
            cursor: cursor(value.cursor, limits)?,
        },
    )
}
pub fn selection(
    value: Option<&List<d::InboxSelection>>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Option<Box<[u8]>>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    let mut rows = List::with_capacity(value.len());
    for row in value {
        let mut goals = List::with_capacity(row.goals.len());
        for number in &row.goals {
            goals.push(*number).expect("supplied roots fit");
        }
        rows.push(w::InboxSelection::new(
            limits,
            w::InboxSelectionParts { cursor: cursor(row.cursor, limits)?, goals },
        )?)
        .expect("supplied selections fit");
    }
    Ok(Some(selection_pieces(&rows, bound, path)?))
}
pub fn document(value: &d::Inbox, limits: &w::Limits) -> Result<w::Inbox, w::Problem> {
    let mut needs = List::with_capacity(value.needs.len());
    for row in &value.needs {
        needs.push(entry(row, limits)?).expect("supplied rows fit");
    }
    let needs = entry_pieces(&needs, limits.inbox_needs, w::Path::InboxNeeds)?;
    let mut updates = List::with_capacity(value.updates.len());
    for row in &value.updates {
        updates.push(entry(row, limits)?).expect("supplied rows fit");
    }
    let updates = entry_pieces(&updates, limits.inbox_updates, w::Path::InboxUpdates)?;

    w::Inbox::new(
        limits,
        w::InboxParts {
            party: value.party,
            needs,
            updates,
            next: match value.next {
                Some(item) => Some(cursor(item, limits)?),
                None => None,
            },
            cached_needs: value.cached_needs,
            cached_updates: live::count(value.cached_updates, limits)?,
            results_seen: value.results_seen,
            updates_time_complete: value.updates_time_complete,
            selection: selection(
                value.selection.as_ref(),
                limits,
                limits.inbox_selection_some,
                w::Path::InboxSelection,
            )?,
        },
    )
}
fn entry_pieces(rows: &List<w::InboxEntry>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
fn selection_pieces(rows: &List<w::InboxSelection>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure()));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}
