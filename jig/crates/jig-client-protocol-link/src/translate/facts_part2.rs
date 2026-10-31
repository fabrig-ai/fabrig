//! Field-for-field client contract translation (protocol/client.md, 3 and 5).
use super::{
    Child, chip_hold, document_text, executor, numbers, part, party, person_wait, proposal_key, retry_notice,
    take_chip, take_owner_chip_ending, take_result_value, wake_policy,
};
use crate::Limits;
use crate::translate::{piece_problem, too_large};
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
use skein_lib::Map;
use skein_lib::{List, Reader, Wall, bytes};
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

pub(crate) fn chip_work(value: &w::ChipWork) -> c::ChipWork {
    match value {
        w::ChipWork::Preparing => c::ChipWork::Preparing,
        w::ChipWork::Starting => c::ChipWork::Starting,
        w::ChipWork::Active => c::ChipWork::Active,
        w::ChipWork::Writing => c::ChipWork::Writing,
        w::ChipWork::Tool(row) => {
            c::ChipWork::Tool { call: bytes::copy_of(row.call()), tool: bytes::copy_of(row.tool()) }
        }
        w::ChipWork::Waiting => c::ChipWork::Waiting,
        w::ChipWork::Ending => c::ChipWork::Ending,
    }
}

pub(crate) fn chip_observation(value: &w::ChipObservation) -> c::ChipObservation {
    let opt0 = match value.started() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    c::ChipObservation {
        channel: skein_lib::Token::new(value.channel()),
        work: chip_work(value.work()),
        since: Wall::from_nanos(value.since()),
        reported_at: Wall::from_nanos(value.reported_at()),
        started: opt0,
    }
}

pub(crate) fn chip_details_fields(
    value: &w::ChipDetailsFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ChipDetails, w::Problem> {
    let opt0 = match value.run_since() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.attempt() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match &value.retry() {
        Some(item) => Some(retry_notice(item)),
        None => None,
    };
    let opt3 = match &value.person_wait() {
        Some(item) => Some(person_wait(item)),
        None => None,
    };
    let opt4 = match &value.hold() {
        Some(item) => Some(chip_hold(item, limits)?),
        None => None,
    };
    let opt5 = match &value.procedure() {
        Some(item) => Some(part(item)),
        None => None,
    };
    let opt6 = match &value.closing_connectors() {
        Some(item) => {
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    let opt7 = match &value.ending() {
        Some(item) => Some(take_owner_chip_ending(*item, index, children)?),
        None => None,
    };
    let opt8 = match value.ended_at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt9 = match &value.observation() {
        Some(item) => Some(chip_observation(item)),
        None => None,
    };
    Ok(c::ChipDetails {
        dependencies: dependencies(value.dependencies()),
        project: value.project(),
        executor: executor(value.executor()),
        phase_since: Wall::from_nanos(value.phase_since()),
        run_since: opt0,
        attempt: opt1,
        retry: opt2,
        wake: wake_policy(value.wake()),
        person_wait: opt3,
        hold: opt4,
        budget: budget(value.budget()),
        activity: value.activity(),
        resources: resource_line_piece(value.resources(), limits)?,
        resources_cut: value.resources_cut(),
        resources_observed: value.resources_observed(),
        procedure: opt5,
        live_delegates: value.live_delegates(),
        closing_connectors: opt6,
        ending: opt7,
        ended_at: opt8,
        observation: opt9,
    })
}

pub(crate) fn requester(value: &w::Requester) -> c::Requester {
    match value {
        w::Requester::Task(row) => c::Requester::Task(row.value()),
        w::Requester::Person(row) => c::Requester::Person(row.value()),
        w::Requester::Deployment(row) => c::Requester::Deployment { project: row.project() },
    }
}

pub(crate) fn chip_ending_fields(
    value: &w::ChipEndingFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ChipEnding, w::Problem> {
    Ok(match value {
        w::ChipEndingFields::Done(row) => c::ChipEnding::Done(take_result_value(row.value(), index, children)?),
        w::ChipEndingFields::Failed(row) => c::ChipEnding::Failed(document_text(row.value(), limits)?),
        w::ChipEndingFields::Cancelled(row) => {
            let opt0 = match &row.result() {
                Some(item) => Some(take_result_value(*item, index, children)?),
                None => None,
            };
            c::ChipEnding::Cancelled {
                reason: document_text(row.reason(), limits)?,
                result: opt0,
                by: requester(row.by()),
                at: Wall::from_nanos(row.at()),
            }
        }
    })
}

pub(crate) fn result_field(value: &w::ResultField) -> c::ResultField {
    c::ResultField { name: bytes::copy_of(value.name()), value: bytes::copy_of(value.value()) }
}

pub(crate) fn result_value_fields(
    value: &w::ResultValueFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ResultValue, w::Problem> {
    Ok(match value {
        w::ResultValueFields::Report(row) => c::ResultValue::Report(document_text(row.value(), limits)?),
        w::ResultValueFields::OwnerVerdict(row) => {
            let mut list0 = List::with_capacity(row.fields().len());
            for item in row.fields() {
                list0.push(result_field(item)).expect("reserved supplied list");
            }
            c::ResultValue::OwnerVerdict {
                verdict: row.verdict(),
                words: document_text(row.words(), limits)?,
                fields: list0,
            }
        }
        w::ResultValueFields::Verdict(row) => {
            let mut list0 = List::with_capacity(row.fields().len());
            for item in row.fields() {
                list0.push(result_field(item)).expect("reserved supplied list");
            }
            let mut list1 = List::with_capacity(row.tasks().len());
            for item in row.tasks() {
                list1.push(take_chip(*item, index, children)?).expect("reserved supplied list");
            }
            let opt2 = match &row.proposal() {
                Some(item) => Some(proposal_key(item)),
                None => None,
            };
            c::ResultValue::Verdict {
                verdict: row.verdict(),
                words: document_text(row.words(), limits)?,
                fields: list0,
                tasks: list1,
                proposal: opt2,
            }
        }
        w::ResultValueFields::Change(row) => c::ResultValue::Change {
            kind: row.kind(),
            part: part(row.part()),
            words: document_text(row.words(), limits)?,
        },
        w::ResultValueFields::Choice(row) => c::ResultValue::Choice {
            option: row.option(),
            by: party(row.by(), limits)?,
            reason: document_text(row.reason(), limits)?,
        },
        w::ResultValueFields::Approved(row) => {
            let opt0 = match &row.words() {
                Some(item) => Some(document_text(item, limits)?),
                None => None,
            };
            c::ResultValue::Approved { state: bytes::copy_of(row.state()), words: opt0 }
        }
        w::ResultValueFields::Failure(row) => c::ResultValue::Failure(document_text(row.value(), limits)?),
        w::ResultValueFields::Words(row) => c::ResultValue::Words(document_text(row.value(), limits)?),
    })
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
