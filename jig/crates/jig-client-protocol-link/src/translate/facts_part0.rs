//! Field-for-field client contract translation (protocol/client.md, 3 and 5).
use super::{Child, take_chip_phase};
use crate::Limits;
use crate::translate::text_problem;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
use skein_lib::Map;
use skein_lib::{List, Wall, bytes};
pub(crate) fn document_text(value: &w::DocumentText, limits: &Limits) -> Result<c::DocumentText, w::Problem> {
    Ok(c::DocumentText {
        bytes: match c::Text::new(bytes::copy_of(value.bytes()), limits.client.text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
        cut: value.cut(),
    })
}

pub(crate) fn executor(value: &w::Executor) -> c::Executor {
    match value {
        w::Executor::Agent(row) => c::Executor::Agent { charter: row.charter() },
        w::Executor::Procedure(row) => c::Executor::Procedure { connector: row.connector(), kind: row.kind() },
        w::Executor::Party(row) => c::Executor::Party { party: row.party() },
        w::Executor::Role(row) => c::Executor::Role { project: row.project(), role: row.role() },
    }
}

pub(crate) fn phase(value: &w::Phase) -> c::Phase {
    match value {
        w::Phase::Waiting => c::Phase::Waiting,
        w::Phase::Idle => c::Phase::Idle,
        w::Phase::Due => c::Phase::Due,
        w::Phase::Preparing => c::Phase::Preparing,
        w::Phase::Claimed => c::Phase::Claimed,
        w::Phase::Running => c::Phase::Running,
        w::Phase::BackingOff => c::Phase::BackingOff,
        w::Phase::Active => c::Phase::Active,
        w::Phase::Closing => c::Phase::Closing,
        w::Phase::Held => c::Phase::Held,
        w::Phase::Ended => c::Phase::Ended,
        w::Phase::Done => c::Phase::Done,
        w::Phase::Failed => c::Phase::Failed,
        w::Phase::Cancelled => c::Phase::Cancelled,
    }
}

pub(crate) fn chip_fields(
    value: &w::ChipFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::Chip, w::Problem> {
    Ok(c::Chip {
        task: value.task(),
        title: document_text(value.title(), limits)?,
        executor: executor(value.executor()),
        phase: phase(value.phase()),
        details: Box::new(take_chip_phase(value.details(), index, children)?),
    })
}

pub(crate) fn run_stage(value: &w::RunStage) -> c::RunStage {
    match value {
        w::RunStage::Idle => c::RunStage::Idle,
        w::RunStage::Due => c::RunStage::Due,
        w::RunStage::Preparing => c::RunStage::Preparing,
        w::RunStage::Claimed(row) => c::RunStage::Claimed { attempt: row.attempt() },
        w::RunStage::Running(row) => c::RunStage::Running { attempt: row.attempt() },
        w::RunStage::BackingOff(row) => c::RunStage::BackingOff { until: Wall::from_nanos(row.until()) },
    }
}

pub(crate) fn person_address(value: &w::PersonAddress) -> c::PersonAddress {
    match value {
        w::PersonAddress::Party(row) => c::PersonAddress::Party(row.value()),
        w::PersonAddress::Role(row) => c::PersonAddress::Role(row.value()),
    }
}

pub(crate) fn closing_stage(value: &w::ClosingStage) -> c::ClosingStage {
    match value {
        w::ClosingStage::Run(row) => c::ClosingStage::Run { attempt: row.attempt() },
        w::ClosingStage::Delegates => c::ClosingStage::Delegates,
        w::ClosingStage::Effects => c::ClosingStage::Effects,
        w::ClosingStage::Releases => c::ClosingStage::Releases,
        w::ClosingStage::Settled => c::ClosingStage::Settled,
    }
}

pub(crate) fn hold_failure(value: &w::HoldFailure) -> c::HoldFailure {
    match value {
        w::HoldFailure::Transient => c::HoldFailure::Transient,
        w::HoldFailure::Permanent => c::HoldFailure::Permanent,
        w::HoldFailure::Run => c::HoldFailure::Run,
        w::HoldFailure::Agent => c::HoldFailure::Agent,
        w::HoldFailure::Lost => c::HoldFailure::Lost,
        w::HoldFailure::Invalid => c::HoldFailure::Invalid,
    }
}

pub(crate) fn hold_category(value: &w::HoldCategory) -> c::HoldCategory {
    match value {
        w::HoldCategory::Exceeded(value) => c::HoldCategory::Exceeded(c::Exceeded {
            bound: match value.bound() {
                w::Bound::Reasoning => c::Bound::Reasoning,
                w::Bound::Metadata => c::Bound::Metadata,
                w::Bound::Items => c::Bound::Items,
                w::Bound::Answer => c::Bound::Answer,
                w::Bound::Tools => c::Bound::Tools,
                w::Bound::HistoryItems => c::Bound::HistoryItems,
                w::Bound::History => c::Bound::History,
                w::Bound::Window => c::Bound::Window,
                w::Bound::Output => c::Bound::Output,
            },
            value: value.value(),
        }),
        w::HoldCategory::Failures(row) => c::HoldCategory::Failures { class: hold_failure(row.class()) },
        w::HoldCategory::Stalled => c::HoldCategory::Stalled,
        w::HoldCategory::Dependency(row) => c::HoldCategory::Dependency { task: row.task() },
        w::HoldCategory::Stopped => c::HoldCategory::Stopped,
        w::HoldCategory::Placed(row) => c::HoldCategory::Placed { hold: row.hold() },
        w::HoldCategory::Drift => c::HoldCategory::Drift,
        w::HoldCategory::Effects => c::HoldCategory::Effects,
        w::HoldCategory::EffectFailed => c::HoldCategory::EffectFailed,
        w::HoldCategory::Uncertain(row) => c::HoldCategory::Uncertain { entry: row.entry() },
        w::HoldCategory::Budget => c::HoldCategory::Budget,
        w::HoldCategory::Deadline => c::HoldCategory::Deadline,
        w::HoldCategory::HoldsWaited => c::HoldCategory::HoldsWaited,
        w::HoldCategory::ResourceUnavailable => c::HoldCategory::ResourceUnavailable,
        w::HoldCategory::ResourceAccess => c::HoldCategory::ResourceAccess,
        w::HoldCategory::PoolLost(row) => c::HoldCategory::PoolLost { pool: row.pool() },
        w::HoldCategory::Procedure => c::HoldCategory::Procedure,
        w::HoldCategory::Circle => c::HoldCategory::Circle,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_chip_phase(value: &w::OwnerChipPhase) -> c::OwnerChipPhase {
    match value {
        w::OwnerChipPhase::Waiting(row) => {
            let mut list0 = List::with_capacity(row.dependencies().len());
            for item in row.dependencies() {
                list0.push(*item).expect("reserved supplied list");
            }
            c::OwnerChipPhase::Waiting { dependencies: list0, more: row.more(), holds: row.holds() }
        }
        w::OwnerChipPhase::Active(row) => c::OwnerChipPhase::Active(run_stage(row.value())),
        w::OwnerChipPhase::Person(row) => {
            let opt0 = match row.taken_by() {
                Some(item) => Some(*item),
                None => None,
            };
            let opt1 = match row.since() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::OwnerChipPhase::Person { address: person_address(row.address()), taken_by: opt0, since: opt1 }
        }
        w::OwnerChipPhase::Closing(row) => c::OwnerChipPhase::Closing(closing_stage(row.value())),
        w::OwnerChipPhase::Held(row) => {
            c::OwnerChipPhase::Held { reason: hold_category(row.reason()), revision: row.revision() }
        }
        w::OwnerChipPhase::Ended => c::OwnerChipPhase::Ended,
    }
}

pub(crate) fn failure_class(value: &w::FailureClass) -> c::FailureClass {
    match value {
        w::FailureClass::HostLost => c::FailureClass::HostLost,
        w::FailureClass::Temporary => c::FailureClass::Temporary,
        w::FailureClass::Permanent => c::FailureClass::Permanent,
        w::FailureClass::Stall => c::FailureClass::Stall,
    }
}

pub(crate) fn retry_notice(value: &w::RetryNotice) -> c::RetryNotice {
    match value {
        w::RetryNotice::OwnerFailed(row) => c::RetryNotice::OwnerFailed {
            class: hold_failure(row.class()),
            count: row.count(),
            allowance: row.allowance(),
        },
        w::RetryNotice::Failed(row) => {
            c::RetryNotice::Failed { class: failure_class(row.class()), count: row.count(), allowance: row.allowance() }
        }
        w::RetryNotice::Refused(row) => c::RetryNotice::Refused { count: row.count() },
    }
}

pub(crate) fn call_identity(value: &w::CallIdentity, limits: &Limits) -> Result<c::CallIdentity, w::Problem> {
    Ok(c::CallIdentity {
        owner: skein_lib::Token::new(value.owner()),
        sequence: value.sequence(),
        retry: value.retry(),
        id: match c::Text::new(bytes::copy_of(value.id()), limits.client.text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
    })
}

pub(crate) fn run_work(value: &w::RunWork, limits: &Limits) -> Result<c::RunWork, w::Problem> {
    Ok(match value {
        w::RunWork::Preparing => c::RunWork::Preparing,
        w::RunWork::Starting => c::RunWork::Starting,
        w::RunWork::Active => c::RunWork::Active,
        w::RunWork::Writing => c::RunWork::Writing,
        w::RunWork::Tool(row) => c::RunWork::Tool {
            call: call_identity(row.call(), limits)?,
            tool: match c::Text::new(bytes::copy_of(row.tool()), limits.client.text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::RunWork::Waiting => c::RunWork::Waiting,
        w::RunWork::Ending => c::RunWork::Ending,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn run_observation(value: &w::RunObservation, limits: &Limits) -> Result<c::RunObservation, w::Problem> {
    let opt0 = match value.started() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::RunObservation {
        channel: skein_lib::Token::new(value.channel()),
        work: run_work(value.work(), limits)?,
        since: Wall::from_nanos(value.since()),
        reported_at: Wall::from_nanos(value.reported_at()),
        started: opt0,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn active_run(value: &w::ActiveRun, limits: &Limits) -> Result<c::ActiveRun, w::Problem> {
    let opt0 = match value.attempt() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.placement() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt2 = match &value.retry() {
        Some(item) => Some(retry_notice(item)),
        None => None,
    };
    let opt3 = match &value.observation() {
        Some(item) => Some(run_observation(item, limits)?),
        None => None,
    };
    Ok(c::ActiveRun {
        stage: run_stage(value.stage()),
        phase_since: Wall::from_nanos(value.phase_since()),
        attempt: opt0,
        placement: opt1,
        retry: opt2,
        observation: opt3,
    })
}

pub(crate) fn proposer(value: &w::Proposer) -> c::Proposer {
    match value {
        w::Proposer::Task(row) => c::Proposer::Task(row.value()),
        w::Proposer::Party(row) => c::Proposer::Party(row.value()),
    }
}

pub(crate) fn proposal_key(value: &w::ProposalKey) -> c::ProposalKey {
    c::ProposalKey { proposer: proposer(value.proposer()), number: value.number() }
}

pub(crate) fn recipient(value: &w::Recipient) -> c::Recipient {
    match value {
        w::Recipient::Task(row) => c::Recipient::Task(row.value()),
        w::Recipient::Party(row) => c::Recipient::Party(row.value()),
        w::Recipient::Role(row) => c::Recipient::Role { project: row.project(), role: row.role() },
        w::Recipient::Roles(row) => {
            let mut list0 = List::with_capacity(row.roles().len());
            for item in row.roles() {
                list0.push(*item).expect("reserved supplied list");
            }
            c::Recipient::Roles { project: row.project(), roles: list0 }
        }
    }
}
