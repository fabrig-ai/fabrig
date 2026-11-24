//! Bounded owner page pieces (protocol/client/wire.md, 1).
use super::super::{owner, pieces, task, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, Writer};

fn charge(size: u32, measure: u32, bound: u32, path: w::Path) -> Result<u32, w::Problem> {
    let invalid = w::Problem { path, reason: skein_codec::Reason::Bound };
    let size = size.checked_add(4).ok_or(invalid)?.checked_add(measure).ok_or(invalid)?;
    if size > bound {
        return Err(invalid);
    }
    Ok(size)
}
pub(super) fn activity_pieces(
    rows: &List<w::DecodedActivityLine>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(limits)?, bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn history_pieces(
    rows: &List<w::HistoryRow>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(limits)?, bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn inbox_pieces(
    rows: &List<w::InboxEntry>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(limits)?, bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn ended_pieces(
    rows: &List<w::EndedDelegate>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(limits)?, bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn note_pieces(
    rows: &List<w::Note>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut size = 0;
    for row in rows {
        size = charge(size, row.measure(limits)?, bound, path)?;
    }
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).or(Err(w::Problem { path, reason: skein_codec::Reason::Bound }))?;
        encoded.push(out.finish()).expect("selected rows fit");
    }
    pieces(&encoded, bound, path)
}
pub(super) fn funding(value: &t::FundingRecord) -> w::FundingRecord {
    w::FundingRecord {
        period: match value.period {
            Some(value) => Some(w::PeriodTimes {
                began: value.began.as_nanos(),
                next: match value.next {
                    Some(value) => Some(value.as_nanos()),
                    None => None,
                },
            }),
            None => None,
        },
        funder: owner::funder(value.funder),
        parent: match value.parent {
            Some(value) => Some(owner::funder(value)),
            None => None,
        },
        numbers: owner::numbers(value.numbers),
        made: value.made,
        closed: value.closed,
    }
}
pub(super) fn note(models: &[crate::Model], value: &d::Note, limits: &w::Limits) -> Result<w::Note, w::Problem> {
    let scope = match &value.scope {
        d::NoteLocation::Party { party } => w::NoteScope::Party(w::NoteScopeParty { party: *party }),
        d::NoteLocation::Deployment => w::NoteScope::Deployment,
        d::NoteLocation::Project(value) => w::NoteScope::Project(w::NoteScopeProject { value: *value }),
        d::NoteLocation::Goal { project, task } => {
            w::NoteScope::Task(w::NoteScopeTask { project: *project, task: *task })
        }
        d::NoteLocation::Resources { project, scope } => w::NoteScope::Resource(w::NoteScopeResource {
            project: *project,
            connector: scope.connector,
            pattern: owner::pattern(&scope.segments, &scope.last),
        }),
    };
    let writer = match value.writer {
        jig_domain_notes::Author::Party { party } => w::NoteAuthor::Party(w::NoteAuthorParty { party }),
        jig_domain_notes::Author::Task { task, attempt } => w::NoteAuthor::Task(w::NoteAuthorTask { task, attempt }),
    };
    if value.tasks.capacity() > limits.note_tasks.min(w::CEILINGS.note_tasks) {
        return Err(w::Problem { path: w::paths::NOTE_TASKS, reason: skein_codec::Reason::Bound });
    }
    let mut tasks = List::with_capacity(value.tasks.capacity());
    for value in &value.tasks {
        tasks.push(*value).expect("selected references fit");
    }
    Ok(w::Note {
        number: value.number,
        revision: w::Revision { value: u64::from(value.revision) },
        scope,
        description: owner::text(&value.description),
        body: owner::text(&value.body),
        writer,
        tasks,
        changed: match value.changed {
            Some(value) => Some(value.as_nanos()),
            None => None,
        },
        acts: task::offered(models, &value.acts, limits, limits.note_acts, w::paths::NOTE_ACTS)?,
        cut: value.cut,
    })
}
