//! Landing view records (temper/docs/design/domain/landing.md, 14).
use super::{forge, maps, w};
use alloc::boxed::Box;
use temper_client_protocol_wire::v1::Record;

pub(super) fn train(value: forge::LandingTrain, limits: &w::Limits) -> Option<w::Train> {
    {
        let value = w::Train {
            forge: value.repository.forge,
            repository: value.repository.repository,
            owner: value.owner,
            name: value.name,
            branch: value.branch,
        };
        value.measure(limits).ok()?;
        Some(value)
    }
}
pub(super) fn left(value: forge::LandingLeave, limits: &w::Limits) -> Option<w::LandingLeave> {
    let value = {
        Some(match value {
            forge::LandingLeave::Conflict | forge::LandingLeave::EarlierConflict => w::LandingLeave::Conflict,
            forge::LandingLeave::Unit { .. } | forge::LandingLeave::Dependency { .. } | forge::LandingLeave::Held => {
                w::LandingLeave::Held
            }
            forge::LandingLeave::HeadMoved { head } => w::LandingLeave::HeadMoved(w::MovedHead { head }),
            forge::LandingLeave::Cancelled => w::LandingLeave::Cancelled,
            forge::LandingLeave::Refused { gate } => w::LandingLeave::Refused(w::RefusedGate { gate }),
            forge::LandingLeave::Failed { gate, run } => w::LandingLeave::Failed(w::FailedGate { gate, run }),
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
    {
        let value = w::LandingSearch {
            green: bound(value, search.green, limits)?,
            red: bound(value, search.red, limits)?,
            probe: value.probe,
            tests: maps::search_tests_to_record(&search.tests).ok()?,
        };
        value.measure(limits).ok()?;
        Some(value)
    }
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
    {
        let value = w::SearchBound { member, prefix };
        value.measure(limits).ok()?;
        Some(value)
    }
}
pub(super) fn judgement(value: &forge::PrefixJudgement, limits: &w::Limits) -> Option<w::PrefixJudgement> {
    let record = maps::prefix_judgement_to_record(value).ok()?;
    record.measure(limits).ok()?;
    Some(record)
}

// Prefix evidence omits the change gate's kind.
pub(crate) fn prefix_gate_to_record(value: &forge::PanelGate) -> w::PrefixGate {
    w::PrefixGate { number: value.number, blocking: value.blocking, verdict: maps::ci_to_record(&value.verdict) }
}

pub(super) fn hold(
    value: &forge::LandingPhase,
    read_hold: Option<Box<[u8]>>,
    limits: &w::Limits,
) -> Option<w::LandingHold> {
    let value = {
        let forge::LandingPhase::Held { why, .. } = value else { return None };
        Some(match *why {
            forge::LandingHold::Faults { member, gate } => w::LandingHold::Faults(w::HeldFaults { member, gate }),
            forge::LandingHold::Prepare => w::LandingHold::Prepare,
            forge::LandingHold::Record { member } => w::LandingHold::Record(w::HeldRecord { member }),
            forge::LandingHold::Timeout { member, gate, run } => {
                w::LandingHold::Timeout(w::HeldTimeout { member, gate, run })
            }
            forge::LandingHold::Turn => w::LandingHold::Turn,
            forge::LandingHold::Read => w::LandingHold::Read(w::HeldRead { reason: read_hold? }),
        })
    }?;
    value.measure(limits).ok()?;
    Some(value)
}

pub(super) fn wait(value: &forge::LandingWait, limits: &w::Limits) -> Option<w::LandingWait> {
    let wait = maps::landing_wait_to_record(value);
    wait.measure(limits).ok()?;
    Some(wait)
}
