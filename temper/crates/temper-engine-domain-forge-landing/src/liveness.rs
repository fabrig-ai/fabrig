//! The check after a landing's decision (temper/docs/design/domain/landing-liveness.md, 2–3).
use crate::{Awaited, Decision, Facts, Landing, Phase, Preparation, RoundPull, Status, Stepped, Stuck};

pub(crate) fn members_and_train(before: &Landing, landing: &Landing, facts: &Facts) -> Option<Stuck> {
    // The legacy single path can remove cancelled members. The remaining facts
    // are then a subsequence in the same order.
    let reduced = landing.members.len() < facts.members.len();
    let mut members = landing.members.iter();
    for (prior, fresh) in before.members.iter().zip(&facts.members) {
        if reduced && fresh.cancelled && crate::domain::active(prior) {
            continue;
        }
        let member = members.next().expect("one fact per member");
        if member.left.is_none() && member.landed.is_none() && (fresh.cancelled || fresh.held) {
            return Some(Stuck::Member { change: member.change, held: !fresh.cancelled });
        }
    }
    if !landing.round.branch.is_empty()
        && let Some(landing) = facts.train_holder
    {
        return Some(Stuck::Train { landing });
    }
    None
}

pub(crate) fn check(before: &Landing, step: &Stepped, facts: &Facts) -> Option<Stuck> {
    let landing = &step.landing;
    match landing.phase {
        Phase::Held { .. } | Phase::Landed | Phase::Cancelled => return None,
        Phase::Waiting | Phase::Preparing | Phase::Proving | Phase::Proved { .. } | Phase::Turn { .. } => {}
    }
    if step.decision == Decision::Finish {
        return None;
    }
    if let Some(cause) = members_and_train(before, landing, facts) {
        return Some(cause);
    }
    if let Decision::Wait { until: None, on } = step.decision
        && !will_wake(landing, facts, on)
    {
        return Some(Stuck::Wake { on });
    }
    None
}

fn will_wake(landing: &Landing, facts: &Facts, on: Awaited) -> bool {
    match on {
        Awaited::Ready { member } | Awaited::Judgement { member } => {
            let index = usize::from(member);
            if let Some(member) = landing.members.get(index)
                && let Some(fresh) = facts.members.get(index)
            {
                member.left.is_none() && member.landed.is_none() && !fresh.held && !fresh.cancelled
            } else {
                false
            }
        }
        Awaited::Turn => landing.round.branch.is_empty() && !facts.turn_free,
        Awaited::Prepare => facts.preparation == Preparation::Running,
        Awaited::Open => facts.opening == Status::Pending,
        Awaited::Close => facts.closed == Status::Pending,
        Awaited::Delete { member } => {
            if let Some(fresh) = facts.members.get(usize::from(member)) {
                fresh.deleted == Status::Pending
            } else {
                false
            }
        }
        Awaited::Post { member, gate } => {
            if let Some(fresh) = facts.members.get(usize::from(member))
                && let Some(gate) = fresh.gates.get(usize::from(gate))
            {
                gate.posting == Status::Pending
            } else {
                false
            }
        }
        Awaited::Record { member } => {
            if let Some(fresh) = facts.members.get(usize::from(member)) {
                fresh.record == Status::Pending
            } else {
                false
            }
        }
        Awaited::Merge => {
            if !crate::domain::matches_turn(&landing.phase) {
                return false;
            }
            if landing.round.branch.is_empty() {
                for (member, fresh) in landing.members.iter().zip(&facts.members) {
                    if member.landed.is_none() {
                        return fresh.merge == Status::Pending;
                    }
                }
                return false;
            }
            if let Some(top) = crate::domain::top(landing)
                && let Some(fresh) = facts.members.get(usize::from(top))
            {
                fresh.merge == Status::Pending
                    || (fresh.merge == Status::Failed && facts.round_pull == RoundPull::Unread)
            } else {
                false
            }
        }
        // A held row does not run the check; a live row cannot await release.
        Awaited::Release
        | Awaited::Prefixes
        | Awaited::Run { .. }
        | Awaited::Receipt { .. }
        | Awaited::Ancestry { .. }
        | Awaited::Read => false,
    }
}
