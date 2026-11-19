//! Landing view records (temper/docs/design/domain/landing.md, 14).
use temper_client_protocol_wire::v1::Record;
use super::{forge, w};
use skein_lib::List;
use temper_engine_domain_forge_change::Status;

pub(super) fn train(value: forge::LandingTrain, limits: &w::Limits) -> Option<w::Train> {
    { let value = w::Train {
            forge: value.repository.forge,
            repository: value.repository.repository,
            owner: value.owner,
            name: value.name,
            branch: value.branch,
        }; value.measure(limits).ok()?; Some(value) }
}
pub(super) fn step(value: forge::LandingStep) -> w::LandingStep {
    match value {
        forge::LandingStep::Waiting => w::LandingStep::Waiting,
        forge::LandingStep::Preparing => w::LandingStep::Preparing,
        forge::LandingStep::Proving => w::LandingStep::Proving,
        forge::LandingStep::Probing => w::LandingStep::Probing,
        forge::LandingStep::ProbingByTest => w::LandingStep::ProbingByTest,
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
    let value = {
    Some(match value {
        forge::LandingLeave::Conflict | forge::LandingLeave::EarlierConflict => w::LandingLeave::Conflict,
        forge::LandingLeave::Unit { .. } | forge::LandingLeave::Dependency { .. } | forge::LandingLeave::Held => {
            w::LandingLeave::Held
        }
        forge::LandingLeave::HeadMoved { head } => {
            w::LandingLeave::HeadMoved(w::MovedHead { head })
        }
        forge::LandingLeave::Cancelled => w::LandingLeave::Cancelled,
        forge::LandingLeave::Refused { gate } => {
            w::LandingLeave::Refused(w::RefusedGate { gate })
        }
        forge::LandingLeave::Failed { gate, run } => {
            w::LandingLeave::Failed(w::FailedGate { gate, run })
        }
        forge::LandingLeave::Blamed { gate, run, red, green } => {
            w::LandingLeave::Blamed(w::BlamedGate { gate, run, red, green })
        }
    })
}?;
    value.measure(limits).ok()?;
    Some(value)
}
pub(super) fn search(value: &forge::LandingView, limits: &w::Limits) -> Option<w::LandingSearch> {
    let search = value.round.search.as_ref()?;
    { let value = w::LandingSearch {
            green: bound(value, search.green, limits)?,
            red: bound(value, search.red, limits)?,
            probe: value.probe,
            tests: search_tests(&search.tests, limits)?,
        }; value.measure(limits).ok()?; Some(value) }
}
fn search_tests(value: &forge::LandingSearchTests, limits: &w::Limits) -> Option<w::SearchTests> {
    let value = {
    Some(match value {
        forge::LandingSearchTests::Whole => w::SearchTests::Whole,
        forge::LandingSearchTests::WholeAgain => w::SearchTests::WholeAgain,
        forge::LandingSearchTests::ByTest { gates, passes } => {
            let mut selections = List::with_capacity(u32::try_from(gates.len()).ok()?);
            for gate in gates {
                selections
                    .push(
                        w::SearchSelection {
                                gate: gate.gate,
                                name: gate.name.clone(),
                                count: gate.count,
                                ids: List::from_boxed(gate.ids.clone()),
                            },
                    )
                    .ok()?;
            }
            let mut probed = List::with_capacity(u32::try_from(passes.len()).ok()?);
            for pass in passes {
                probed
                    .push(
                        w::SearchPass {
                                member: pass.member,
                                prefix: pass.prefix,
                                gate: pass.gate,
                                name: pass.name.clone(),
                                run: pass.run,
                            },
                    )
                    .ok()?;
            }
            w::SearchTests::ByTest(
                w::SearchByTest { gates: selections, passes: probed },
            )
        }
    })
}?;
    value.measure(limits).ok()?;
    Some(value)
}
fn bound(value: &forge::LandingView, member: u16, limits: &w::Limits) -> Option<w::SearchBound> {
    let prefix = if member == 0 {
        match value.round.upstream {
            Some(base) => base,
            None => {
                let red = value.round.search.as_ref()?.red.checked_sub(1)?;
                let member = value.members.get(usize::from(red))?;
                if let Some(base) = member.base {
                    base
                } else if let Some(forge::LandingLeave::Blamed { green, .. }) = member.left {
                    green
                } else {
                    return None;
                }
            }
        }
    } else {
        let member = value.members.get(usize::from(member.checked_sub(1)?))?;
        if let Some(prefix) = member.candidate {
            prefix
        } else if let Some(forge::LandingLeave::Blamed { red, .. }) = member.left {
            red
        } else {
            return None;
        }
    };
    { let value = w::SearchBound { member, prefix }; value.measure(limits).ok()?; Some(value) }
}
pub(super) fn judgement(value: forge::PrefixJudgement, limits: &w::Limits) -> Option<w::PrefixJudgement> {
    let mut gates = List::with_capacity(u32::try_from(value.gates.len()).ok()?);
    for gate in value.gates {
        gates
            .push(
                w::PrefixGate { number: gate.number, blocking: gate.blocking, verdict: status(gate.verdict) },
            )
            .ok()?;
    }
    { let value = w::PrefixJudgement { prefix: value.prefix, verdict: status(value.verdict), gates }; value.measure(limits).ok()?; Some(value) }
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
    let value = {
    let forge::LandingPhase::Held { why, .. } = value else { return None };
    Some(match *why {
        forge::LandingHold::Faults { member, gate } => {
            w::LandingHold::Faults(w::HeldFaults { member, gate })
        }
        forge::LandingHold::Prepare => w::LandingHold::Prepare,
        forge::LandingHold::Record { member } => {
            w::LandingHold::Record(w::HeldRecord { member })
        }
        forge::LandingHold::Timeout { member, gate, run } => {
            w::LandingHold::Timeout(w::HeldTimeout { member, gate, run })
        }
        forge::LandingHold::Turn => w::LandingHold::Turn,
    })
}?;
    value.measure(limits).ok()?;
    Some(value)
}
