//! One level-triggered decision (domain/landing.md, 3–10).
use crate::{
    Decision, Facts, Flight, GateFacts, GateProof, Hold, Landing, Limits, Member, Phase, Posted, Posting, Project,
    Proof, Status, Stepped, Verdict,
};
use alloc::boxed::Box;
use skein_lib::{List, Wall};

/// Add a member after its upstreams while the landing can still grow
/// (domain/landing.md, 4.1–4.2). Existing candidates and proofs remain valid.
#[must_use]
pub fn join(mut landing: Landing, member: Member, after: &[u64]) -> Option<Landing> {
    match landing.phase {
        Phase::Waiting | Phase::Preparing | Phase::Proving | Phase::Proved { .. } => {}
        Phase::Turn { .. } | Phase::Landed | Phase::Cancelled | Phase::Held { .. } => return None,
    }
    for prior in &landing.members {
        if prior.change == member.change || prior.repository == member.repository {
            return None;
        }
    }
    let mut position = 0;
    for (index, prior) in landing.members.iter().enumerate() {
        if after.contains(&prior.change) {
            position = index.checked_add(1).expect("member position fits usize");
        }
    }
    let mut members = List::with_capacity(
        u32::try_from(landing.members.len().checked_add(1).expect("member count fits usize")).expect("bounded members"),
    );
    for prior in landing.members.get(..position).expect("member position") {
        members.push(prior.clone()).expect("member room");
    }
    members.push(member).expect("new member room");
    for prior in landing.members.get(position..).expect("member position") {
        members.push(prior.clone()).expect("member room");
    }
    landing.members = members.into_boxed();
    landing.phase = Phase::Preparing;
    Some(landing)
}

/// Retain the cancelled member's identity until its round has closed (domain/landing.md, 4.3).
#[must_use]
pub fn leave(mut landing: Landing, change: u64) -> Landing {
    for member in &mut landing.members {
        if member.change == change && member.landed.is_none() {
            member.left = Some(crate::Leave::Cancelled);
        }
    }
    landing
}

fn leaving(mut landing: Landing, member: u16, why: crate::Leave) -> Stepped {
    landing.members.get_mut(usize::from(member)).expect("member").left = Some(why);
    landing.round.ending = Some(crate::RoundEnd::Leave);
    done(landing, Decision::Leave { member, why })
}

/// Decide against a coherent snapshot. The top commits the state and its
/// effect together; effect facts always describe the current preparation/try.
#[must_use]
pub fn step(landing: &Landing, facts: &Facts, project: &Project, limits: &Limits, now: Wall) -> Stepped {
    admitted(landing, facts, limits);
    let mut next = landing.clone();
    if let Some(ending) = next.round.ending {
        return closing(next, facts, ending);
    }
    match &landing.phase {
        Phase::Landed => return done(next, Decision::Finish),
        Phase::Cancelled => return done(next, Decision::None),
        Phase::Held { .. } => {
            if !facts.released {
                return wait(next, None);
            }
            if let Phase::Held { why: Hold::Record { .. }, .. } = landing.phase {
                next.phase = Phase::Turn { at: now };
            } else {
                next.phase = Phase::Preparing;
                next.round.ending = Some(crate::RoundEnd::Stale);
            }
            for member in &mut next.members {
                for gate in &mut member.gates {
                    gate.faults = 0;
                }
            }
            return done(next, Decision::None);
        }
        Phase::Waiting | Phase::Preparing | Phase::Proving | Phase::Proved { .. } | Phase::Turn { .. } => {}
    }
    for (index, (member, fresh)) in next.members.iter().zip(&facts.members).enumerate() {
        if member.landed.is_some() { continue; }
        let why = if let Some(gate) = fresh.refused { Some(crate::Leave::Refused { gate }) }
            else if fresh.cancelled { Some(crate::Leave::Cancelled) }
            else if member.head.is_some() && member.head != Some(fresh.head) {
                Some(crate::Leave::HeadMoved { head: fresh.head })
            } else if fresh.held { Some(crate::Leave::Held) } else { member.left };
        if let Some(why) = why { return leaving(next, position(index), why); }
    }
    match landing.phase {
        Phase::Waiting | Phase::Preparing | Phase::Proving => {
            for (member, fresh) in next.members.iter().zip(&facts.members) {
                if member.landed.is_none() && !fresh.ready {
                    next.phase = Phase::Waiting;
                    if let Some(decision) = failed_post(&mut next, facts) {
                        return done(next, decision);
                    }
                    return wait(next, None);
                }
            }
            if let Some(decision) = failed_post(&mut next, facts) {
                return done(next, decision);
            }
        }
        Phase::Proved { .. } | Phase::Turn { .. } | Phase::Landed | Phase::Held { .. } | Phase::Cancelled => {}
    }
    match landing.phase {
        Phase::Waiting => {
            next.phase = Phase::Preparing;
            done(next, Decision::None)
        }
        Phase::Preparing => preparing(next, facts),
        Phase::Proving => proving(next, facts, *project, limits, now),
        Phase::Proved { at } => {
            if facts.turn_free {
                next.phase = Phase::Turn { at };
                done(next, Decision::None)
            } else {
                wait(next, None)
            }
        }
        Phase::Turn { .. } => turn(next, facts),
        Phase::Landed | Phase::Held { .. } | Phase::Cancelled => unreachable!("inactive phases returned above"),
    }
}

fn admitted(landing: &Landing, facts: &Facts, limits: &Limits) {
    assert!(landing.members.len() <= skein_lib::usize_of(limits.members), "members admitted");
    assert!(landing.members.len() <= usize::from(u16::MAX).saturating_add(1), "member positions fit u16");
    assert_eq!(landing.members.len(), facts.members.len(), "one fact per member");
    for (member, fresh) in landing.members.iter().zip(&facts.members) {
        assert!(member.gates.len() <= skein_lib::usize_of(limits.gates), "gates admitted");
        assert!(member.gates.len() <= usize::from(u16::MAX).saturating_add(1), "gate positions fit u16");
        assert_eq!(member.gates.len(), fresh.gates.len(), "one fact per gate");
    }
}

fn done(landing: Landing, decision: Decision) -> Stepped {
    Stepped { landing, decision }
}
fn wait(landing: Landing, until: Option<Wall>) -> Stepped {
    done(landing, Decision::Wait { until })
}

fn preparing(mut landing: Landing, facts: &Facts) -> Stepped {
    for (index, (member, fresh)) in landing.members.iter_mut().zip(&facts.members).enumerate() {
        if member.landed.is_some() {
            continue;
        }
        if member.base.is_none() {
            landing.preparation = landing.preparation.checked_add(1).expect("round fits u16");
            member.head = Some(fresh.head);
            member.base = Some(fresh.main);
        }
        if member.head != Some(fresh.head) || member.base != Some(fresh.main) {
            landing.round.ending =
                Some(if member.head == Some(fresh.head) { crate::RoundEnd::Stale } else { crate::RoundEnd::Leave });
            return done(landing, Decision::None);
        }
        if fresh.contained == Status::Passed {
            member.landed = Some(fresh.main);
            landing.round.told = true;
            landing.phase = Phase::Landed;
            return done(landing, Decision::Landed { member: position(index), prefix: fresh.main });
        }
        if let Some((candidate, combined)) = fresh.candidate {
            member.candidate = Some(candidate);
            landing.round.combined = Some(combined);
        }
        if member.candidate.is_none() {
            match fresh.preparation {
                crate::Preparation::Running => return wait(landing, None),
                crate::Preparation::Conflict => {
                    return leaving(landing, position(index), crate::Leave::Conflict);
                }
                crate::Preparation::Failed => {
                    let why = Hold::Prepare;
                    landing.phase = Phase::Held { was: Box::new(landing.phase), why };
                    return done(landing, Decision::Hold(why));
                }
                crate::Preparation::Idle => {
                    return done(
                        landing,
                        Decision::Prepare { member: position(index), head: fresh.head, main: fresh.main },
                    );
                }
            }
        }
    }
    if landing.round.pull.is_none() {
        if let Some(pull) = facts.opened {
            landing.round.pull = Some(pull);
        } else if facts.opening == Status::Failed {
            landing.phase = Phase::Held { was: Box::new(landing.phase), why: Hold::Turn };
            return done(landing, Decision::Hold(Hold::Turn));
        } else if facts.opening == Status::Pending {
            return wait(landing, None);
        } else {
            return done(landing, Decision::Open);
        }
    }
    landing.phase = Phase::Proving;
    done(landing, Decision::None)
}

fn closing(mut landing: Landing, facts: &Facts, ending: crate::RoundEnd) -> Stepped {
    if landing.round.pull.is_some() && facts.closed != Status::Passed {
        return if facts.closed == Status::Pending { wait(landing, None) } else { done(landing, Decision::Close) };
    }
    if facts.deleted != Status::Passed {
        return if facts.deleted == Status::Pending { wait(landing, None) } else { done(landing, Decision::Delete) };
    }
    landing.round.pull = None;
    landing.round.combined = None;
    landing.round.ending = None;
    match ending {
        crate::RoundEnd::Stale => {
            landing.stale_proofs = landing.stale_proofs.checked_add(1).expect("stale count fits");
            landing.phase = Phase::Preparing;
            landing.round.judging = false;
            for member in &mut landing.members {
                member.base = None;
                member.candidate = None;
                member.asked = None;
                for gate in &mut member.gates {
                    *gate = GateProof { proof: Proof::Unkeyed, posted: Posted::Nothing, faults: 0 };
                }
            }
            done(landing, Decision::Stale)
        }
        crate::RoundEnd::Conflict => {
            let base = landing.members.first().expect("one member").base.expect("round base");
            landing.phase = Phase::Landed;
            done(landing, Decision::Resolve { member: 0, base })
        }
        crate::RoundEnd::Leave => {
            landing.phase = Phase::Landed;
            done(landing, Decision::Finish)
        }
    }
}

fn position(index: usize) -> u16 {
    u16::try_from(index).expect("position admitted as u16")
}

// A repair may make a member unready before its failure status is posted.
// Posting that known receipt still belongs to its old candidate.
fn failed_post(landing: &mut Landing, facts: &Facts) -> Option<Decision> {
    for (index, (member, fresh)) in landing.members.iter_mut().zip(&facts.members).enumerate() {
        for (number, (gate, fact)) in member.gates.iter_mut().zip(&fresh.gates).enumerate() {
            match gate.proof {
                Proof::Failed { .. } if gate.posted != Posted::Verdict => {
                    if fact.posting == Status::Pending {
                        return Some(Decision::Wait { until: None });
                    }
                    gate.posted = Posted::Verdict;
                    return Some(Decision::Post {
                        member: position(index),
                        gate: position(number),
                        posting: Posting::Ran { passed: false },
                    });
                }
                Proof::Unkeyed
                | Proof::Keyed { .. }
                | Proof::Dispatched { .. }
                | Proof::Passed { .. }
                | Proof::Failed { .. } => {}
            }
        }
    }
    None
}

fn proving(mut landing: Landing, facts: &Facts, project: Project, limits: &Limits, now: Wall) -> Stepped {
    let mut blocked = false;
    let mut poll = false;
    for (index, fresh) in facts.members.iter().enumerate() {
        if let Some(gate) = fresh.refused {
            return leaving(landing, position(index), crate::Leave::Refused { gate });
        }
        if fresh.judgement != Status::Passed {
            blocked = true;
            if !landing.round.judging {
                landing.round.judging = true;
                return done(landing, Decision::Judge { member: position(index) });
            }
        }
    }
    for (index, (member, fresh)) in landing.members.iter_mut().zip(&facts.members).enumerate() {
        if member.landed.is_some() {
            continue;
        }
        let candidate = member.candidate.expect("proving has every candidate");
        for (number, (gate, fact)) in member.gates.iter_mut().zip(&fresh.gates).enumerate() {
            let decision = gate_step(gate, fact, position(index), position(number), candidate, project);
            match decision {
                Decision::None => {}
                Decision::Wait { .. } => {
                    if fact.blocking {
                        blocked = true;
                        match fact.flight {
                            Flight::Mine { .. } => poll = true,
                            Flight::None | Flight::Other => {}
                        }
                    }
                }
                Decision::Hold(why) => {
                    landing.phase = Phase::Held { was: Box::new(landing.phase.clone()), why };
                    return done(landing, decision);
                }
                Decision::Key { .. }
                | Decision::Dispatch { .. }
                | Decision::Retry { .. }
                | Decision::Post { .. }
                => return done(landing, decision),
                Decision::Repair { member, gate } => {
                    let run = facts.members.get(usize::from(member)).expect("member").gates.get(usize::from(gate)).expect("gate").run;
                    return leaving(landing, member, crate::Leave::Failed { gate, run });
                }
                Decision::Prepare { .. }
                | Decision::Leave { .. }
                | Decision::Judge { .. }
                | Decision::Proved
                | Decision::Merge { .. }
                | Decision::Stale
                | Decision::Finish
                | Decision::Cancel
                | Decision::Open
                | Decision::Close
                | Decision::Delete
                | Decision::Record { .. }
                | Decision::Landed { .. }
                | Decision::Resolve { .. } => unreachable!("gate decisions only"),
            }
        }
    }
    if blocked {
        let until =
            if poll { Some(Wall::from_nanos(now.as_nanos().saturating_add(limits.poll.as_nanos()))) } else { None };
        return wait(landing, until);
    }
    landing.phase = Phase::Proved { at: now };
    done(landing, Decision::Proved)
}

fn gate_step(
    gate: &mut GateProof,
    fact: &GateFacts,
    member: u16,
    number: u16,
    candidate: crate::Commit,
    project: Project,
) -> Decision {
    let key = match gate.proof {
        Proof::Unkeyed => match fact.key {
            Some(key) => {
                gate.proof = Proof::Keyed { key };
                key
            }
            None => return Decision::Key { member, gate: number, candidate },
        },
        Proof::Keyed { key } | Proof::Dispatched { key, .. } | Proof::Passed { key } | Proof::Failed { key } => key,
    };
    match fact.receipt {
        Status::Passed => {
            let received = match gate.proof {
                Proof::Dispatched { .. } => false,
                Proof::Keyed { .. } | Proof::Passed { .. } | Proof::Failed { .. } | Proof::Unkeyed => {
                    gate.posted == Posted::Nothing
                }
            };
            gate.proof = Proof::Passed { key };
            if gate.posted != Posted::Verdict {
                if fact.posting == Status::Pending {
                    return Decision::Wait { until: None };
                }
                gate.posted = Posted::Verdict;
                return Decision::Post {
                    member,
                    gate: number,
                    posting: if received { Posting::Received } else { Posting::Ran { passed: true } },
                };
            }
            return Decision::None;
        }
        Status::Failed => {
            let failed = gate.proof == Proof::Failed { key };
            gate.proof = Proof::Failed { key };
            if !failed && fact.blocking {
                return Decision::Repair { member, gate: number };
            }
            if gate.posted != Posted::Verdict {
                if fact.posting == Status::Pending {
                    return Decision::Wait { until: None };
                }
                gate.posted = Posted::Verdict;
                return Decision::Post { member, gate: number, posting: Posting::Ran { passed: false } };
            }
            return Decision::Wait { until: None };
        }
        Status::Unknown | Status::Pending => {}
    }
    match gate.proof {
        Proof::Passed { .. } => return Decision::None,
        Proof::Failed { .. } => return Decision::Wait { until: None },
        Proof::Dispatched { .. } if gate.posted == Posted::Nothing => {
            gate.posted = Posted::Pending;
            return Decision::Post { member, gate: number, posting: Posting::Pending };
        }
        Proof::Unkeyed | Proof::Keyed { .. } | Proof::Dispatched { .. } => {}
    }
    if let Some(run) = fact.expired {
        return Decision::Hold(Hold::Timeout { member, gate: number, run });
    }
    match fact.flight {
        Flight::Other => Decision::Wait { until: None },
        Flight::None => match gate.proof {
            Proof::Keyed { .. } => {
                gate.proof = Proof::Dispatched { key, try_: 1 };
                Decision::Dispatch { member, gate: number, key, try_: 1 }
            }
            Proof::Unkeyed | Proof::Dispatched { .. } | Proof::Passed { .. } | Proof::Failed { .. } => {
                Decision::Wait { until: None }
            }
        },
        Flight::Mine { run: None | Some(Verdict::Running | Verdict::Passed | Verdict::Failed) } => {
            Decision::Wait { until: None }
        }
        Flight::Mine { run: Some(Verdict::Fault(_)) } => {
            if gate.faults >= project.gate_retries {
                if fact.blocking {
                    return Decision::Hold(Hold::Faults { member, gate: number });
                }
                return Decision::Wait { until: None };
            }
            let try_ = match gate.proof {
                Proof::Dispatched { try_, .. } => try_.checked_add(1).expect("gate try fits u16"),
                Proof::Unkeyed | Proof::Keyed { .. } | Proof::Passed { .. } | Proof::Failed { .. } => 1,
            };
            gate.faults = gate.faults.checked_add(1).expect("faults below retry limit");
            gate.proof = Proof::Dispatched { key, try_ };
            Decision::Retry { member, gate: number, key, try_ }
        }
    }
}

fn turn(mut landing: Landing, facts: &Facts) -> Stepped {
    for (index, (member, fresh)) in landing.members.iter_mut().zip(&facts.members).enumerate() {
        let candidate = member.candidate.expect("proved candidate");
        if fresh.main == candidate {
            member.landed = Some(candidate);
        }
        if let Some(prefix) = member.landed {
            if landing.round.combined == Some(crate::Combine::Squash) && !landing.round.recorded {
                match fresh.record {
                    Status::Passed => landing.round.recorded = true,
                    Status::Unknown => return done(landing, Decision::Record { member: position(index), prefix }),
                    Status::Pending => return wait(landing, None),
                    Status::Failed => {
                        let why = Hold::Record { member: position(index) };
                        landing.phase = Phase::Held { was: Box::new(landing.phase), why };
                        return done(landing, Decision::Hold(why));
                    }
                }
            }
            if !landing.round.told {
                landing.round.told = true;
                return done(landing, Decision::Landed { member: position(index), prefix });
            }
            continue;
        }
        if member.head == Some(fresh.head) && member.base == Some(fresh.main) && fresh.merge == Status::Failed {
            landing.phase = Phase::Held { was: Box::new(landing.phase), why: Hold::Turn };
            return done(landing, Decision::Hold(Hold::Turn));
        }
        if member.head != Some(fresh.head) || member.base != Some(fresh.main) {
            landing.round.ending =
                Some(if member.head == Some(fresh.head) { crate::RoundEnd::Stale } else { crate::RoundEnd::Leave });
            return done(landing, Decision::None);
        }
        return match fresh.merge {
            Status::Unknown => done(landing, Decision::Merge { member: position(index), candidate }),
            Status::Pending | Status::Passed => wait(landing, None),
            Status::Failed => unreachable!("refused merges ended above"),
        };
    }
    landing.phase = Phase::Landed;
    done(landing, Decision::Finish)
}
