//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use super::parties::proposer;
use super::{person_address, requester};
use crate::Limits;
use crate::translate::{piece_problem, text_problem, too_large};
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v22 as w;
use skein_lib::{List, Reader, Wall, bytes};

pub(crate) fn document_text(value: &w::DocumentText, limits: &Limits) -> Result<c::DocumentText, w::Problem> {
    Ok(c::DocumentText {
        bytes: match c::Text::new(bytes::copy_of(value.bytes()), limits.client().long_text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
        cut: value.cut(),
    })
}

pub(crate) fn proposal_key(value: &w::ProposalKey) -> c::ProposalKey {
    c::ProposalKey { proposer: proposer(value.proposer()), number: value.number() }
}

pub(crate) fn executor(value: &w::Executor) -> c::Executor {
    match value {
        w::Executor::Job(row) => c::Executor::Job { kind: row.kind() },
        w::Executor::Agent(row) => c::Executor::Agent { charter: row.charter() },
        w::Executor::Procedure(row) => c::Executor::Procedure { connector: row.connector(), code: row.code() },
        w::Executor::Person(row) => c::Executor::Person(person_address(row.value())),
    }
}

pub(crate) fn part_kind(value: &w::PartKind) -> c::PartKind {
    match value {
        w::PartKind::Resource => c::PartKind::Resource,
        w::PartKind::Pool => c::PartKind::Pool,
        w::PartKind::Result => c::PartKind::Result,
        w::PartKind::Procedure => c::PartKind::Procedure,
        w::PartKind::Effect => c::PartKind::Effect,
        w::PartKind::Outbox => c::PartKind::Outbox,
        w::PartKind::Hold => c::PartKind::Hold,
    }
}

pub(crate) fn part_key(value: &w::PartKey) -> c::PartKey {
    match value {
        w::PartKey::Path(row) => {
            let mut list0 = List::with_capacity(row.value().len());
            for item in row.value() {
                list0.push(bytes::copy_of(item)).expect("reserved supplied list");
            }
            c::PartKey::Path(list0)
        }
        w::PartKey::Number(row) => c::PartKey::Number(row.value()),
    }
}

pub(crate) fn part(value: &w::Part) -> c::Part {
    c::Part { connector: value.connector(), kind: part_kind(value.kind()), key: part_key(value.key()) }
}

pub(crate) fn numbers(value: &w::Numbers) -> c::Numbers {
    c::Numbers {
        budget: value.budget(),
        spent: value.spent(),
        spent_below: value.spent_below(),
        reserved: value.reserved(),
    }
}

pub(crate) fn funder(value: &w::Funder) -> c::Funder {
    match value {
        w::Funder::Task(row) => c::Funder::Task(row.value()),
        w::Funder::Pool(row) => c::Funder::Pool { project: row.project(), person: row.person(), period: row.period() },
        w::Funder::Recurring(row) => {
            c::Funder::Recurring { project: row.project(), task: row.task(), period: row.period() }
        }
        w::Funder::Period(row) => c::Funder::Period { project: row.project(), period: row.period() },
    }
}

pub(crate) fn budget(value: &w::Budget) -> c::Budget {
    let opt0 = match value.deadline() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    c::Budget { numbers: numbers(value.numbers()), left: value.left(), funder: funder(value.funder()), deadline: opt0 }
}

pub(crate) fn resource_writer(value: &w::ResourceWriter) -> c::ResourceWriter {
    match value {
        w::ResourceWriter::Run(row) => c::ResourceWriter::Run { task: row.task(), attempt: row.attempt() },
        w::ResourceWriter::Effect(row) => c::ResourceWriter::Effect { entry: row.entry() },
    }
}

pub(crate) fn resource_line(value: &w::ResourceLine) -> c::ResourceLine {
    let opt0 = match value.holder() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match &value.writer() {
        Some(item) => Some(resource_writer(item)),
        None => None,
    };
    c::ResourceLine {
        part: part(value.part()),
        kind: value.kind(),
        taken: value.taken(),
        holder: opt0,
        writer: opt1,
        writer_lost: value.writer_lost(),
    }
}

pub(crate) fn resource_line_piece(bytes: &[u8], limits: &Limits) -> Result<List<c::ResourceLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(limits.client().rows);
    for _ in 0..=limits.client().rows {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(problem) => return Err(piece_problem(problem)),
        };
        let Some(item) = item else { return Ok(rows) };
        let row = w::ResourceLine::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = resource_line(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn dependencies(values: &List<w::Dependency>) -> List<c::Dependency> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        let added = match value.added() {
            Some(added) => Some(c::DependencyAdded { at: Wall::from_nanos(added.at()), by: requester(added.by()) }),
            None => None,
        };
        rows.push(c::Dependency {
            number: value.number(),
            pending: value.pending(),
            added,
            keeps: value.keeps(),
            status: match value.status() {
                Some(value) => Some(match value {
                    w::TaskStatus::Done => c::TaskStatus::Done,
                    w::TaskStatus::Failed => c::TaskStatus::Failed,
                    w::TaskStatus::Cancelled => c::TaskStatus::Cancelled,
                }),
                None => None,
            },
        })
        .expect("same dependency count");
    }
    rows
}
