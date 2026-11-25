//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use super::parties::{actor, decision};
use super::parts::resource_line_piece;
use super::{
    Child, budget, chip_hold, dependencies, document_text, part, party, person_wait, proposal_key, requester,
    retry_notice, take_chip, take_chip_phase, take_ending, take_owner_chip_details, take_owner_chip_ending,
    take_result_value, wake_policy,
};
use crate::Limits;
use crate::generated::client;
use crate::translate::text_problem;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use skein_lib::{List, Map, Wall, bytes};

pub(crate) fn executor(value: &w::ChipExecutor) -> c::ChipExecutor {
    client::chip_executor_to_domain(value.clone())
}

pub(crate) fn phase(value: &w::Phase) -> c::Phase {
    client::phase_to_domain(value.clone())
}

pub(crate) fn chip_fields(
    value: &w::ChipFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::Chip, w::Problem> {
    Ok(c::Chip {
        task: value.task,
        title: document_text(&value.title, limits)?,
        executor: executor(&value.executor),
        phase: phase(&value.phase),
        details: Box::new(take_chip_phase(value.details, index, children)?),
    })
}

pub(crate) fn closing_stage(value: &w::ClosingStage) -> c::ClosingStage {
    client::closing_stage_to_domain(value.clone())
}

pub(crate) fn chip_phase(value: &w::ChipPhase) -> c::ChipPhase {
    client::chip_phase_to_domain(value.clone())
}

pub(crate) fn chip_phase_fields(
    value: &w::ChipPhaseOwnerFields,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ChipPhaseFacts, w::Problem> {
    let phase = chip_phase(&value.phase);
    let details = take_owner_chip_details(value.details, index, children)?;
    let required_missing = match &phase {
        c::ChipPhase::Ended => details.ended_at.is_none(),
        c::ChipPhase::Active(c::RunStage::BackingOff { .. }) => details.retry.is_none(),
        c::ChipPhase::Waiting { .. }
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_)
        | c::ChipPhase::Held { .. }
        | c::ChipPhase::Active(
            c::RunStage::Idle
            | c::RunStage::Due
            | c::RunStage::Preparing
            | c::RunStage::Claimed { .. }
            | c::RunStage::Running { .. },
        ) => false,
    };
    if required_missing {
        return Err(crate::translate::piece_problem(skein_codec::Reason::Malformed));
    }
    Ok(c::ChipPhaseFacts { phase, details: Box::new(details) })
}

pub(crate) fn closing_fields(
    value: &w::ClosingFields,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::Closing, w::Problem> {
    let opt0 = match &(&value.connectors) {
        Some(item) => {
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    let opt1 = match &(&value.stage) {
        Some(item) => Some(closing_stage(item)),
        None => None,
    };
    let opt2 = match &(&value.ending) {
        Some(item) => Some(take_ending(*item, index, children)?),
        None => None,
    };
    Ok(c::Closing { delegates: value.delegates, connectors: opt0, stage: opt1, ending: opt2 })
}

pub(crate) fn ending_fields(
    value: &w::EndingFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::Ending, w::Problem> {
    Ok(match value {
        w::EndingFields::Task(row) => c::Ending::Task(Box::new(take_owner_chip_ending(row.value, index, children)?)),
        w::EndingFields::Done => c::Ending::Done,
        w::EndingFields::Failed(row) => {
            let opt0 = match &(&row.reason) {
                Some(item) => Some(match c::Text::new(bytes::copy_of(item), limits.client().text) {
                    Ok(value) => value,
                    Err(_) => return Err(text_problem()),
                }),
                None => None,
            };
            c::Ending::Failed { reason: opt0 }
        }
        w::EndingFields::Cancelled(row) => {
            let opt0 = match &(&row.reason) {
                Some(item) => Some(match c::Text::new(bytes::copy_of(item), limits.client().text) {
                    Ok(value) => value,
                    Err(_) => return Err(text_problem()),
                }),
                None => None,
            };
            let opt1 = match &row.at {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::Ending::Cancelled { by: actor(&row.by), reason: opt0, at: opt1 }
        }
        w::EndingFields::Withdrawn(row) => c::Ending::Withdrawn {
            reason: match c::Text::new(bytes::copy_of(&row.reason), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::EndingFields::Decided(row) => {
            let opt0 = match &(&row.party) {
                Some(item) => Some(party(item, limits)?),
                None => None,
            };
            let opt1 = match &(&row.how) {
                Some(item) => Some(decision(item, limits)?),
                None => None,
            };
            c::Ending::Decided { by: row.by, party: opt0, how: opt1, at: Wall::from_nanos(row.at) }
        }
        w::EndingFields::Deleted => c::Ending::Deleted,
    })
}

pub(crate) fn prior_phase(value: &w::PriorPhase) -> c::PriorPhase {
    client::prior_phase_to_domain(value.clone())
}

pub(crate) fn chip_observation(value: &w::ChipObservation) -> c::ChipObservation {
    client::chip_observation_to_domain(value.clone())
}

pub(crate) fn chip_details_fields(
    value: &w::ChipDetailsFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ChipDetails, w::Problem> {
    let opt0 = match &value.run_since {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match &value.attempt {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match &(&value.retry) {
        Some(item) => Some(retry_notice(item)?),
        None => None,
    };
    let opt3 = match &(&value.person_wait) {
        Some(item) => Some(person_wait(item)),
        None => None,
    };
    let opt4 = match &(&value.hold) {
        Some(item) => Some(chip_hold(item, limits)?),
        None => None,
    };
    let opt5 = match &(&value.procedure) {
        Some(item) => Some(part(item)),
        None => None,
    };
    let opt6 = match &(&value.closing_connectors) {
        Some(item) => {
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    let opt7 = match &(&value.ending) {
        Some(item) => Some(take_owner_chip_ending(*item, index, children)?),
        None => None,
    };
    let opt8 = match &value.ended_at {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt9 = match &(&value.observation) {
        Some(item) => Some(chip_observation(item)),
        None => None,
    };
    Ok(c::ChipDetails {
        dependencies: dependencies(&value.dependencies),
        project: value.project,
        executor: super::executor(&value.executor),
        phase_since: Wall::from_nanos(value.phase_since),
        run_since: opt0,
        attempt: opt1,
        retry: opt2,
        wake: wake_policy(&value.wake),
        person_wait: opt3,
        hold: opt4,
        budget: budget(&value.budget),
        activity: value.activity,
        resources: resource_line_piece(&value.resources, limits)?,
        resources_cut: value.resources_cut,
        resources_observed: value.resources_observed,
        procedure: opt5,
        live_delegates: value.live_delegates,
        closing_connectors: opt6,
        ending: opt7,
        ended_at: opt8,
        observation: opt9,
    })
}

pub(crate) fn chip_ending_fields(
    value: &w::ChipEndingFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ChipEnding, w::Problem> {
    Ok(match value {
        w::ChipEndingFields::Done(row) => c::ChipEnding::Done(take_result_value(row.value, index, children)?),
        w::ChipEndingFields::Failed(row) => c::ChipEnding::Failed(document_text(&row.value, limits)?),
        w::ChipEndingFields::Cancelled(row) => {
            let opt0 = match &(&row.result) {
                Some(item) => Some(take_result_value(*item, index, children)?),
                None => None,
            };
            c::ChipEnding::Cancelled {
                reason: document_text(&row.reason, limits)?,
                result: opt0,
                by: requester(&row.by),
                at: Wall::from_nanos(row.at),
            }
        }
    })
}

pub(crate) fn result_field(value: &w::ResultField) -> c::ResultField {
    client::result_field_to_domain(value.clone())
}

pub(crate) fn result_value_fields(
    value: &w::ResultValueFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ResultValue, w::Problem> {
    Ok(match value {
        w::ResultValueFields::Report(row) => c::ResultValue::Report(document_text(&row.value, limits)?),
        w::ResultValueFields::OwnerVerdict(row) => {
            let mut list0 = List::with_capacity(row.fields.len());
            for item in &row.fields {
                list0.push(result_field(item)).expect("reserved supplied list");
            }
            c::ResultValue::OwnerVerdict {
                verdict: row.verdict,
                words: document_text(&row.words, limits)?,
                fields: list0,
            }
        }
        w::ResultValueFields::Verdict(row) => {
            let mut list0 = List::with_capacity(row.fields.len());
            for item in &row.fields {
                list0.push(result_field(item)).expect("reserved supplied list");
            }
            let mut list1 = List::with_capacity(row.tasks.len());
            for item in &row.tasks {
                list1.push(take_chip(*item, index, children)?).expect("reserved supplied list");
            }
            let opt2 = match &(&row.proposal) {
                Some(item) => Some(proposal_key(item)),
                None => None,
            };
            c::ResultValue::Verdict {
                verdict: row.verdict,
                words: document_text(&row.words, limits)?,
                fields: list0,
                tasks: list1,
                proposal: opt2,
            }
        }
        w::ResultValueFields::Change(row) => {
            c::ResultValue::Change { kind: row.kind, part: part(&row.part), words: document_text(&row.words, limits)? }
        }
        w::ResultValueFields::Choice(row) => c::ResultValue::Choice {
            option: row.option,
            by: party(&row.by, limits)?,
            reason: document_text(&row.reason, limits)?,
        },
        w::ResultValueFields::Approved(row) => {
            let opt0 = match &(&row.words) {
                Some(item) => Some(document_text(item, limits)?),
                None => None,
            };
            c::ResultValue::Approved { state: bytes::copy_of(&row.state), words: opt0 }
        }
        w::ResultValueFields::Failure(row) => c::ResultValue::Failure(document_text(&row.value, limits)?),
        w::ResultValueFields::Words(row) => c::ResultValue::Words(document_text(&row.value, limits)?),
    })
}
