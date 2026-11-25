//! Whole owner facts encoded without clipping (protocol/client/wire.md, 1).
use super::owner;
use super::{pieces, problem, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, bytes};
#[must_use]
pub fn class(value: t::Class) -> w::HoldFailure {
    mapped::hold_failure_to_record(&value)
}
#[must_use]
pub fn retry(value: &t::RetryNotice) -> w::RetryNotice {
    match value {
        t::RetryNotice::Failed { class: value, count, allowance, reason } => {
            w::RetryNotice::OwnerFailed(w::RetryNoticeOwnerFailed {
                class: class(*value),
                count: *count,
                allowance: *allowance,
                reason: reason.clone(),
            })
        }
        t::RetryNotice::Refused { count, reason } => {
            w::RetryNotice::Refused(w::RetryNoticeRefused { count: *count, reason: reason.clone() })
        }
    }
}
#[must_use]
pub fn hold(value: t::Hold) -> w::HoldCategory {
    mapped::hold_category_to_record(&value)
}
#[must_use]
pub fn active(value: t::Active) -> w::RunStage {
    mapped::run_stage_to_record(&value)
}
#[must_use]
pub fn stage(value: t::Stage) -> w::ClosingStage {
    mapped::closing_stage_to_record(&value)
}
#[must_use]
pub fn address(value: t::PersonAddress) -> w::PersonAddress {
    mapped::person_address_to_record(&value)
}
#[must_use]
pub fn phase(value: &d::PhaseLine) -> w::ChipPhase {
    mapped::chip_phase_to_record(value)
}
#[must_use]
pub fn prior(value: d::PriorPhase) -> w::PriorPhase {
    mapped::prior_phase_to_record(&value)
}
#[must_use]
pub fn holder(value: t::EscalationHolder) -> w::EscalationHolder {
    mapped::escalation_holder_to_record(&value)
}
#[expect(clippy::match_same_arms, reason = "the interim wire projects each decider kind to its number")]
#[must_use]
pub fn escalation(value: &d::EscalationState) -> w::HoldEscalation {
    match value {
        d::EscalationState::Routing => w::HoldEscalation::Routing,
        d::EscalationState::Waiting { holder: value, entry, since } => {
            w::HoldEscalation::Waiting(w::HoldEscalationWaiting {
                holder: holder(*value),
                entry: *entry,
                since: skein_codec::convert::wall_to_record(since),
            })
        }
        d::EscalationState::Rejected { by, reason } => w::HoldEscalation::Rejected(w::HoldEscalationRejected {
            by: match by {
                t::EscalationDecider::Task(number) => *number,
                t::EscalationDecider::Person(number) => *number,
            },
            reason: owner::text(reason),
        }),
    }
}
#[must_use]
pub fn tries(value: t::Tries) -> w::FailureTries {
    mapped::failure_tries_to_record(&value)
}
#[must_use]
pub fn held(value: &d::HoldPhaseLine) -> w::ChipHold {
    mapped::chip_hold_to_record(value)
}
#[must_use]
pub fn writer(value: t::Writer) -> w::ResourceWriter {
    mapped::resource_writer_to_record(&value)
}
#[must_use]
pub fn resource(value: &d::ResourceLine) -> w::ResourceLine {
    mapped::resource_line_to_record(value)
}
pub fn resources_pieces(
    rows: &List<w::ResourceLine>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
#[must_use]
pub fn work(value: &d::ChipWork) -> w::ChipWork {
    mapped::chip_work_to_record(value)
}
#[must_use]
pub fn observation(value: &d::ChipObservation) -> w::ChipObservation {
    mapped::chip_observation_to_record(value)
}
#[must_use]
pub fn person_notice(value: t::PersonStallNotice) -> w::PersonStallNotice {
    mapped::person_stall_notice_to_record(&value)
}
#[must_use]
pub fn person_wait(value: t::PersonWait) -> w::PersonWait {
    mapped::person_wait_to_record(&value)
}
fn compact(value: &d::PhaseLine) -> w::Phase {
    match value {
        d::PhaseLine::Waiting { .. } => w::Phase::Waiting,
        d::PhaseLine::Active(value) => match value {
            t::Active::Idle => w::Phase::Idle,
            t::Active::Due => w::Phase::Due,
            t::Active::Preparing => w::Phase::Preparing,
            t::Active::Claimed { .. } => w::Phase::Claimed,
            t::Active::Running { .. } => w::Phase::Running,
            t::Active::BackingOff { .. } => w::Phase::BackingOff,
        },
        d::PhaseLine::Person { .. } => w::Phase::Active,
        d::PhaseLine::Closing(_) => w::Phase::Closing,
        d::PhaseLine::Held { .. } => w::Phase::Held,
        d::PhaseLine::Ended => w::Phase::Ended,
    }
}
fn chip_executor(value: &d::Chip) -> Result<w::ChipExecutor, w::Problem> {
    Ok(match value.details.executor {
        t::Executor::Job { kind } => {
            if value.executor != d::ExecutorKind::Job {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Job(w::ChipExecutorJob { kind })
        }
        t::Executor::Agent { charter } => {
            if value.executor != d::ExecutorKind::Agent {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Agent(w::ChipExecutorAgent { charter })
        }
        t::Executor::Procedure { connector, code } => {
            if value.executor != d::ExecutorKind::Procedure {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Procedure(w::ChipExecutorProcedure { connector, kind: code })
        }
        t::Executor::Person(address) => {
            if value.executor != d::ExecutorKind::Person {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            match address {
                t::PersonAddress::Person(party) => w::ChipExecutor::Party(w::ExecutorParty { party }),
                t::PersonAddress::Role(role) => {
                    w::ChipExecutor::Role(w::ExecutorRole { project: value.details.project, role })
                }
            }
        }
    })
}
pub fn chip(value: &d::Chip, limits: &w::Limits) -> Result<w::Chip, w::Problem> {
    let mut facts = List::with_capacity(5);
    facts
        .push(w::Fact::Chip(w::ChipRow {
            parent: None,
            value: w::ChipFields {
                task: value.number,
                title: owner::text(&value.title),
                executor: chip_executor(value)?,
                phase: compact(&value.phase),
                details: 1,
            },
        }))
        .expect("chip root");
    facts
        .push(w::Fact::ChipPhase(w::ChipPhaseRow {
            parent: Some(0),
            value: w::ChipPhaseOwnerFields { phase: phase(&value.phase), details: 2 },
        }))
        .expect("chip phase");
    let ending = match &value.details.ending {
        Some(_) => Some(3),
        None => None,
    };
    facts
        .push(w::Fact::ChipDetails(Box::new(w::ChipDetailsRow {
            parent: Some(1),
            value: details_fields(&value.details, ending, limits)?,
        })))
        .expect("chip details");
    if let Some(ending) = &value.details.ending {
        append_ending(ending, Some(2), &mut facts)?;
    }
    Ok(w::Chip { rows: fact_pieces(&facts, limits.chip_rows.min(w::CEILINGS.chip_rows), w::paths::CHIP_ROWS, limits)? })
}
fn details_fields(
    value: &d::ChipDetails,
    ending: Option<u32>,
    limits: &w::Limits,
) -> Result<w::ChipDetailsFields, w::Problem> {
    let mut resources = List::with_capacity(value.resources.len());
    for item in &value.resources {
        resources.push(resource(item)).expect("supplied resource rows");
    }
    Ok(w::ChipDetailsFields {
        dependencies: dependencies(&value.dependencies),
        project: value.project,
        executor: owner::executor(value.executor),
        phase_since: skein_codec::convert::wall_to_record(&value.phase_since),
        run_since: match value.run_since {
            Some(item) => Some(skein_codec::convert::wall_to_record(&item)),
            None => None,
        },
        attempt: value.attempt,
        retry: match &value.retry {
            Some(item) => Some(retry(item)),
            None => None,
        },
        wake: owner::wake(value.wake),
        person_wait: match value.person_wait {
            Some(item) => Some(person_wait(item)),
            None => None,
        },
        hold: match &value.hold {
            Some(item) => Some(held(item)),
            None => None,
        },
        budget: owner::budget(&value.budget),
        activity: value.activity,
        resources: resources_pieces(
            &resources,
            limits.chip_details_fields_resources.min(w::CEILINGS.chip_details_fields_resources),
            w::paths::CHIP_DETAILS_FIELDS_RESOURCES,
            limits,
        )?,
        resources_cut: value.resources_cut,
        resources_observed: value.resources_observed,
        procedure: match &value.procedure {
            Some(item) => Some(owner::part(item)),
            None => None,
        },
        live_delegates: value.live_delegates,
        closing_connectors: value.closing_connectors.clone(),
        ending,
        ended_at: match value.ended_at {
            Some(item) => Some(skein_codec::convert::wall_to_record(&item)),
            None => None,
        },
        observation: match &value.observation {
            Some(item) => Some(observation(item)),
            None => None,
        },
    })
}

pub fn ending(value: &d::Ending, limits: &w::Limits) -> Result<w::ChipEnding, w::Problem> {
    let mut facts = List::with_capacity(2);
    append_ending(value, None, &mut facts)?;
    Ok(w::ChipEnding {
        rows: fact_pieces(
            &facts,
            limits.chip_ending_rows.min(w::CEILINGS.chip_ending_rows),
            w::paths::CHIP_ENDING_ROWS,
            limits,
        )?,
    })
}
pub(super) fn append_ending(
    value: &d::Ending,
    parent: Option<u32>,
    facts: &mut List<w::Fact>,
) -> Result<(), w::Problem> {
    let index = facts.len();
    let child = index.checked_add(1).ok_or(problem(skein_codec::Reason::Bound))?;
    let fields = match value {
        d::Ending::Done(_) => w::ChipEndingFields::Done(w::ChipEndingDoneFields { value: child }),
        d::Ending::Failed(value) => {
            w::ChipEndingFields::Failed(w::ChipEndingFailedFields { value: owner::text(value) })
        }
        d::Ending::Cancelled { reason, result, by, at } => {
            w::ChipEndingFields::Cancelled(w::ChipEndingCancelledFields {
                reason: owner::text(reason),
                result: match result {
                    Some(_) => Some(child),
                    None => None,
                },
                by: owner::requester(*by),
                at: skein_codec::convert::wall_to_record(at),
            })
        }
    };
    facts.push(w::Fact::ChipEnding(w::ChipEndingRow { parent, value: fields })).expect("bounded ending facts");
    let result = match value {
        d::Ending::Done(value) => Some(value),
        d::Ending::Failed(_) => None,
        d::Ending::Cancelled { result, .. } => result.as_ref(),
    };
    if let Some(value) = result {
        facts
            .push(w::Fact::ResultValue(w::ResultValueRow { parent: Some(index), value: result_fields(value)? }))
            .expect("bounded result facts");
    }
    Ok(())
}
pub(super) fn result_fields(value: &d::ResultValue) -> Result<w::ResultValueFields, w::Problem> {
    Ok(match value {
        d::ResultValue::Approved { state, words } => w::ResultValueFields::Approved(w::ResultValueApprovedFields {
            state: bytes::copy_of(state),
            words: match words {
                Some(item) => Some(owner::text(item)),
                None => None,
            },
        }),
        d::ResultValue::Report(value) => {
            w::ResultValueFields::Report(w::ResultValueReportFields { value: owner::text(value) })
        }
        d::ResultValue::Failure(value) => {
            w::ResultValueFields::Failure(w::ResultValueFailureFields { value: owner::text(value) })
        }
        d::ResultValue::Change { kind, part, words } => w::ResultValueFields::Change(w::ResultValueChangeFields {
            kind: *kind,
            part: owner::part(part),
            words: owner::text(words),
        }),
        d::ResultValue::Verdict { code, fields, words } => {
            let mut copied = List::with_capacity(super::count(fields.len())?);
            for item in fields {
                copied.push(mapped::result_field_to_record(item)).expect("supplied result fields");
            }
            w::ResultValueFields::OwnerVerdict(w::ResultValueOwnerVerdictFields {
                verdict: *code,
                words: owner::text(words),
                fields: copied,
            })
        }
    })
}

pub fn fact_pieces(
    rows: &List<w::Fact>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}

pub(crate) fn dependencies(values: &List<d::Dependency>) -> List<w::Dependency> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(mapped::dependency_to_record(value)).expect("same dependency count");
    }
    rows
}
