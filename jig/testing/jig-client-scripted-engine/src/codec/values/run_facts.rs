//! Wire projections of run facts facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, w};
use super::{wire_call_identity, wire_failure_class, wire_hold_failure};
pub(in crate::codec) fn wire_run_stage(value: &c::RunStage) -> w::RunStage {
    match value {
        c::RunStage::Idle => w::RunStage::Idle,
        c::RunStage::Due => w::RunStage::Due,
        c::RunStage::Preparing => w::RunStage::Preparing,
        c::RunStage::Claimed { attempt } => w::RunStage::Claimed(w::RunStageClaimed { attempt: *(attempt) }),
        c::RunStage::Running { attempt } => w::RunStage::Running(w::RunStageRunning { attempt: *(attempt) }),
        c::RunStage::BackingOff { until } => {
            w::RunStage::BackingOff(w::RunStageBackingOff { until: (until).as_nanos() })
        }
    }
}

pub(in crate::codec) fn wire_closing_stage(value: &c::ClosingStage) -> w::ClosingStage {
    match value {
        c::ClosingStage::Run { attempt } => w::ClosingStage::Run(w::ClosingStageRun { attempt: *(attempt) }),
        c::ClosingStage::Delegates => w::ClosingStage::Delegates,
        c::ClosingStage::Effects => w::ClosingStage::Effects,
        c::ClosingStage::Releases => w::ClosingStage::Releases,
        c::ClosingStage::Settled => w::ClosingStage::Settled,
    }
}

pub(in crate::codec) fn wire_active_run(value: &c::ActiveRun) -> w::ActiveRun {
    w::ActiveRun {
        stage: wire_run_stage(&value.stage),
        phase_since: value.phase_since.as_nanos(),
        attempt: value.attempt,
        placement: value.placement.as_ref().map(|item| (item).as_nanos()),
        retry: value.retry.as_ref().map(wire_retry_notice),
        observation: value.observation.as_ref().map(wire_run_observation),
    }
}

pub(in crate::codec) fn wire_retry_notice(value: &c::RetryNotice) -> w::RetryNotice {
    match value {
        c::RetryNotice::OwnerFailed { class, count, allowance, reason } => {
            w::RetryNotice::OwnerFailed(w::RetryNoticeOwnerFailed {
                class: wire_hold_failure(class),
                count: *(count),
                allowance: *(allowance),
                reason: reason.as_ref().map(|value| bytes::copy_of(value.bytes())),
            })
        }
        c::RetryNotice::Failed { class, count, allowance, reason } => w::RetryNotice::Failed(w::RetryNoticeFailed {
            class: wire_failure_class(class),
            count: *(count),
            allowance: *(allowance),
            reason: reason.as_ref().map(|value| bytes::copy_of(value.bytes())),
        }),
        c::RetryNotice::Refused { count, reason } => w::RetryNotice::Refused(w::RetryNoticeRefused {
            count: *(count),
            reason: reason.as_ref().map(|value| bytes::copy_of(value.bytes())),
        }),
    }
}

pub(in crate::codec) fn wire_run_observation(value: &c::RunObservation) -> w::RunObservation {
    w::RunObservation {
        channel: value.channel.raw(),
        work: wire_run_work(&value.work),
        since: value.since.as_nanos(),
        reported_at: value.reported_at.as_nanos(),
        started: value.started.as_ref().map(|item| (item).as_nanos()),
    }
}

pub(in crate::codec) fn wire_run_work(value: &c::RunWork) -> w::RunWork {
    match value {
        c::RunWork::Preparing => w::RunWork::Preparing,
        c::RunWork::Starting => w::RunWork::Starting,
        c::RunWork::Active => w::RunWork::Active,
        c::RunWork::Writing => w::RunWork::Writing,
        c::RunWork::Tool { call, tool } => {
            w::RunWork::Tool(w::RunWorkTool { call: wire_call_identity(call), tool: bytes::copy_of((tool).bytes()) })
        }
        c::RunWork::Waiting => w::RunWork::Waiting,
        c::RunWork::Ending => w::RunWork::Ending,
    }
}

pub(in crate::codec) fn wire_held_runtime_fields(
    value: &c::HeldRuntime,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::HeldRuntimeFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HeldRuntime::Waiting => w::HeldRuntimeFields::Waiting,
        c::HeldRuntime::Active(value) => {
            w::HeldRuntimeFields::Active(w::HeldRuntimeActiveFields { value: wire_active_run(value) })
        }
        c::HeldRuntime::Closing(value) => w::HeldRuntimeFields::Closing(w::HeldRuntimeClosingFields {
            value: facts.closing(value, Some(parent), limits)?,
        }),
    })
}
