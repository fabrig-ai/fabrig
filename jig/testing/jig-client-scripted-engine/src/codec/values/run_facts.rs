//! Wire projections of run facts facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, w};
use super::{wire_call_identity, wire_failure_class, wire_hold_failure};
pub(in crate::codec) fn wire_run_stage(value: &c::RunStage, limits: &w::Limits) -> Result<w::RunStage, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunStage::Idle => w::RunStage::Idle,
        c::RunStage::Due => w::RunStage::Due,
        c::RunStage::Preparing => w::RunStage::Preparing,
        c::RunStage::Claimed { attempt } => {
            w::RunStage::Claimed(w::RunStageClaimed::new(limits, w::RunStageClaimedParts { attempt: *(attempt) })?)
        }
        c::RunStage::Running { attempt } => {
            w::RunStage::Running(w::RunStageRunning::new(limits, w::RunStageRunningParts { attempt: *(attempt) })?)
        }
        c::RunStage::BackingOff { until } => w::RunStage::BackingOff(w::RunStageBackingOff::new(
            limits,
            w::RunStageBackingOffParts { until: (until).as_nanos() },
        )?),
    })
}

pub(in crate::codec) fn wire_closing_stage(
    value: &c::ClosingStage,
    limits: &w::Limits,
) -> Result<w::ClosingStage, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ClosingStage::Run { attempt } => {
            w::ClosingStage::Run(w::ClosingStageRun::new(limits, w::ClosingStageRunParts { attempt: *(attempt) })?)
        }
        c::ClosingStage::Delegates => w::ClosingStage::Delegates,
        c::ClosingStage::Effects => w::ClosingStage::Effects,
        c::ClosingStage::Releases => w::ClosingStage::Releases,
        c::ClosingStage::Settled => w::ClosingStage::Settled,
    })
}

pub(in crate::codec) fn wire_active_run(value: &c::ActiveRun, limits: &w::Limits) -> Result<w::ActiveRun, w::Problem> {
    w::ActiveRun::new(
        limits,
        w::ActiveRunParts {
            stage: wire_run_stage(&value.stage, limits)?,
            phase_since: value.phase_since.as_nanos(),
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            placement: value.placement.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            retry: value.retry.as_ref().map(|item| wire_retry_notice(item, limits)).transpose()?,
            observation: value.observation.as_ref().map(|item| wire_run_observation(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_retry_notice(
    value: &c::RetryNotice,
    limits: &w::Limits,
) -> Result<w::RetryNotice, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RetryNotice::OwnerFailed { class, count, allowance } => {
            w::RetryNotice::OwnerFailed(w::RetryNoticeOwnerFailed::new(
                limits,
                w::RetryNoticeOwnerFailedParts {
                    class: wire_hold_failure(class, limits)?,
                    count: *(count),
                    allowance: *(allowance),
                },
            )?)
        }
        c::RetryNotice::Failed { class, count, allowance } => w::RetryNotice::Failed(w::RetryNoticeFailed::new(
            limits,
            w::RetryNoticeFailedParts {
                class: wire_failure_class(class, limits)?,
                count: *(count),
                allowance: *(allowance),
            },
        )?),
        c::RetryNotice::Refused { count } => {
            w::RetryNotice::Refused(w::RetryNoticeRefused::new(limits, w::RetryNoticeRefusedParts { count: *(count) })?)
        }
    })
}

pub(in crate::codec) fn wire_run_observation(
    value: &c::RunObservation,
    limits: &w::Limits,
) -> Result<w::RunObservation, w::Problem> {
    w::RunObservation::new(
        limits,
        w::RunObservationParts {
            channel: value.channel.raw(),
            work: wire_run_work(&value.work, limits)?,
            since: value.since.as_nanos(),
            reported_at: value.reported_at.as_nanos(),
            started: value.started.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_run_work(value: &c::RunWork, limits: &w::Limits) -> Result<w::RunWork, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunWork::Preparing => w::RunWork::Preparing,
        c::RunWork::Starting => w::RunWork::Starting,
        c::RunWork::Active => w::RunWork::Active,
        c::RunWork::Writing => w::RunWork::Writing,
        c::RunWork::Tool { call, tool } => w::RunWork::Tool(w::RunWorkTool::new(
            limits,
            w::RunWorkToolParts { call: wire_call_identity(call, limits)?, tool: bytes::copy_of((tool).bytes()) },
        )?),
        c::RunWork::Waiting => w::RunWork::Waiting,
        c::RunWork::Ending => w::RunWork::Ending,
    })
}

pub(in crate::codec) fn wire_held_runtime_fields(
    value: &c::HeldRuntime,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::HeldRuntimeFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HeldRuntime::Waiting => w::HeldRuntimeFields::Waiting,
        c::HeldRuntime::Active(value) => w::HeldRuntimeFields::Active(w::HeldRuntimeActiveFields::new(
            limits,
            w::HeldRuntimeActiveFieldsParts { value: wire_active_run(value, limits)? },
        )?),
        c::HeldRuntime::Closing(value) => w::HeldRuntimeFields::Closing(w::HeldRuntimeClosingFields::new(
            limits,
            w::HeldRuntimeClosingFieldsParts { value: facts.closing(value, Some(parent), limits)? },
        )?),
    })
}
