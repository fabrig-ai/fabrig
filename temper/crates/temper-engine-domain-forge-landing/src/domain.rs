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
    if landing.round.branch.is_empty() {
        return crate::single::leave(landing, change);
    }
    for (index, member) in landing.members.iter_mut().enumerate() {
        if member.change == change && member.landed.is_none() && member.left.is_none() {
            landing.last_left = Some(position(index));
            member.left = Some(crate::Leave::Cancelled);
        }
    }
    landing
}

// Leaves are committed as a closure before the first notification is emitted.
fn cascade(landing: &mut Landing, facts: &Facts) {
    for _pass in 0..landing.members.len() {
        for index in 0..landing.members.len() {
            let member = landing.members.get(index).expect("member");
            if member.left.is_some() || member.landed.is_some() {
                continue;
            }
            let mut why = None;
            for prior in &landing.members {
                if prior.left.is_some() && prior.unit == member.unit {
                    why = Some(crate::Leave::Unit { change: prior.change });
                    break;
                }
            }
            if why.is_none() {
                for dependency in &facts.members.get(index).expect("facts").dependencies {
                    let prior = landing.members.get(usize::from(*dependency)).expect("dependency admitted");
                    if prior.left.is_some() {
                        why = Some(crate::Leave::Dependency { change: prior.change });
                        break;
                    }
                }
            }
            if let Some(why) = why {
                landing.members.get_mut(index).expect("member").left = Some(why);
            }
        }
    }
    for (index, member) in landing.members.iter().enumerate() {
        if member.left.is_some() && !blamed(member.left) && !member.leave_told && member.candidate.is_some() {
            changed(&mut landing.round, position(index));
        }
    }
}

fn changed(round: &mut crate::Round, member: u16) {
    round.ending = Some(crate::RoundEnd::Leave);
    round.changed = Some(match round.changed {
        Some(prior) => prior.min(member),
        None => member,
    });
}

fn notify_leave(landing: &mut Landing) -> Option<Decision> {
    for (index, member) in landing.members.iter_mut().enumerate() {
        if !member.leave_told
            && let Some(why) = member.left
        {
            member.leave_told = true;
            return Some(Decision::Leave { member: position(index), why });
        }
    }
    None
}

fn leaving(mut landing: Landing, facts: &Facts, member: u16, why: crate::Leave) -> Stepped {
    landing.members.get_mut(usize::from(member)).expect("member").left = Some(why);
    cascade(&mut landing, facts);
    let decision = notify_leave(&mut landing).expect("a new leave");
    done(landing, decision)
}

#[expect(clippy::match_like_matches_macro, reason = "step policy uses no matches macro")]
fn blamed(why: Option<crate::Leave>) -> bool {
    if let Some(crate::Leave::Blamed { .. }) = why { true } else { false }
}

fn active(member: &Member) -> bool {
    member.left.is_none() && member.landed.is_none()
}

pub(crate) fn matches_turn(phase: &Phase) -> bool {
    match phase {
        Phase::Turn { .. } => true,
        Phase::Waiting
        | Phase::Preparing
        | Phase::Proving
        | Phase::Proved { .. }
        | Phase::Landed
        | Phase::Held { .. }
        | Phase::Cancelled => false,
    }
}

fn top(landing: &Landing) -> Option<u16> {
    if let Some(search) = &landing.round.search
        && probe(landing).is_none()
        && search.green != 0
    {
        let index = search.green.checked_sub(1).expect("green above base");
        if active(landing.members.get(usize::from(index)).expect("green")) {
            return Some(index);
        }
        return None;
    }
    let mut top = None;
    for (index, member) in landing.members.iter().enumerate() {
        if active(member) && member.candidate.is_some() {
            top = Some(position(index));
        }
    }
    top
}

/// The midpoint unit boundary's member, counted from 0 for decisions.
/// Bounds in the stored search count members from 1, with 0 the base.
#[must_use]
pub fn probe(landing: &Landing) -> Option<u16> {
    let search = landing.round.search.as_ref()?;
    let mut boundaries = List::with_capacity(u32::try_from(landing.members.len()).expect("members"));
    for (index, member) in landing.members.iter().enumerate() {
        let number = position(index).checked_add(1).expect("member number");
        if number <= search.green || number >= search.red || member.candidate.is_none() {
            continue;
        }
        let next = landing.members.get(index.checked_add(1).expect("member bound"));
        if let Some(next) = next
            && next.candidate.is_some()
            && next.unit == member.unit
        {
            continue;
        }
        boundaries.push(position(index)).expect("boundaries");
    }
    if boundaries.is_empty() {
        return None;
    }
    // With m internal boundaries there are m+1 units between the bounds.
    let middle = boundaries
        .len()
        .checked_add(1)
        .expect("bounded")
        .checked_div(2)
        .expect("two")
        .checked_sub(1)
        .expect("an internal boundary");
    boundaries.as_slice().get(skein_lib::usize_of(middle)).copied()
}

fn open(mut landing: Landing, facts: &Facts) -> Stepped {
    if let Some(pull) = facts.opened {
        landing.round.pull = Some(pull);
        done(landing, Decision::None)
    } else if facts.opening == Status::Failed {
        landing.phase = Phase::Held { was: Box::new(landing.phase), why: Hold::Turn };
        done(landing, Decision::Hold(Hold::Turn))
    } else if facts.opening == Status::Pending {
        wait(landing, None)
    } else {
        done(landing, Decision::Open)
    }
}

// A verdict at another prefix only finishes its posting. It never retries,
// holds, repairs or moves the current probe's bounds (landing-search.md, 3, 6).
fn followed_post(landing: &mut Landing, facts: &Facts) -> Option<Decision> {
    let current = if landing.round.ending.is_none() { probe(landing).or(top(landing)) } else { None };
    for (index, (member, fresh)) in landing.members.iter_mut().zip(&facts.members).enumerate() {
        if current == Some(position(index)) || member.candidate.is_none() {
            continue;
        }
        for (number, (gate, fact)) in member.gates.iter_mut().zip(&fresh.gates).enumerate() {
            let key = match gate.proof {
                Proof::Dispatched { key, .. } => key,
                Proof::Keyed { key } if gate.posted == Posted::ByTest => key,
                Proof::Unkeyed | Proof::Keyed { .. } | Proof::Passed { .. } | Proof::Failed { .. } => continue,
            };
            if fact.posting == Status::Pending {
                return Some(Decision::Wait { until: None });
            }
            let posting = match fact.receipt {
                Status::Passed => {
                    gate.proof = Proof::Passed { key };
                    if gate.posted == Posted::ByTest { None } else { Some(Posting::Ran { passed: true }) }
                }
                Status::Failed => {
                    gate.proof = Proof::Failed { key };
                    Some(Posting::Ran { passed: false })
                }
                Status::Unknown | Status::Pending => {
                    if gate.posted == Posted::Nothing {
                        Some(Posting::Pending)
                    } else {
                        None
                    }
                }
            };
            if let Some(posting) = posting {
                gate.posted = if posting == Posting::Pending { Posted::Pending } else { Posted::Verdict };
                return Some(Decision::Post { member: position(index), gate: position(number), posting });
            }
        }
    }
    None
}

/// Decide against a coherent snapshot. The top commits the state and its
/// effect together; effect facts always describe the current preparation/try.
#[must_use]
pub fn step(landing: &Landing, facts: &Facts, project: &Project, limits: &Limits, now: Wall) -> Stepped {
    let mut stepped = decide(landing, facts, project, limits, now);
    remember_left(landing, &mut stepped.landing);
    stepped
}
fn remember_left(before: &Landing, after: &mut Landing) {
    for (index, (old, member)) in before.members.iter().zip(&after.members).enumerate() {
        if old.left.is_none() && member.left.is_some() {
            after.last_left = Some(position(index));
        }
    }
}

#[expect(clippy::too_many_lines, reason = "ordered phase, leave closure, cleanup and freshness decisions")]
fn decide(landing: &Landing, facts: &Facts, project: &Project, limits: &Limits, now: Wall) -> Stepped {
    if landing.round.branch.is_empty() {
        return crate::single::step(landing, facts, project, limits, now);
    }
    admitted(landing, facts, limits);
    let mut next = landing.clone();
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
    if let Some(decision) = followed_post(&mut next, facts) {
        return done(next, decision);
    }
    if next.round.ending.is_none() {
        // A successful turn is read back before heads that its changes may now delete.
        let readback = match top(&next) {
            Some(index) => {
                let member = next.members.get(usize::from(index)).expect("top");
                let fresh = facts.members.get(usize::from(index)).expect("top facts");
                match next.phase {
                    Phase::Turn { .. } => member.candidate == Some(fresh.main),
                    Phase::Waiting
                    | Phase::Preparing
                    | Phase::Proving
                    | Phase::Proved { .. }
                    | Phase::Landed
                    | Phase::Held { .. }
                    | Phase::Cancelled => false,
                }
            }
            None => false,
        };
        if !readback {
            for (member, fresh) in next.members.iter_mut().zip(&facts.members) {
                if !active(member) {
                    continue;
                }
                member.left = if let Some(gate) = fresh.refused {
                    Some(crate::Leave::Refused { gate })
                } else if fresh.cancelled {
                    Some(crate::Leave::Cancelled)
                } else if member.head.is_some() && member.head != Some(fresh.head) {
                    Some(crate::Leave::HeadMoved { head: fresh.head })
                } else if fresh.held {
                    Some(crate::Leave::Held)
                } else {
                    None
                };
            }
        }
    }
    cascade(&mut next, facts);
    if let Some(decision) = notify_leave(&mut next) {
        return done(next, decision);
    }
    if let Some(ending) = next.round.ending {
        if let Some(decision) = failed_post(&mut next, facts) {
            return done(next, decision);
        }
        return closing(next, facts, ending);
    }
    let mut remaining = false;
    for member in &next.members {
        remaining |= active(member);
    }
    if !remaining && !matches_turn(&next.phase) {
        next.phase = Phase::Landed;
        return done(next, Decision::Finish);
    }
    match landing.phase {
        Phase::Waiting | Phase::Preparing | Phase::Proving => {
            for (member, fresh) in next.members.iter().zip(&facts.members) {
                if active(member) && !fresh.ready {
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
        Phase::Turn { .. } => turn(next, facts, limits, now),
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

fn done(mut landing: Landing, decision: Decision) -> Stepped {
    if landing.members.len() == 1
        && let Some(member) = landing.members.first()
    {
        landing.round.combined = member.combined;
        landing.round.recorded = member.recorded;
        landing.round.told = member.told;
        landing.round.judging = member.judging;
    }
    Stepped { landing, decision }
}
fn wait(landing: Landing, until: Option<Wall>) -> Stepped {
    done(landing, Decision::Wait { until })
}

fn preparing(mut landing: Landing, facts: &Facts) -> Stepped {
    let mut new_round = false;
    for (member, fresh) in landing.members.iter_mut().zip(&facts.members) {
        if active(member) && member.base.is_none() {
            new_round = true;
            member.head = Some(fresh.head);
            member.base = Some(fresh.main);
        }
    }
    if new_round {
        landing.preparation = landing.preparation.checked_add(1).expect("round fits u16");
    }
    let mut found = false;
    for (index, (member, fresh)) in landing.members.iter_mut().zip(&facts.members).enumerate() {
        if !active(member) {
            continue;
        }
        if fresh.contained == Status::Passed && member.candidate.is_none() {
            member.landed = Some(fresh.main);
            member.told = true;
            return done(landing, Decision::Landed { member: position(index), prefix: fresh.main });
        }
        if member.candidate.is_none()
            && let Some((candidate, combined)) = fresh.candidate
        {
            member.candidate = Some(candidate);
            member.combined = Some(combined);
            found = true;
        }
    }
    if found {
        for member in &mut landing.members {
            member.asked = None;
        }
    }
    for (index, fresh) in facts.members.iter().enumerate() {
        if !active(landing.members.get(index).expect("member")) {
            continue;
        }
        let why = match fresh.outcome {
            Some(crate::Outcome::Conflict) => Some(crate::Leave::Conflict),
            Some(crate::Outcome::EarlierConflict) => Some(crate::Leave::EarlierConflict),
            Some(crate::Outcome::Combined) | None => None,
        };
        if let Some(why) = why {
            return leaving(landing, facts, position(index), why);
        }
    }
    let mut from = None;
    for (index, member) in landing.members.iter().enumerate() {
        if !active(member) {
            continue;
        }
        from = from.or(member.base);
        if let Some(candidate) = member.candidate {
            from = Some(candidate);
            continue;
        }
        let main = from.expect("round base");
        let head = member.head.expect("round head");
        match facts.preparation {
            crate::Preparation::Running => return wait(landing, None),
            crate::Preparation::Failed if member.asked == Some(landing.preparation) && !found => {
                let why = Hold::Prepare;
                landing.phase = Phase::Held { was: Box::new(landing.phase), why };
                return done(landing, Decision::Hold(why));
            }
            crate::Preparation::Conflict => return leaving(landing, facts, position(index), crate::Leave::Conflict),
            crate::Preparation::Idle if member.asked == Some(landing.preparation) && !found => {
                return wait(landing, None);
            }
            crate::Preparation::Absent | crate::Preparation::Idle | crate::Preparation::Failed => {}
        }
        for member in &mut landing.members {
            if active(member) && member.candidate.is_none() {
                member.asked = Some(landing.preparation);
            }
        }
        return done(landing, Decision::Prepare { member: position(index), head, main });
    }
    for (member, fresh) in landing.members.iter().zip(&facts.members) {
        if active(member) && member.base != Some(fresh.main) {
            landing.round.ending = Some(crate::RoundEnd::Stale);
            landing.round.changed = Some(0);
            return done(landing, Decision::None);
        }
    }
    prepared(landing, facts)
}

fn prepared(mut landing: Landing, facts: &Facts) -> Stepped {
    if top(&landing).is_none() {
        landing.phase = Phase::Landed;
        return done(landing, Decision::Finish);
    }
    if landing.round.pull.is_none() {
        if let Some(pull) = facts.opened {
            landing.round.pull = Some(pull);
            landing.searched = None;
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
    // A released prepare can finish before its prefixes enter the round.
    // Its pushes settle before cleanup (domain/landing.md, 5.1–5.2).
    if facts.preparation == crate::Preparation::Running {
        return wait(landing, None);
    }
    for (member, fresh) in landing.members.iter_mut().zip(&facts.members) {
        if active(member)
            && member.candidate.is_none()
            && let Some((candidate, combined)) = fresh.candidate
        {
            member.candidate = Some(candidate);
            member.combined = Some(combined);
        }
    }
    if landing.round.pull.is_some() && facts.closed != Status::Passed {
        return if facts.closed == Status::Pending { wait(landing, None) } else { done(landing, Decision::Close) };
    }
    let first = match ending {
        crate::RoundEnd::Stale => 0,
        crate::RoundEnd::Leave | crate::RoundEnd::Conflict | crate::RoundEnd::Cut => landing.round.changed.unwrap_or(0),
    };
    let mut remaining = false;
    for member in &landing.members {
        remaining |= active(member);
    }
    // With no next round, task closure releases the held branches (domain/landing.md, 5.1).
    for (index, (member, fresh)) in landing.members.iter().zip(&facts.members).enumerate() {
        if !remaining || index < usize::from(first) || member.candidate.is_none() {
            continue;
        }
        if fresh.deleted != Status::Passed {
            return if fresh.deleted == Status::Pending {
                wait(landing, None)
            } else {
                done(landing, Decision::Delete { member: position(index) })
            };
        }
    }
    landing.round.pull = None;
    landing.round.ending = None;
    landing.round.changed = None;
    for member in landing.members.get_mut(usize::from(first)..).expect("changed member") {
        member.base = None;
        member.candidate = None;
        member.combined = None;
        member.asked = None;
        member.judging = false;
        for gate in &mut member.gates {
            *gate = GateProof { proof: Proof::Unkeyed, posted: Posted::Nothing, faults: 0 };
        }
    }
    let by_test = match &landing.round.search {
        Some(search) => match search.tests {
            crate::SearchTests::ByTest { .. } => true,
            crate::SearchTests::Whole | crate::SearchTests::WholeAgain => false,
        },
        None => false,
    };
    if by_test {
        for member in landing.members.get_mut(..usize::from(first)).expect("kept members") {
            for gate in &mut member.gates {
                *gate = GateProof { proof: Proof::Unkeyed, posted: Posted::Nothing, faults: 0 };
            }
        }
    }
    if ending == crate::RoundEnd::Cut && !by_test && landing.round.search.as_ref().expect("cut searches").green != 0 {
        landing.phase = Phase::Proving;
        return done(landing, Decision::None);
    }
    landing.round.search = None;
    landing.phase = Phase::Preparing;
    match ending {
        crate::RoundEnd::Stale => {
            landing.stale_proofs = landing.stale_proofs.checked_add(1).expect("stale count fits");
            done(landing, Decision::Stale)
        }
        crate::RoundEnd::Leave | crate::RoundEnd::Conflict | crate::RoundEnd::Cut => {
            let mut missing = false;
            for member in &landing.members {
                if active(member) {
                    missing |= member.base.is_none();
                }
            }
            if remaining {
                // A kept top opens another round without initializing any new member.
                if !missing {
                    landing.preparation = landing.preparation.checked_add(1).expect("round fits u16");
                }
                return done(landing, Decision::None);
            }
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
    if let Some(search) = &landing.round.search {
        if probe(&landing).is_some() {
            return probing(landing, facts, project, limits, now);
        }
        let red = search.red.checked_sub(1).expect("red above base");
        if !blamed(landing.members.get(usize::from(red)).expect("red member").left) {
            return boundary(landing, facts);
        }
        if landing.round.pull.is_none() {
            return open(landing, facts);
        }
    }
    let proved_top = top(&landing).expect("proving has a top");
    let mut blocked = false;
    let mut poll = false;
    for (index, fresh) in facts.members.iter().enumerate() {
        let member = landing.members.get_mut(index).expect("member");
        if !active(member) || position(index) > proved_top {
            continue;
        }
        if fresh.judgement != Status::Passed {
            blocked = true;
            if !member.judging {
                member.judging = true;
                return done(landing, Decision::Judge { member: position(index) });
            }
        }
    }
    let top = top(&landing).expect("proving has a top");
    for (index, (member, fresh)) in landing.members.iter_mut().zip(&facts.members).enumerate() {
        if position(index) != top {
            continue;
        }
        let candidate = member.candidate.expect("proving has every candidate");
        for (number, (gate, fact)) in member.gates.iter_mut().zip(&fresh.gates).enumerate() {
            if landing.round.search.is_some() && !fact.blocking {
                continue;
            }
            let decision = gate_step(gate, fact, position(index), position(number), candidate, project, false);
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
                Decision::Key { .. } | Decision::Dispatch { .. } | Decision::Retry { .. } | Decision::Post { .. } => {
                    return done(landing, decision);
                }
                Decision::Repair { member, gate } => {
                    let run = facts
                        .members
                        .get(usize::from(member))
                        .expect("member")
                        .gates
                        .get(usize::from(gate))
                        .expect("gate")
                        .run;
                    return red(landing, facts, limits, gate, run);
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
                | Decision::Delete { .. }
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
    by_test: bool,
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
            if gate.posted != Posted::Verdict && gate.posted != Posted::ByTest {
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
            gate.proof = Proof::Failed { key };
            if fact.blocking {
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
                if by_test {
                    gate.posted = Posted::ByTest;
                } else if gate.posted == Posted::ByTest {
                    gate.posted = Posted::Nothing;
                }
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

fn red(mut landing: Landing, facts: &Facts, limits: &Limits, gate: u16, run: u64) -> Stepped {
    let mut units = skein_lib::Set::with_capacity(u32::try_from(landing.members.len()).expect("members admitted"));
    for member in &landing.members {
        if active(member) {
            units.insert(member.unit).expect("unit room");
        }
    }
    let top = top(&landing).expect("red has a top");
    if units.len() > 1 {
        landing.round.search = Some(crate::Search {
            green: 0,
            red: top.checked_add(1).expect("member number"),
            tests: selection(facts.members.get(usize::from(top)).expect("top facts"), limits),
        });
        return done(landing, Decision::None);
    }
    let why = crate::Leave::Failed { gate, run };
    for member in &mut landing.members {
        if active(member) {
            member.left = Some(why);
        }
    }
    cascade(&mut landing, facts);
    let decision = notify_leave(&mut landing).expect("red leaves members");
    done(landing, decision)
}

fn turn(mut landing: Landing, facts: &Facts, limits: &Limits, now: Wall) -> Stepped {
    let top = top(&landing);
    if let Some(top) = top {
        let member = landing.members.get(usize::from(top)).expect("top");
        let fresh = facts.members.get(usize::from(top)).expect("top facts");
        let candidate = member.candidate.expect("proved candidate");
        if fresh.main == candidate {
            landing.phase = Phase::Turn { at: now };
            for (index, member) in landing.members.iter_mut().enumerate() {
                if active(member) && position(index) <= top {
                    member.landed = member.candidate;
                }
            }
        } else {
            if member.base != Some(fresh.main) {
                landing.round.ending = Some(crate::RoundEnd::Stale);
                landing.round.changed = Some(0);
                return done(landing, Decision::None);
            }
            return match fresh.merge {
                Status::Unknown => done(landing, Decision::Merge { member: top, candidate }),
                Status::Pending | Status::Passed => wait(landing, None),
                Status::Failed => {
                    landing.phase = Phase::Held { was: Box::new(landing.phase), why: Hold::Turn };
                    done(landing, Decision::Hold(Hold::Turn))
                }
            };
        }
    }
    for (index, (member, fresh)) in landing.members.iter_mut().zip(&facts.members).enumerate() {
        if let Some(prefix) = member.landed {
            if member.combined == Some(crate::Combine::Squash) && !member.recorded {
                match fresh.record {
                    Status::Passed => member.recorded = true,
                    Status::Unknown => return done(landing, Decision::Record { member: position(index), prefix }),
                    Status::Pending => return wait(landing, None),
                    Status::Failed => {
                        let why = Hold::Record { member: position(index) };
                        landing.phase = Phase::Held { was: Box::new(landing.phase), why };
                        return done(landing, Decision::Hold(why));
                    }
                }
            }
            if !member.told {
                // Give manual-merge detection one poll from merge readback;
                // the branch readback remains authoritative (landing.md, 8).
                if member.combined == Some(crate::Combine::Merge) && fresh.merged != Some(prefix) {
                    let at = match &landing.phase {
                        Phase::Turn { at } => *at,
                        Phase::Waiting
                        | Phase::Preparing
                        | Phase::Proving
                        | Phase::Proved { .. }
                        | Phase::Landed
                        | Phase::Cancelled
                        | Phase::Held { .. } => unreachable!("the turn owns landed notifications"),
                    };
                    let until = Wall::from_nanos(at.as_nanos().saturating_add(limits.poll.as_nanos()));
                    if now < until {
                        return wait(landing, Some(until));
                    }
                }
                member.told = true;
                return done(landing, Decision::Landed { member: position(index), prefix });
            }
        }
    }
    for member in &landing.members {
        if active(member) {
            landing.round.search = None;
            landing.round.pull = None;
            landing.phase = Phase::Preparing;
            return done(landing, Decision::None);
        }
    }
    landing.phase = Phase::Landed;
    done(landing, Decision::Finish)
}

// A probe takes only blocking receipts at its own prefix; older flights are
// followed by the top and cannot move these bounds (landing-search.md, 3, 6).
fn probing(mut landing: Landing, facts: &Facts, project: Project, limits: &Limits, now: Wall) -> Stepped {
    let by_test = match &landing.round.search.as_ref().expect("search").tests {
        crate::SearchTests::ByTest { .. } => true,
        crate::SearchTests::Whole | crate::SearchTests::WholeAgain => false,
    };
    if by_test {
        return probing_tests(landing, facts, project, limits, now);
    }
    let index = probe(&landing).expect("probe");
    let member = landing.members.get_mut(usize::from(index)).expect("probe member");
    let fresh = facts.members.get(usize::from(index)).expect("probe facts");
    let candidate = member.candidate.expect("probe prefix");
    let mut blocked = false;
    for (number, (gate, fact)) in member.gates.iter_mut().zip(&fresh.gates).enumerate() {
        if !fact.blocking {
            continue;
        }
        let decision = gate_step(gate, fact, index, position(number), candidate, project, false);
        if let Decision::Repair { .. } = decision {
            landing.round.search.as_mut().expect("search").red = index.checked_add(1).expect("member number");
            return done(landing, Decision::None);
        }
        if let Decision::Wait { .. } = decision {
            blocked = true;
            continue;
        }
        if let Decision::Hold(why) = decision {
            landing.phase = Phase::Held { was: Box::new(landing.phase), why };
        }
        if decision != Decision::None {
            return done(landing, decision);
        }
    }
    if blocked {
        return wait(landing, Some(Wall::from_nanos(now.as_nanos().saturating_add(limits.poll.as_nanos()))));
    }
    landing.round.search.as_mut().expect("search").green = index.checked_add(1).expect("member number");
    done(landing, Decision::None)
}

fn boundary(mut landing: Landing, facts: &Facts) -> Stepped {
    let search = landing.round.search.as_ref().expect("boundary");
    let red_index = search.red.checked_sub(1).expect("red above base");
    let red_member = landing.members.get(usize::from(red_index)).expect("red member");
    let red = red_member.candidate.expect("red prefix");
    let unit = red_member.unit;
    let green = if search.green == 0 {
        red_member.base.expect("base")
    } else {
        landing
            .members
            .get(usize::from(search.green.checked_sub(1).expect("green")))
            .expect("green member")
            .candidate
            .expect("green prefix")
    };
    let by_test = match &search.tests {
        crate::SearchTests::ByTest { .. } => true,
        crate::SearchTests::Whole | crate::SearchTests::WholeAgain => false,
    };
    let green_prefix = if search.green == 0 { None } else { Some(green) };
    let mut failed = None;
    for (number, fact) in facts.members.get(usize::from(red_index)).expect("red facts").gates.iter().enumerate() {
        let selected_failure = match &search.tests {
            crate::SearchTests::Whole | crate::SearchTests::WholeAgain => true,
            crate::SearchTests::ByTest { gates, .. } => match gates.get(number).expect("gate") {
                Some(selected) => intersects(
                    selected,
                    failed_ids(facts.members.get(usize::from(red_index)).expect("red facts"), number),
                ),
                None => false,
            },
        };
        if fact.blocking && fact.receipt == Status::Failed && selected_failure {
            failed = Some((position(number), fact.run));
            break;
        }
    }
    let (gate, run) = failed.expect("red receipt");
    let mut changes = List::with_capacity(u32::try_from(landing.members.len()).expect("members"));
    for (index, member) in landing.members.iter_mut().enumerate() {
        if member.unit == unit && active(member) {
            changes.push(member.change).expect("members");
            member.left = Some(crate::Leave::Blamed { gate, run, red, green });
            landing.round.changed = Some(landing.round.changed.unwrap_or(position(index)).min(position(index)));
        }
    }
    if by_test {
        landing.searched = Some(crate::Searched { red, green: green_prefix, changes: changes.into_boxed() });
    }
    landing.round.ending = Some(crate::RoundEnd::Cut);
    cascade(&mut landing, facts);
    let decision = notify_leave(&mut landing).expect("blamed leaves");
    done(landing, decision)
}

// The room includes one line feed per id (landing-search-by-test.md, 3).
fn selection(fresh: &crate::MemberFacts, limits: &Limits) -> crate::SearchTests {
    let mut gates = List::with_capacity(u32::try_from(fresh.gates.len()).expect("gates"));
    for (number, fact) in fresh.gates.iter().enumerate() {
        let selected = if fact.blocking && fact.receipt == Status::Failed {
            let Some(ids) = failed_ids(fresh, number) else {
                return crate::SearchTests::Whole;
            };
            let mut room = skein_lib::usize_of(limits.tests_bytes);
            for id in ids {
                let Some(left) = room.checked_sub(id.len().saturating_add(1)) else {
                    return crate::SearchTests::Whole;
                };
                room = left;
            }
            Some(Box::from(ids))
        } else {
            None
        };
        gates.push(selected).expect("one per gate");
    }
    crate::SearchTests::ByTest { gates: gates.into_boxed(), probed: Box::new([]) }
}

fn intersects(selected: &[crate::TestId], failed: Option<&[crate::TestId]>) -> bool {
    if let Some(failed) = failed {
        for id in selected {
            if failed.contains(id) {
                return true;
            }
        }
    }
    false
}

fn probing_tests(mut landing: Landing, facts: &Facts, project: Project, limits: &Limits, now: Wall) -> Stepped {
    let original_top = top(&landing).expect("top");
    let index = probe(&landing).expect("probe");
    let member = landing.members.get_mut(usize::from(index)).expect("probe member");
    let fresh = facts.members.get(usize::from(index)).expect("probe facts");
    let candidate = member.candidate.expect("probe prefix");
    let search = landing.round.search.as_mut().expect("search");
    let (gates, probed) = match &mut search.tests {
        crate::SearchTests::ByTest { gates, probed } => (gates, probed),
        crate::SearchTests::Whole | crate::SearchTests::WholeAgain => unreachable!("by test"),
    };
    let mut blocked = false;
    for (number, (gate, fact)) in member.gates.iter_mut().zip(&fresh.gates).enumerate() {
        let Some(selected) = gates.get(number).expect("selection per gate") else { continue };
        if !fact.blocking {
            continue;
        }
        gate.posted = Posted::ByTest;
        if fact.receipt == Status::Passed {
            continue;
        }
        if fact.receipt == Status::Failed && intersects(selected, failed_ids(fresh, number)) {
            // Every red gate narrows, even if another gate supplied the first red.
            for (number, selected) in gates.iter_mut().enumerate() {
                let fact = fresh.gates.get(number).expect("gate");
                if fact.blocking
                    && fact.receipt == Status::Failed
                    && let Some(ids) = selected
                    && let Some(failed) = failed_ids(fresh, number)
                    && intersects(ids, Some(failed))
                {
                    let mut kept = List::with_capacity(u32::try_from(ids.len()).expect("selected ids"));
                    for id in ids.iter() {
                        if failed.contains(id) {
                            kept.push(id.clone()).expect("selection narrows");
                        }
                    }
                    *ids = kept.into_boxed();
                    let proof = member.gates.get_mut(number).expect("gate");
                    if let Some(key) = fact.key {
                        proof.proof = Proof::Failed { key };
                    }
                }
            }
            search.red = index.checked_add(1).expect("member number");
            return done(landing, Decision::None);
        }
        let mut verdict = None;
        for outcome in probed.iter() {
            if outcome.member == index.checked_add(1).expect("member number") && outcome.gate == position(number) {
                verdict = Some(outcome.passed);
                break;
            }
        }
        if let Some(passed) = verdict {
            if passed {
                continue;
            }
            // No assumption made by a selected run enters the whole search.
            search.green = 0;
            search.red = original_top.checked_add(1).expect("member number");
            landing.round.search.as_mut().expect("search").tests = crate::SearchTests::WholeAgain;
            for member in &mut landing.members {
                for proof in &mut member.gates {
                    if proof.posted == Posted::ByTest {
                        proof.proof = match proof.proof {
                            Proof::Keyed { key } | Proof::Dispatched { key, .. } | Proof::Passed { key } => {
                                Proof::Keyed { key }
                            }
                            Proof::Unkeyed | Proof::Failed { .. } => proof.proof,
                        };
                    }
                }
            }
            return done(landing, Decision::None);
        }
        let mut absent = *fact;
        absent.receipt = Status::Unknown;
        let decision = gate_step(gate, &absent, index, position(number), candidate, project, true);
        if let Decision::Wait { .. } = decision {
            blocked = true;
            continue;
        }
        if let Decision::Hold(why) = decision {
            landing.phase = Phase::Held { was: Box::new(landing.phase), why };
        }
        if decision != Decision::None {
            return done(landing, decision);
        }
    }
    if blocked {
        return wait(landing, Some(Wall::from_nanos(now.as_nanos().saturating_add(limits.poll.as_nanos()))));
    }
    landing.round.search.as_mut().expect("search").green = index.checked_add(1).expect("member number");
    done(landing, Decision::None)
}

fn failed_ids(fresh: &crate::MemberFacts, number: usize) -> Option<&[crate::TestId]> {
    fresh.failed.get(number)?.as_deref()
}
