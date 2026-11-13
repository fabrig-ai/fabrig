//! Landing view records (temper/docs/design/domain/landing.md, 14).
use super::{forge, w};
use skein_lib::List;
use temper_engine_domain_forge_change::Status;

pub(super) fn train(value: forge::LandingTrain, limits: &w::Limits) -> Option<w::Train> {
    w::Train::new(
        limits,
        w::TrainParts {
            forge: value.repository.forge,
            repository: value.repository.repository,
            owner: value.owner,
            name: value.name,
            branch: value.branch,
        },
    )
    .ok()
}
pub(super) fn step(value: forge::LandingStep) -> w::LandingStep {
    match value {
        forge::LandingStep::Waiting => w::LandingStep::Waiting,
        forge::LandingStep::Preparing => w::LandingStep::Preparing,
        forge::LandingStep::Proving => w::LandingStep::Proving,
        forge::LandingStep::Proved => w::LandingStep::Proved,
        forge::LandingStep::Turn => w::LandingStep::Turn,
        forge::LandingStep::Landed => w::LandingStep::Landed,
        forge::LandingStep::Held => w::LandingStep::Held,
        forge::LandingStep::Cancelled => w::LandingStep::Cancelled,
        forge::LandingStep::Judging => w::LandingStep::Judging,
        forge::LandingStep::Leaving => w::LandingStep::Leaving,
        forge::LandingStep::Recording => w::LandingStep::Recording,
        forge::LandingStep::Telling => w::LandingStep::Telling,
    }
}
pub(super) fn combined(value: forge::LandingCombine) -> w::Combined {
    match value {
        forge::LandingCombine::Merge => w::Combined::Merge,
        forge::LandingCombine::Squash => w::Combined::Squash,
    }
}
pub(super) fn left(value: forge::LandingLeave, limits: &w::Limits) -> Option<w::LandingLeave> {
    Some(match value {
        forge::LandingLeave::Conflict => w::LandingLeave::Conflict,
        forge::LandingLeave::EarlierConflict => w::LandingLeave::Conflict,
        forge::LandingLeave::Unit { .. }
        | forge::LandingLeave::Dependency { .. }
        | forge::LandingLeave::Fallback { .. } => w::LandingLeave::Held,
        forge::LandingLeave::HeadMoved { head } => {
            w::LandingLeave::HeadMoved(w::MovedHead::new(limits, w::MovedHeadParts { head }).ok()?)
        }
        forge::LandingLeave::Cancelled => w::LandingLeave::Cancelled,
        forge::LandingLeave::Held => w::LandingLeave::Held,
        forge::LandingLeave::Refused { gate } => {
            w::LandingLeave::Refused(w::RefusedGate::new(limits, w::RefusedGateParts { gate }).ok()?)
        }
        forge::LandingLeave::Failed { gate, run } => {
            w::LandingLeave::Failed(w::FailedGate::new(limits, w::FailedGateParts { gate, run }).ok()?)
        }
    })
}
pub(super) fn judgement(value: forge::PrefixJudgement, limits: &w::Limits) -> Option<w::PrefixJudgement> {
    let mut gates = List::with_capacity(u32::try_from(value.gates.len()).ok()?);
    for gate in value.gates {
        gates
            .push(
                w::PrefixGate::new(
                    limits,
                    w::PrefixGateParts { number: gate.number, blocking: gate.blocking, verdict: status(gate.verdict) },
                )
                .ok()?,
            )
            .ok()?;
    }
    w::PrefixJudgement::new(
        limits,
        w::PrefixJudgementParts { prefix: value.prefix, verdict: status(value.verdict), gates },
    )
    .ok()
}
fn status(value: Status) -> w::Ci {
    match value {
        Status::Unknown => w::Ci::Unknown,
        Status::Pending => w::Ci::Pending,
        Status::Passed => w::Ci::Passed,
        Status::Failed => w::Ci::Failed,
    }
}
pub(super) fn hold(value: &forge::LandingPhase, limits: &w::Limits) -> Option<w::LandingHold> {
    let forge::LandingPhase::Held { why, .. } = value else { return None };
    Some(match *why {
        forge::LandingHold::Faults { member, gate } => {
            w::LandingHold::Faults(w::HeldFaults::new(limits, w::HeldFaultsParts { member, gate }).ok()?)
        }
        forge::LandingHold::Prepare => w::LandingHold::Prepare,
        forge::LandingHold::Record { member } => {
            w::LandingHold::Record(w::HeldRecord::new(limits, w::HeldRecordParts { member }).ok()?)
        }
        forge::LandingHold::Timeout { member, gate, run } => {
            w::LandingHold::Timeout(w::HeldTimeout::new(limits, w::HeldTimeoutParts { member, gate, run }).ok()?)
        }
        forge::LandingHold::Turn => w::LandingHold::Turn,
    })
}
