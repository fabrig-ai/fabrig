//! Watch subjects and terminal reasons (protocol/client/README.md, 5).
use super::{documents, live, objects, pages};
use crate::{Limits, Undecodable};
use alloc::boxed::Box;
use jig_client_domain::Watchable;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::{VERSION, v19 as w};
use skein_lib::{List, Reader, Writer, length::len_u64};

pub fn subject(value: &c::Subject, limits: &w::Limits) -> Result<w::Subject, w::Problem> {
    Ok(match value {
        c::Subject::Settings => w::Subject::Settings,
        c::Subject::Me => w::Subject::Me,
        c::Subject::Task { task } => {
            w::Subject::Task(w::SubjectTask::new(limits, w::SubjectTaskParts { task: *task })?)
        }
        c::Subject::Tree { task, window } => {
            w::Subject::Tree(w::SubjectTree::new(limits, w::SubjectTreeParts { task: *task, window: *window })?)
        }
        c::Subject::Chips { tasks } => {
            w::Subject::Chips(w::SubjectChips::new(limits, w::SubjectChipsParts { tasks: tasks.clone() })?)
        }
        c::Subject::Inbox { window } => {
            w::Subject::Inbox(w::SubjectInbox::new(limits, w::SubjectInboxParts { window: *window })?)
        }
        c::Subject::Tasks { query, window } => w::Subject::Tasks(w::SubjectTasks::new(
            limits,
            w::SubjectTasksParts { query: super::queries::query(*query, limits)?, window: *window },
        )?),
        c::Subject::Run { task, turn, offset } => {
            let turn = match turn {
                Some(turn) => Some(w::ProvisionalTurnName::new(
                    limits,
                    w::ProvisionalTurnNameParts { attempt: turn.attempt, number: turn.number, retry: turn.retry },
                )?),
                None => None,
            };
            w::Subject::Run(w::SubjectRun::new(limits, w::SubjectRunParts { task: *task, turn, offset: *offset })?)
        }
        c::Subject::Deployment => w::Subject::Deployment,
        c::Subject::Accounts => w::Subject::Accounts,
        c::Subject::Terminal { terminal } => w::Subject::Terminal(w::SubjectTerminal::new(limits, w::SubjectTerminalParts { terminal: *terminal })?),
    })
}
pub fn owned(value: Watchable, limits: &w::Limits) -> Result<w::Owned, w::Problem> {
    let (owner, version, record) = match value {
        Watchable::Jig(value) => {
            let value = subject(&value, limits)?;
            let mut writer = Writer::new(skein_lib::length::usize_of(value.measure()));
            value.encode(&mut writer).expect("measured subject");
            (w::Owner::Jig, VERSION, writer.finish())
        }
        Watchable::Family { scope, window, record: value } => (
            w::Owner::Family(w::Family::new(limits, w::FamilyParts { number: value.family })?),
            value.version,
            super::envelope::record(scope, window, value, limits)?,
        ),
    };
    w::Owned::new(limits, w::OwnedParts { owner, version, record })
}
#[must_use]
pub fn refused(value: &w::WatchRefusal) -> c::WatchEnd {
    match value {
        w::WatchRefusal::Unknown => c::WatchEnd::Unknown,
        w::WatchRefusal::NotAllowed => c::WatchEnd::NotAllowed,
        w::WatchRefusal::Busy => c::WatchEnd::Busy,
        w::WatchRefusal::SignedOut => c::WatchEnd::SignedOut,
    }
}
#[must_use]
pub fn ended(value: &w::WatchEnd) -> c::WatchEnd {
    match value {
        w::WatchEnd::NotServed(_) => c::WatchEnd::NotServed,
        w::WatchEnd::Closed => c::WatchEnd::Closed,
        w::WatchEnd::Unknown => c::WatchEnd::Unknown,
        w::WatchEnd::NotAllowed => c::WatchEnd::NotAllowed,
        w::WatchEnd::Busy => c::WatchEnd::Busy,
        w::WatchEnd::SubjectEnded => c::WatchEnd::SubjectEnded,
        w::WatchEnd::SignedOut => c::WatchEnd::SignedOut,
        w::WatchEnd::Stopping => c::WatchEnd::Stopping,
    }
}

/// A delivery's discriminator and its whole record are admitted before decoding.
pub fn delivery(bytes: &[u8], limits: &Limits) -> Result<c::Delivery, Undecodable> {
    if len_u64(bytes.len()) > limits.client().document_bytes {
        return Err(Undecodable::TooLarge);
    }
    let value = match w::DeliveryValue::decode(limits.receiving(), &mut Reader::new(bytes)) {
        Ok(value) => value,
        Err(problem) => return Err(super::classify(problem)),
    };
    Ok(match value {
        w::DeliveryValue::Terminal(value) => {
            c::Delivery::Terminal(Box::new(match documents::terminal(value, limits) {
                Ok(value) => value,
                Err(problem) => return Err(super::classify(problem)),
            }))
        }
        w::DeliveryValue::Project(value) => c::Delivery::Project(Box::new(documents::project(value, limits)?)),
        w::DeliveryValue::Accounts(value) => {
            c::Delivery::Accounts(Box::new(match documents::accounts(value, limits) {
                Ok(value) => value,
                Err(problem) => return Err(super::classify(problem)),
            }))
        }
        w::DeliveryValue::Settings(value) => {
            c::Delivery::Settings(Box::new(match super::settings::restore_settings_document(&value, limits) {
                Ok(value) => value,
                Err(problem) => return Err(super::classify(problem)),
            }))
        }
        w::DeliveryValue::Me(value) => c::Delivery::Me(Box::new(documents::frame(value, limits)?)),
        w::DeliveryValue::Task(value) => c::Delivery::Task(Box::new(documents::task(*value, limits)?)),
        w::DeliveryValue::Proposal(value) => c::Delivery::Proposal(Box::new(objects::proposal(value, limits)?)),
        w::DeliveryValue::Tree(value) => c::Delivery::Tree(Box::new(tree(value, limits)?)),
        w::DeliveryValue::Inbox(value) => c::Delivery::Inbox(Box::new(inbox(value, limits)?)),
        w::DeliveryValue::Tasks(value) => c::Delivery::Tasks(Box::new(documents::tasks(value, limits)?)),
        w::DeliveryValue::RunState(value) => c::Delivery::RunState(Box::new(live::run_state(value, limits)?)),
        w::DeliveryValue::Deployment(value) => c::Delivery::Deployment(Box::new(documents::deployment(value, limits)?)),
        w::DeliveryValue::Chips(value) => {
            c::Delivery::Chips(match documents::chip_piece(value.rows(), limits, limits.client().rows) {
                Ok(value) => value,
                Err(problem) => return Err(super::classify(problem)),
            })
        }
    })
}
fn inbox(record: w::Inbox, limits: &Limits) -> Result<c::Inbox, Undecodable> {
    let needs = match pages::inbox_entry_piece(record.needs(), limits, limits.client().rows) {
        Ok(value) => value,
        Err(problem) => return Err(super::classify(problem)),
    };
    let updates = match pages::inbox_entry_piece(record.updates(), limits, limits.client().rows) {
        Ok(value) => value,
        Err(problem) => return Err(super::classify(problem)),
    };
    let selection = match record.selection() {
        Some(value) => Some(match pages::inbox_selection_piece(value, limits, limits.client().rows) {
            Ok(value) => value,
            Err(problem) => return Err(super::classify(problem)),
        }),
        None => None,
    };
    let next = match record.next() {
        Some(value) => Some(pages::inbox_cursor_fields(value)),
        None => None,
    };
    Ok(c::Inbox {
        party: record.party(),
        needs,
        updates,
        next,
        cached_needs: record.cached_needs(),
        cached_updates: documents::count_fields(record.cached_updates()),
        results_seen: record.results_seen(),
        updates_time_complete: record.updates_time_complete(),
        selection,
    })
}
fn tree(record: w::Tree, limits: &Limits) -> Result<c::Tree, Undecodable> {
    let root = match documents::chip_fields(record.root(), limits) {
        Ok(value) => value,
        Err(problem) => return Err(super::classify(problem)),
    };
    let rows = match tree_rows(record.rows(), limits) {
        Ok(value) => value,
        Err(problem) => return Err(super::classify(problem)),
    };
    let mut folded = List::with_capacity(record.folded().len());
    for value in record.folded() {
        folded
            .push(c::TreeFold {
                parent: value.parent(),
                phases: documents::phase_counts_fields(value.phases()),
                held_below: value.held_below(),
            })
            .expect("bounded folds");
    }
    let next = match record.next() {
        Some(value) => Some(match documents::cursor_fields(value, limits) {
            Ok(value) => value,
            Err(problem) => return Err(super::classify(problem)),
        }),
        None => None,
    };
    Ok(c::Tree { root, rows, folded, held_below: record.held_below(), next })
}
fn tree_rows(bytes: &[u8], limits: &Limits) -> Result<List<c::TreeRow>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(limits.client().rows);
    let mut sum = 0_u64;
    for _ in 0..=limits.client().rows {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::TreeRow::decode(limits.receiving(), &mut Reader::new(&item))?;
        let mut proposals = List::with_capacity(row.proposals().len());
        for key in row.proposals() {
            proposals.push(super::facts::proposal_key(key)).expect("bounded proposals");
        }
        let row = c::TreeRow {
            chip: documents::chip_fields(row.chip(), limits)?,
            parent: row.parent(),
            depth: row.depth(),
            proposals,
            spent: row.spent(),
            dependencies: documents::task_link_piece(row.dependencies(), limits, limits.client().rows)?,
            cut: row.cut(),
        };
        if rows.push(row).is_err() {
            return Err(super::too_large());
        }
    }
    Err(super::too_large())
}
