//! One level-triggered change step (domain/forge/change/README.md).
//! Updates with no result or outstanding write decide again (temper/docs/design/domain/forge/updates.md, 1).
//!
//! The top keeps the returned state and commits each decision before it sends
//! a delegate or effect. The top reports Pending while a write is outstanding.
//! This policy never reads the forge or commits an outbox entry itself.
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall, bytes::copy_of};

use crate::{
    Ask, AuthorityHold, AuthorityLack, Cause, Change, Decision, Delegate, Early, Effect, EffectResult, Facts,
    Freshness, Gate, GateFailure, GateKind, Handed, Heard, Hold, Judged, Limits, PendingKind, Pull, Recheck, Repair,
    Replay, StartedFrom, State, Status, Stepped, Took, Verification, Wait, Way,
};

/// Decide one change from current forge and task facts, independent of its wake.
#[must_use]
pub fn step(change: &Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    let mut stepped = advance(change, facts, standing, now, limits);
    if let Decision::Delegate(delegate) = stepped.decision
        && let Some(why) = missing(standing, delegate)
    {
        // Keep the unrequested phase and its counters: no delegate was made.
        stepped = hold(change.clone(), Cause::Authority(why), now);
    }
    if let State::Updating { .. } | State::Preparing { .. } = stepped.change.state {
    } else {
        stepped.change.stale = None;
    }
    stepped
}

fn advance(change: &Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    let mut next = change.clone();
    if facts.kept {
        next.owns_turn = false;
    }
    if let State::Held { .. } = change.state {
        // The held step and its account are not replaced by a reduced bound.
    } else if change.gates.len() > skein_lib::usize_of(limits.gates)
        || change.clean.len() > skein_lib::usize_of(limits.clean_heads)
        || change.gate_failures.len() > skein_lib::usize_of(limits.gates)
    {
        return hold(next, Cause::Failed, now);
    }
    if !standing.cancelled {
        let trigger = if standing.released {
            ReleaseTrigger::Task { owes_readback: false, close: facts.close }
        } else if next.released.is_some() {
            ReleaseTrigger::Fresh
        } else {
            ReleaseTrigger::Green
        };
        if let Some(stepped) = release(&next, trigger, Some(facts), Some(standing), now) {
            return stepped;
        }
    }
    if let Some(stepped) = settle_landed(&next, facts, standing, now) {
        return stepped;
    }
    if standing.cancelled {
        if let State::Held { hold: Hold { cause: why, .. }, .. } = &next.state {
            next.ended_held = Some(*why);
        }
        next.state = State::Ended;
        next.released = None;
        next.owns_turn = false;
        return Stepped { took: None, change: next, decision: Decision::Cancel };
    }
    if facts.behind.is_none()
        && facts.contains_head == Status::Passed
        && (change.way == Way::Queue || facts.landing.is_none())
        && !standing.delegate_live
        && !facts.writer_taken
        && facts.drift.is_none()
        && head_may_end(&next.state)
        && let Some(head) = facts.branch
    {
        next.state = State::NoChange { words: contained_words(b"its head ", head) };
        next.owns_turn = false;
        return Stepped { took: None, change: next, decision: Decision::NoChange };
    }
    if let Some(since) = facts.behind {
        let until = Wall::from_nanos(since.as_nanos().saturating_add(limits.stall.as_nanos()));
        return if now >= until {
            hold(next, Cause::Stalled, now)
        } else {
            Stepped { took: None, change: next, decision: Decision::Wait { until: Some(until) } }
        };
    }
    if let State::Held { .. } = next.state {
        return Stepped { took: None, change: next, decision: Decision::None };
    }
    clear_old_gate_failures(&mut next, facts);
    // A conflicting preparation cancels its producer before waiting for that
    // producer's writer slot (domain/forge/change/questions.md, 1).
    let start_conflicted = match next.early {
        Early::None => false,
        Early::Started(_) | Early::Spent(_) => standing.start_conflict.is_some(),
    };
    if facts.writer_taken && !start_conflicted {
        return waiting_on_agent(next, facts, standing, now, limits);
    }
    if let Some(base) = moved_delegate_base(&next, facts)
        && standing.delegate_live
        && !standing.delegate_running
        && !facts.writer_taken
        && (standing.delegate == Status::Unknown || standing.delegate == Status::Pending)
        && let Some(head) = facts.branch
        && next.last_head != Some(head)
    {
        // End the stale pin before judging drift; WaitingProducer keeps the
        // change's own cancel apart from a person's (repairs.md, A resolution's moved head).
        next.state = State::WaitingProducer { base };
        next.since = now;
        next.owns_turn = false;
        return Stepped { took: None, change: next, decision: Decision::CancelMovedDelegate { head } };
    }
    if let Some(why) = facts.drift {
        return hold(next, why, now);
    }
    if let Some(head) = facts.branch
        && let Some(prior) = step_head(&change.state, change.last_head, standing.effect, facts.update_head)
        && prior != head
    {
        if facts.pull == Pull::Missing {
            next = take_pushed_head(next, head, now).change;
            let requested =
                standing.effect == EffectResult::Pending && change.state == State::Opening { requested: true };
            next.state = State::Opening { requested };
            return opening(next, requested, facts, standing, now, limits);
        }
        return take_pushed_head(next, head, now);
    }
    advance_phase(change, next, facts, standing, now, limits)
}

// Resolutions and repairs end at a moved head
// (temper/docs/design/domain/forge/change/repairs.md, A resolution's moved head).
fn moved_delegate_base(change: &Change, facts: &Facts) -> Option<[u8; 32]> {
    match &change.state {
        State::Resolving { requested: true, base } if facts.delegate == Some(Delegate::Resolve { base: *base }) => {
            Some(*base)
        }
        State::Repairing { requested: true, why } if facts.delegate == Some(Delegate::Repair(*why)) => match why {
            Repair::Ci
            | Repair::Semantic
            | Repair::Landing { .. }
            | Repair::LandingAt { .. }
            | Repair::Gate(_)
            | Repair::Review { .. } => Some(facts.base_tip),
        },
        State::Taking { .. }
        | State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::NoChange { .. }
        | State::Held { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::Ended => None,
    }
}

/// Only before its turn may a contained head end with no change (domain/forge/change/README.md, 2).
fn head_may_end(state: &State) -> bool {
    match state {
        State::Taking { .. }
        | State::Held { .. }
        | State::Updating { .. }
        | State::Preparing { .. }
        | State::First { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. }
        | State::Ended => false,
        State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::Queued { .. }
        | State::Ready { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. } => true,
    }
}

fn settle_landed(change: &Change, facts: &Facts, standing: &Heard, now: Wall) -> Option<Stepped> {
    // Take the landing's durable prefix before any other queued ask or forge read.
    if change.way == Way::Turn
        && let Some(Handed { number, ask: Ask::Landed { prefix }, .. }) = facts.ask
    {
        let mut stepped = record_landed(change, prefix, facts, standing, now);
        stepped.took = Some(Took::Taken(number));
        return Some(stepped);
    }
    if let Some(stepped) = after_landing(change, facts, standing, now) {
        return Some(stepped);
    }
    match facts.pull {
        Pull::Merged { commit } if change.way == Way::Queue || facts.landing.is_none() => {
            Some(record_landed(change, commit, facts, standing, now))
        }
        Pull::Missing | Pull::Open { .. } | Pull::Closed | Pull::Merged { .. } => None,
    }
}

fn record_landed(change: &Change, merge: [u8; 32], facts: &Facts, standing: &Heard, now: Wall) -> Stepped {
    let mut next = change.clone();
    next.state = State::Landed { merge, since: now };
    next.owns_turn = false;
    next.released = None;
    verifying(next, merge, now, facts, standing.cancelled, now)
}

fn after_landing(change: &Change, facts: &Facts, standing: &Heard, now: Wall) -> Option<Stepped> {
    let decision = match &change.state {
        State::Landed { merge, .. } | State::Red { merge, .. } if standing.cancelled => {
            let mut next = change.clone();
            next.state = State::Uncleared { merge: *merge };
            return Some(Stepped { took: None, change: next, decision: Decision::Finish { merge: *merge } });
        }
        State::Landed { merge, since } => {
            return Some(verifying(change.clone(), *merge, *since, facts, standing.cancelled, now));
        }
        State::Red { merge, .. } => {
            let mut next = change.clone();
            let decision = match facts.verification {
                Verification::Cleared => {
                    next.state = State::Cleared { merge: *merge };
                    Decision::Finish { merge: *merge }
                }
                Verification::Unfollowed | Verification::Verifying | Verification::Red => {
                    if facts.wait == Wait::Landed {
                        next.state = State::Uncleared { merge: *merge };
                        Decision::Finish { merge: *merge }
                    } else {
                        Decision::Wait { until: None }
                    }
                }
            };
            return Some(Stepped { took: None, change: next, decision });
        }
        State::Cleared { merge } | State::Uncleared { merge } => Decision::Finish { merge: *merge },
        State::NoChange { .. } => Decision::NoChange,
        State::Ended => Decision::Cancel,
        State::Held { .. }
        | State::Taking { .. }
        | State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. } => return None,
    };
    Some(Stepped { took: None, change: change.clone(), decision })
}

fn clear_old_gate_failures(change: &mut Change, facts: &Facts) {
    if let Some(head) = facts.branch
        && change.last_head != Some(head)
    {
        change.gate_failures = Box::new([]);
    }
}

fn verifying(mut change: Change, merge: [u8; 32], since: Wall, facts: &Facts, cancelled: bool, now: Wall) -> Stepped {
    // Verification still consumes the fresh head; gate failures belong to the old one.
    clear_old_gate_failures(&mut change, facts);
    let decision = match facts.verification {
        Verification::Unfollowed => Decision::Finish { merge },
        Verification::Verifying | Verification::Red | Verification::Cleared if cancelled => {
            change.state = State::Uncleared { merge };
            Decision::Finish { merge }
        }
        Verification::Cleared => {
            change.state = State::Cleared { merge };
            Decision::Finish { merge }
        }
        Verification::Verifying | Verification::Red if facts.wait == Wait::Landed => {
            change.state = State::Uncleared { merge };
            Decision::Finish { merge }
        }
        Verification::Red => {
            change.state = State::Red { merge, since };
            Decision::Wait { until: None }
        }
        Verification::Verifying => {
            let until = Wall::from_nanos(since.as_nanos().saturating_add(facts.verification_bound.as_nanos()));
            if now >= until {
                change.state = State::Uncleared { merge };
                Decision::Finish { merge }
            } else {
                Decision::Wait { until: Some(until) }
            }
        }
    };
    Stepped { took: None, change, decision }
}

fn advance_phase(
    change: &Change,
    next: Change,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    match &change.state {
        State::Taking { .. } => taking(next, facts, standing, now, limits),
        State::Producing { requested } => producing(next, *requested, facts, standing, now, limits),
        State::Opening { requested } => opening(next, *requested, facts, standing, now, limits),
        State::Recreating { head } => recreating(next, *head, facts, now, limits),
        State::Reopening { requested, retarget } => {
            reopening(next, *requested, *retarget, facts, standing, now, limits)
        }
        State::Checking { .. } | State::Gating { .. } | State::CancellingGates { .. }
            if standing.effect == EffectResult::Pending =>
        {
            waiting(next, facts, now, limits)
        }
        State::Checking { head } if facts.recheck == Recheck::Owed && standing.effect == EffectResult::Failed => {
            checking(next, *head, facts, standing, now, limits)
        }
        State::Checking { .. } | State::Gating { .. } | State::CancellingGates { .. }
            if standing.effect == EffectResult::Failed =>
        {
            hold(next, Cause::Failed, now)
        }
        State::Checking { head } => checking(next, *head, facts, standing, now, limits),
        State::Gating { head, asked } => gating(next, *head, asked, facts, now, limits),
        State::Judging { .. } => judging(next, facts, standing, now, limits),
        State::Ready { head, ready_since } => ready(next, *head, *ready_since, facts, standing, now, limits),
        State::Preparing { from, main, ready_since } => {
            preparing(next, *from, *main, *ready_since, facts, standing, now, limits)
        }
        State::CancellingGates { head } => cancelling_gates(next, *head, facts, now, limits),
        State::Queued { head, ready_since } => queued(next, *head, *ready_since, facts, standing, now, limits),
        State::First { head, ready_since } => first(next, *head, *ready_since, facts, standing, now, limits),
        State::Updating { from, base, ready_since } => {
            updating(next, *from, *base, *ready_since, facts, standing, now, limits)
        }
        State::Resolving { requested, base } => resolving(next, *requested, *base, facts, standing, now, limits),
        State::WaitingProducer { base } => waiting_producer(next, *base, facts, standing, now, limits),
        State::Repairing { requested, why } => repairing(next, *requested, *why, facts, standing, now, limits),
        State::Landing { head } => landing(next, *head, facts, standing, now, limits),
        State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. }
        | State::Ended
        | State::Held { .. } => {
            unreachable!("terminal states returned above")
        }
    }
}

// The update's own new head retains clean-update semantics, not another push.
fn step_head(state: &State, last: Option<[u8; 32]>, effect: EffectResult, update_head: bool) -> Option<[u8; 32]> {
    match state {
        State::Opening { .. } => last,
        State::Checking { head }
        | State::Judging { head, .. }
        | State::Gating { head, .. }
        | State::CancellingGates { head }
        | State::Queued { head, .. }
        | State::First { head, .. }
        | State::Ready { head, .. } => Some(*head),
        State::Landing { head } if effect != EffectResult::Pending => Some(*head),
        State::Preparing { from, .. } | State::Updating { from, .. }
            if effect != EffectResult::Pending && !update_head =>
        {
            Some(*from)
        }
        State::Taking { .. }
        | State::Producing { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Preparing { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::NoChange { .. }
        | State::Ended
        | State::Held { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. } => None,
    }
}

/// A hold whose reason is CI failed at the head (temper/docs/design/domain/forge/change/repairs.md).
/// `was` `Repairing` with a CI reason: `Failed`, `Stalled`, `Repairs`, `Authority` or `RedStands`.
/// `was` `Checking` or `CancellingGates`: `Repairs` or `Authority`.
/// A `Runner` hold always belongs to CI.
#[must_use]
pub fn held_on_ci(state: &State) -> bool {
    let State::Held { was, hold: Hold { cause: why, .. } } = state else { return false };
    match why {
        Cause::Runner => true,
        Cause::Repairs | Cause::Authority(_) => ci_state(was),
        Cause::Failed | Cause::Stalled | Cause::RedStands => {
            if let State::Repairing { why, .. } = was.as_ref() {
                why.ci()
            } else {
                false
            }
        }
        Cause::BranchMoved
        | Cause::BranchMissing
        | Cause::PullClosed
        | Cause::Retargeted
        | Cause::Resolutions
        | Cause::Updates
        | Cause::Rejected
        | Cause::Questions
        | Cause::CloseRefused => false,
    }
}

/// What admitted a release, or the fresh snapshot continuing an admitted one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ReleaseTrigger {
    Task { owes_readback: bool, close: bool },
    Green,
    DelegateEnded { owes_readback: bool, no_change: Option<Box<[u8]>> },
    DependenciesDone { owes_readback: bool },
    Fresh,
}

/// Release a hold, or continue that release on fresh forge facts. The caller
/// takes the task revision and fences outstanding reads before supplying new
/// facts. An owed readback keeps the held predecessor until it can be settled.
/// `Change::released` marks the continuation, including across restart.
#[must_use]
#[expect(clippy::too_many_lines, reason = "the release orders admission, settlement, forgetting and re-entry")]
pub fn release(
    change: &Change,
    trigger: ReleaseTrigger,
    facts: Option<&Facts>,
    standing: Option<&Heard>,
    now: Wall,
) -> Option<Stepped> {
    let predecessor = match &change.state {
        State::Held { was, hold } => Some((was.as_ref(), hold)),
        State::Taking { .. }
        | State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. }
        | State::Ended => None,
    };
    let mut end_hold = false;
    let mut nothing = None;
    let (owes_readback, close) = match trigger {
        ReleaseTrigger::Task { owes_readback, close } => (owes_readback, close),
        ReleaseTrigger::Fresh => {
            change.released?;
            (false, facts?.close)
        }
        ReleaseTrigger::Green => {
            let f = facts?;
            let h = standing?;
            if change.released.is_some()
                || f.release_blocked
                || f.behind.is_some()
                || !held_on_ci(&change.state)
                || green_head(f).is_none()
                || h.delegate_running
                || f.drift.is_some()
                || f.writer_taken
                || f.pull == Pull::Closed
            {
                return None;
            }
            end_hold = true;
            (false, f.close)
        }
        ReleaseTrigger::DelegateEnded { owes_readback, no_change } => {
            let (was, hold) = predecessor?;
            if hold.cause != Cause::Stalled || !delegate_wait(was) || change.released.is_some() {
                return None;
            }
            end_hold = true;
            nothing = no_change;
            (owes_readback, false)
        }
        ReleaseTrigger::DependenciesDone { owes_readback } => {
            let (was, hold) = predecessor?;
            if hold.cause != Cause::Stalled
                || *was != (State::Producing { requested: false })
                || change.released.is_some()
            {
                return None;
            }
            end_hold = true;
            (owes_readback, false)
        }
    };
    let decision = if end_hold { Decision::EndHold } else { Decision::None };
    let mut next = change.clone();
    let cause = match predecessor {
        Some((_, hold)) => hold.cause,
        None => match change.released {
            Some(cause) => cause,
            None => {
                // A task may be released from a party's stop while its change is live.
                next.since = now;
                next.stale = None;
                next.clock = None;
                next.gate_failures = Box::new([]);
                return Some(Stepped { took: None, change: next, decision });
            }
        },
    };
    if let Some(facts) = facts {
        let standing = standing?;
        if let Some(stepped) = settle_landed(change, facts, standing, now) {
            return Some(stepped);
        }
        if facts.behind.is_some() {
            return None;
        }
        let was = match predecessor {
            Some((was, _)) => was,
            None => &change.state,
        };
        if let Some(stepped) = contained_landing_at(change, was, facts, standing, now) {
            return Some(stepped);
        }
        if cause == Cause::Failed
            && standing.effect == EffectResult::Pending
            && let State::Landing { .. } = was
        {
            // An open pull is not proof that the outstanding merge failed.
            next.released = Some(cause);
            return Some(Stepped { took: None, change: next, decision: Decision::Wait { until: None } });
        }
        if nothing.is_none() {
            nothing.clone_from(&facts.produced_nothing);
        }
    }
    if change.released.is_none() {
        next.since = now;
    }
    if owes_readback {
        next.released = Some(cause);
        return Some(Stepped { took: None, change: next, decision });
    }
    if let Some(words) = nothing {
        next.state = State::NoChange { words };
        next.released = None;
        next.owns_turn = false;
        return Some(Stepped {
            took: None,
            change: next,
            decision: if end_hold { decision } else { Decision::NoChange },
        });
    }
    if let Some((was, hold)) = predecessor {
        // The judgement's predecessor, inputs and account must survive until
        // the fenced reads arrive, including a restart during their gather.
        if hold.cause.kind() == crate::Kind::Judgement && facts.is_none() {
            next.released = Some(cause);
            return Some(Stepped { took: None, change: next, decision });
        }
        forget_release(&mut next, was, cause);
        if close && cause == Cause::Failed && *was == (State::Producing { requested: true }) {
            next.state = State::Producing { requested: false };
            next.released = None;
            return Some(Stepped { took: None, change: next, decision });
        }
    }
    let Some(facts) = facts else {
        next.released = Some(cause);
        return Some(Stepped { took: None, change: next, decision });
    };
    next.released = None;
    let prior = match predecessor {
        Some((_, hold)) => hold.clone(),
        None => Hold::new(cause, now),
    };
    let mut stepped = reenter(next, &prior, facts, standing?, now, &change.clean);
    if end_hold {
        stepped.decision = Decision::EndHold;
    }
    Some(stepped)
}

fn forget_release(next: &mut Change, was: &State, cause: Cause) {
    next.state = was.clone();
    next.stale = None;
    next.clock = None;
    next.owns_turn = false;
    next.gate_failures = Box::new([]);
    next.clean = Box::new([]);
    match cause {
        Cause::Repairs => next.repairs = 0,
        Cause::Resolutions => next.resolutions = 0,
        Cause::Updates => next.updates = 0,
        Cause::Failed if resolving_state(was) => next.released_resolution = true,
        Cause::Stalled
        | Cause::Runner
        | Cause::RedStands
        | Cause::Questions
        | Cause::BranchMoved
        | Cause::BranchMissing
        | Cause::PullClosed
        | Cause::Retargeted
        | Cause::Failed
        | Cause::CloseRefused
        | Cause::Rejected
        | Cause::Authority(_) => {}
    }
}

fn delegate_wait(state: &State) -> bool {
    match state {
        State::Producing { requested } | State::Repairing { requested, .. } | State::Resolving { requested, .. } => {
            *requested
        }
        State::Taking { .. }
        | State::WaitingProducer { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Updating { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::NoChange { .. }
        | State::Ended
        | State::Held { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. } => false,
    }
}

fn resolving_state(state: &State) -> bool {
    match state {
        State::Resolving { .. } => true,
        State::Taking { .. }
        | State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::Ready { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Preparing { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::Held { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. }
        | State::Ended => false,
    }
}

fn reenter(
    mut change: Change,
    prior: &Hold,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    clean: &[[u8; 32]],
) -> Stepped {
    let why = prior.cause;
    if why == Cause::RedStands
        && let State::Repairing { why: repair, .. } = change.state
    {
        return release_red(change, repair, prior, facts, now);
    }
    if why == Cause::Rejected
        && let State::Gating { head, .. } = change.state
        && facts.branch == Some(head)
        && rejection_stands(&change, head, facts, clean)
    {
        change.clean = Box::from(clean);
        return replay_judgement(change, prior.judged, now, Replay::Rejected { head });
    }
    if why == Cause::CloseRefused && facts.close && change.state == (State::Producing { requested: true }) {
        if !facts.measure
            && let Some(replay) = close_refusal_stands(facts)
        {
            return replay_judgement(change, prior.judged, now, replay);
        }
        change.state = State::Producing { requested: false };
        return Stepped { took: None, change, decision: Decision::None };
    }
    let delegate = if let State::Gating { head, .. } = change.state {
        Delegate::Gate { number: 0, head }
    } else {
        Delegate::Produce
    };
    if !(ci_state(&change.state) && green_head(facts).is_some())
        && let Cause::Authority(prior) = why
        && let Some(current) = missing(standing, delegate)
        && current.lack == prior.lack
    {
        return hold(change, Cause::Authority(AuthorityHold { released: true, ..current }), now);
    }
    let was = if why == Cause::Failed
        && (standing.effect == EffectResult::Failed || standing.effect == EffectResult::None)
        && let State::Landing { head } = change.state
    {
        // The hold gave up the turn. Revalidate the head and rejoin the
        // queue at the original ready time instead of skipping to First.
        State::Gating { head, asked: Box::new([]) }
    } else {
        change.state.clone()
    };
    reenter_phase(change, &was, why, facts, standing)
}

fn release_red(mut change: Change, why: Repair, prior: &Hold, facts: &Facts, now: Wall) -> Stepped {
    let previous = match prior.judged {
        Some(judged) => Some(judged.head),
        None => change.last_head,
    };
    let Some(head) = facts.branch else {
        change.state = State::Producing { requested: false };
        return Stepped { took: None, change, decision: Decision::None };
    };
    change.last_head = Some(head);
    let checked = match facts.pull {
        Pull::Open { .. } => pushed(&change, head),
        Pull::Closed => State::Reopening { requested: false, retarget: false },
        Pull::Missing | Pull::Merged { .. } => State::Opening { requested: false },
    };
    if !matches_open(facts.pull) || Some(head) != previous || (why.ci() && green_head(facts).is_some()) {
        change.state = checked;
        return Stepped { took: None, change, decision: Decision::None };
    }
    if !facts.measure
        && let Some(judged) = prior.judged
    {
        if facts.revision > judged.spec {
            change.state = State::Repairing { requested: false, why };
            return Stepped { took: None, change, decision: Decision::None };
        }
        if facts.recheck == Recheck::Taken {
            return replay_judgement(change, prior.judged, now, Replay::RedStands { head });
        }
    }
    if why.ci() && facts.rerun && facts.ci.head == head && facts.ci.status == Status::Failed {
        change.state = State::Checking { head };
        return rerun(change, head, now);
    }
    if facts.regate {
        change.state = checked;
        return Stepped { took: None, change, decision: Decision::None };
    }
    red_stands(change, why, facts, now)
}

fn matches_open(pull: Pull) -> bool {
    let Pull::Open { .. } = pull else { return false };
    true
}

fn rejection_stands(change: &Change, head: [u8; 32], facts: &Facts, clean: &[[u8; 32]]) -> bool {
    // A judgement retains the facts used to read its verdict, even though the
    // re-entry itself forgets old clean heads and the gate's asks and tries.
    for gate in &change.gates {
        if gate.kind == GateKind::Person
            && gate.blocking
            && gate_verdict(*gate, &facts.gates, head, facts.revision, clean) == Status::Failed
        {
            return true;
        }
    }
    false
}

fn replay_judgement(change: Change, judged: Option<Judged>, now: Wall, replay: Replay) -> Stepped {
    let mut stepped = hold(change, replay.cause(), now);
    if let State::Held { hold, .. } = &mut stepped.change.state {
        hold.judged = judged;
        stepped.decision = Decision::ReplayJudgement(replay);
    }
    stepped
}

fn reenter_phase(mut change: Change, was: &State, why: Cause, facts: &Facts, standing: &Heard) -> Stepped {
    match why {
        Cause::PullClosed => {
            change.state = State::Reopening { requested: true, retarget: false };
            Stepped { took: None, change, decision: Decision::Effect(Effect::Reopen) }
        }
        Cause::Retargeted => {
            change.state = State::Reopening { requested: true, retarget: true };
            Stepped { took: None, change, decision: Decision::Effect(Effect::Retarget) }
        }
        Cause::BranchMissing => match change.last_head {
            Some(head) => {
                change.state = State::Recreating { head };
                Stepped { took: None, change, decision: Decision::Effect(Effect::CreateBranch { head }) }
            }
            None => {
                change.state = if facts.requester_branch {
                    State::Opening { requested: false }
                } else {
                    State::Producing { requested: false }
                };
                Stepped { took: None, change, decision: Decision::None }
            }
        },
        Cause::BranchMoved => {
            if let State::WaitingProducer { .. } = was {
                change.state = reentry(&mut change, was, why, facts, standing);
                return Stepped { took: None, change, decision: Decision::None };
            }
            change.state = match facts.branch {
                Some(head) => {
                    change.last_head = Some(head);
                    match facts.pull {
                        Pull::Open { .. } => pushed(&change, head),
                        Pull::Closed => State::Reopening { requested: false, retarget: false },
                        Pull::Missing | Pull::Merged { .. } => State::Opening { requested: false },
                    }
                }
                None => State::Producing { requested: false },
            };
            Stepped { took: None, change, decision: Decision::None }
        }
        Cause::Stalled
        | Cause::Runner
        | Cause::RedStands
        | Cause::Repairs
        | Cause::Questions
        | Cause::Resolutions
        | Cause::Updates
        | Cause::Failed
        | Cause::CloseRefused
        | Cause::Rejected
        | Cause::Authority(_) => {
            change.state = reentry(&mut change, was, why, facts, standing);
            Stepped { took: None, change, decision: Decision::None }
        }
    }
}

fn reentry(change: &mut Change, was: &State, why: Cause, facts: &Facts, standing: &Heard) -> State {
    let pull = facts.pull;
    let open = match pull {
        Pull::Open { .. } => true,
        Pull::Missing | Pull::Closed | Pull::Merged { .. } => false,
    };
    match was {
        State::Opening { .. } | State::Reopening { .. } if standing.effect == EffectResult::Pending => was.clone(),
        State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::Ready { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Opening { .. }
        | State::Reopening { .. }
            if facts.branch.is_none() =>
        {
            State::Producing { requested: false }
        }
        State::Checking { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::Ready { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Opening { .. }
        | State::Reopening { .. }
            if !open =>
        {
            change.last_head = facts.branch;
            match pull {
                Pull::Closed => State::Reopening { requested: false, retarget: false },
                Pull::Missing | Pull::Open { .. } | Pull::Merged { .. } => State::Opening { requested: false },
            }
        }
        State::Checking { head }
        | State::Ready { head, .. }
        | State::Queued { head, .. }
        | State::First { head, .. }
        | State::Judging { head, .. }
        | State::Gating { head, .. }
            if facts.drift.is_none() && facts.branch != Some(*head) =>
        {
            let observed = facts.branch.expect("release read a branch");
            change.last_head = Some(observed);
            pushed(change, observed)
        }
        State::Opening { .. } | State::Reopening { .. } => {
            let head = facts.branch.expect("release read a branch with an open pull");
            change.last_head = Some(head);
            pushed(change, head)
        }
        State::Ready { head, ready_since } | State::Judging { head, ready_since, .. }
            if facts.ask.is_some() && change.way == Way::Turn =>
        {
            State::Ready { head: *head, ready_since: *ready_since }
        }
        State::Ready { head, .. }
        | State::Queued { head, .. }
        | State::First { head, .. }
        | State::Judging { head, .. }
        | State::Gating { head, .. } => State::Gating { head: *head, asked: Box::new([]) },
        State::Held { .. }
        | State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. }
        | State::Ended => {
            unreachable!("held state retains a live predecessor")
        }
        // A release decides from branch, base and pull, not the ended resolution's
        // status (temper/docs/design/domain/forge/change/repairs.md, A release).
        State::Resolving { base, .. } if why == Cause::Failed => State::WaitingProducer { base: *base },
        State::Taking { .. } => State::Taking { requested: false },
        State::Producing { requested: true } if why == Cause::Failed || why == Cause::CloseRefused => {
            State::Producing { requested: false }
        }
        State::Repairing { why: repair, .. } if why == Cause::Failed => {
            State::Repairing { requested: false, why: *repair }
        }
        State::Producing { .. }
        | State::Recreating { .. }
        | State::Checking { .. }
        | State::Preparing { .. }
        | State::CancellingGates { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. } => was.clone(),
    }
}

fn waiting(mut change: Change, facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    if let State::Producing { .. } = change.state
        && !facts.pending.is_empty()
    {
        change.since = now;
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    let until = Wall::from_nanos(change.since.as_nanos().saturating_add(limits.stall.as_nanos()));
    if now >= until {
        hold(change, Cause::Stalled, now)
    } else {
        Stepped { took: None, change, decision: Decision::Wait { until: Some(until) } }
    }
}

fn waiting_on_agent(mut change: Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    if facts.asking {
        change.since = now;
        Stepped { took: None, change, decision: Decision::Wait { until: None } }
    } else if standing.delegate_live {
        change.since = now;
        let until = Wall::from_nanos(now.as_nanos().saturating_add(limits.stall.as_nanos()));
        Stepped { took: None, change, decision: Decision::Wait { until: Some(until) } }
    } else {
        waiting(change, facts, now, limits)
    }
}

fn settling(change: Change, facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    let stall = Wall::from_nanos(change.since.as_nanos().saturating_add(limits.stall.as_nanos()));
    if let Some(end) = facts.settling
        && now < end.min(stall)
    {
        Stepped { took: None, change, decision: Decision::Wait { until: Some(end.min(stall)) } }
    } else {
        waiting(change, facts, now, limits)
    }
}

fn hold(mut change: Change, why: Cause, now: Wall) -> Stepped {
    if change.state == State::Ended {
        return Stepped { took: None, change, decision: Decision::Cancel };
    }
    if let State::Held { .. } = change.state {
        return Stepped { took: None, change, decision: Decision::None };
    }
    change.owns_turn = false;
    change.state = State::Held { was: Box::new(change.state), hold: Hold::new(why, now) };
    Stepped { took: None, change, decision: Decision::Hold(why) }
}

// Taking makes only the head the connector kept (temper/docs/design/domain/forge/change/opened.md, 4).
fn taking(mut change: Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    let Some(head) = facts.taken else { return waiting(change, facts, now, limits) };
    if let Some(observed) = facts.branch {
        if observed != head {
            return waiting(change, facts, now, limits);
        }
        if standing.effect != EffectResult::Pending {
            change.last_head = Some(head);
            change.state = State::Opening { requested: false };
            change.since = now;
            return Stepped { took: None, change, decision: Decision::None };
        }
    }
    match standing.effect {
        EffectResult::Pending => waiting(change, facts, now, limits),
        EffectResult::Failed | EffectResult::Conflict => hold(change, Cause::Failed, now),
        EffectResult::None | EffectResult::Stale | EffectResult::Made => {
            change.state = State::Taking { requested: true };
            Stepped { took: None, change, decision: Decision::Effect(Effect::CreateBranch { head }) }
        }
    }
}

fn producing(
    mut change: Change,
    requested: bool,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if facts.close {
        return producing_close(change, requested, facts, standing, now, limits);
    }
    if !requested && facts.follows && !facts.pending.is_empty() {
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    match &change.early {
        Early::Started(heads) => {
            if standing.start_conflict.is_some() {
                change.early = Early::Spent(heads.clone());
                change.state = State::Producing { requested: false };
                return Stepped { took: None, change, decision: Decision::CancelProducer };
            }
            if requested && standing.delegate == Status::Failed {
                change.early = Early::Spent(heads.clone());
                change.state = State::Producing { requested: false };
                return Stepped { took: None, change, decision: Decision::Wait { until: None } };
            }
        }
        Early::Spent(_) => {
            if !requested {
                return request_producer(change, facts, standing, now);
            }
            match standing.delegate {
                Status::Failed => return hold(change, Cause::Failed, now),
                Status::Unknown | Status::Pending => return waiting_on_agent(change, facts, standing, now, limits),
                Status::Passed => {}
            }
        }
        Early::None => {}
    }
    // A failed producer holds whatever it delivered (domain/forge/change/repairs.md).
    if requested && standing.delegate == Status::Failed {
        return hold(change, Cause::Failed, now);
    }
    match facts.branch {
        Some(head) if facts.titled => {
            change.state = State::Opening { requested: false };
            change.last_head = Some(head);
            change.since = now;
            Stepped { took: None, change, decision: Decision::None }
        }
        Some(_) if !requested => request_producer(change, facts, standing, now),
        Some(_) => waiting_on_agent(change, facts, standing, now, limits),
        None if !requested => {
            if let Some(commit) = facts.catch_up {
                return catch_up_due(change, commit, facts, now, limits);
            }
            request_producer(change, facts, standing, now)
        }
        None => match standing.delegate {
            Status::Passed => match facts.delegate {
                Some(Delegate::Merge { base, .. }) => {
                    let mut stepped = resolving_due(change, base, facts, standing, now, limits);
                    if stepped.decision == Decision::Delegate(Delegate::Resolve { base }) {
                        stepped.change.state = State::Producing { requested: true };
                    }
                    stepped
                }
                Some(
                    Delegate::Produce
                    | Delegate::Close
                    | Delegate::Repair(_)
                    | Delegate::Resolve { .. }
                    | Delegate::Gate { .. },
                )
                | None => waiting(change, facts, now, limits),
            },
            Status::Failed => hold(change, Cause::Failed, now),
            Status::Unknown | Status::Pending => waiting_on_agent(change, facts, standing, now, limits),
        },
    }
}

fn producing_close(
    mut change: Change,
    requested: bool,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if requested {
        match standing.delegate {
            Status::Failed => {
                let cause = match facts.close_at {
                    Some(at) => at.failed_cause(),
                    None => Cause::Failed,
                };
                return hold(change, cause, now);
            }
            Status::Pending | Status::Unknown => return waiting_on_agent(change, facts, standing, now, limits),
            Status::Passed => {}
        }
    }
    if !requested && standing.closer_waiting {
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    if let Some(head) = facts.branch
        && standing.delegate != Status::Failed
    {
        change.state = State::Opening { requested: false };
        change.last_head = Some(head);
        change.since = now;
        return Stepped { took: None, change, decision: Decision::None };
    }
    if requested {
        if facts.delegate == Some(Delegate::Close) {
            change.state = State::NoChange { words: copy_of(b"Nothing to settle.") };
            return Stepped { took: None, change, decision: Decision::NoChange };
        }
        return waiting(change, facts, now, limits);
    }
    let Some(_) = facts.close_tips else {
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    };
    change.state = State::Producing { requested: true };
    if let Some(replay) = close_refusal_stands(facts) {
        return replay_judgement(change, None, now, replay);
    }
    change.since = now;
    Stepped { took: None, change, decision: Decision::Delegate(Delegate::Close) }
}

fn close_refusal_stands(facts: &Facts) -> Option<Replay> {
    let at = facts.close_at?;
    let tips = facts.close_tips?;
    if at.refused.is_some() && at.branch == tips.branch && at.landing == tips.landing {
        Some(Replay::CloseRefused { branch: tips.branch, landing: tips.landing })
    } else {
        None
    }
}

// A catch-up's producer is the merge job (temper/docs/design/domain/forge/change/README.md, 2, temper/docs/design/domain/forge/change/repairs.md).
fn catch_up_due(mut change: Change, commit: [u8; 32], facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    match facts.contains_catch_up {
        Status::Passed => {
            change.state = State::NoChange { words: contained_words(b"commit ", commit) };
            Stepped { took: None, change, decision: Decision::NoChange }
        }
        Status::Failed => {
            change.state = State::Producing { requested: true };
            Stepped { took: None, change, decision: Decision::Delegate(Delegate::Merge { base: commit, from: None }) }
        }
        Status::Unknown | Status::Pending => waiting(change, facts, now, limits),
    }
}

fn contained_words(subject: &[u8], commit: [u8; 32]) -> Box<[u8]> {
    let mut words = List::with_capacity(128);
    put(&mut words, b"The landing branch already contains ");
    put(&mut words, subject);
    let mut length = 20;
    for byte in commit.get(20..).expect("a commit has 32 bytes") {
        if *byte != 0 {
            length = 32;
            break;
        }
    }
    for byte in commit.get(..length).expect("a commit has 20 or 32 bytes") {
        words.push(skein_lib::bytes::hex_digit(*byte >> 4)).expect("containment words fit");
        words.push(skein_lib::bytes::hex_digit(*byte)).expect("containment words fit");
    }
    put(&mut words, b".");
    words.into_boxed()
}

fn request_producer(mut change: Change, facts: &Facts, standing: &Heard, now: Wall) -> Stepped {
    if !producer_start(&mut change, facts, standing) {
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    change.state = State::Producing { requested: true };
    change.since = now;
    Stepped { took: None, change, decision: Decision::Delegate(Delegate::Produce) }
}

// Dependency waits arm no stall; producing starts its clock when the wait ends
// (temper/docs/design/domain/forge/change/questions.md, 1).
fn producer_start(change: &mut Change, facts: &Facts, standing: &Heard) -> bool {
    match change.early {
        Early::Started(_) | Early::Spent(_) => {
            // A refused preparation still owns its delegate until the top sees its end.
            return facts.pending.is_empty() && standing.start_conflict.is_none();
        }
        Early::None => {}
    }
    if facts.pending.is_empty() {
        return true;
    }
    let capacity = u32::try_from(facts.pending.len()).expect("pending dependencies fit u32");
    let mut heads = List::with_capacity(capacity);
    for pending in &facts.pending {
        match pending.kind {
            PendingKind::Change { checked: Some(head), kept: false } => {
                heads.push(StartedFrom { task: pending.task, head }).expect("one head per pending dependency");
            }
            PendingKind::Elsewhere => {}
            PendingKind::Change { .. } | PendingKind::Other => return false,
        }
    }
    change.early = Early::Started(heads.into_boxed());
    true
}

fn opening(
    mut change: Change,
    requested: bool,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    let Some(head) = facts.branch else {
        return waiting(change, facts, now, limits);
    };
    change.last_head = Some(head);
    match facts.pull {
        Pull::Open { .. } => {
            change.state = pushed(&change, head);
            change.since = now;
            Stepped { took: None, change, decision: Decision::None }
        }
        Pull::Missing if !requested => {
            change.state = State::Opening { requested: true };
            change.since = now;
            Stepped { took: None, change, decision: Decision::Effect(Effect::Open) }
        }
        Pull::Missing => match standing.effect {
            EffectResult::Failed | EffectResult::Conflict => hold(change, Cause::Failed, now),
            EffectResult::None | EffectResult::Stale | EffectResult::Pending | EffectResult::Made => {
                waiting(change, facts, now, limits)
            }
        },
        Pull::Closed | Pull::Merged { .. } => waiting(change, facts, now, limits),
    }
}

fn recreating(mut change: Change, head: [u8; 32], facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    match facts.branch {
        Some(observed) if observed == head => {
            change.state = match facts.pull {
                Pull::Closed => State::Reopening { requested: false, retarget: false },
                Pull::Missing | Pull::Open { .. } | Pull::Merged { .. } => State::Opening { requested: false },
            };
            Stepped { took: None, change, decision: Decision::None }
        }
        Some(_) | None => waiting(change, facts, now, limits),
    }
}

fn reopening(
    mut change: Change,
    requested: bool,
    retarget: bool,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    let Some(head) = facts.branch else {
        return waiting(change, facts, now, limits);
    };
    match facts.pull {
        Pull::Open { .. } => {
            change.state = pushed(&change, head);
            change.since = now;
            Stepped { took: None, change, decision: Decision::None }
        }
        Pull::Closed | Pull::Missing if !requested => {
            change.state = State::Reopening { requested: true, retarget };
            let effect = if retarget { Effect::Retarget } else { Effect::Reopen };
            Stepped { took: None, change, decision: Decision::Effect(effect) }
        }
        Pull::Closed | Pull::Missing => match standing.effect {
            EffectResult::Failed | EffectResult::Conflict => hold(change, Cause::Failed, now),
            EffectResult::None | EffectResult::Stale | EffectResult::Pending | EffectResult::Made => {
                waiting(change, facts, now, limits)
            }
        },
        Pull::Merged { .. } => unreachable!("merged pull returned above"),
    }
}

fn runner_ci(change: &Change, head: [u8; 32], facts: &Facts, now: Wall, limits: &Limits) -> Option<Stepped> {
    match facts.fault {
        crate::Fault::Runner { reruns } if reruns < limits.reruns => Some(rerun(change.clone(), head, now)),
        crate::Fault::Runner { .. } => Some(hold(change.clone(), Cause::Runner, now)),
        crate::Fault::Change => None,
    }
}

fn rerun(mut change: Change, head: [u8; 32], now: Wall) -> Stepped {
    change.since = now;
    Stepped { took: None, change, decision: Decision::Effect(Effect::Rerun { head }) }
}

fn ci_recheck_reason(facts: &Facts) -> Repair {
    if facts.semantic_recheck { Repair::Semantic } else { Repair::Ci }
}

fn checking(
    mut change: Change,
    head: [u8; 32],
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if facts.branch.is_none() {
        return waiting(change, facts, now, limits);
    }
    change.last_head = Some(head);
    if facts.ci.head != head {
        return waiting(change, facts, now, limits);
    }
    if facts.recheck == Recheck::Owed {
        match standing.effect {
            EffectResult::Failed | EffectResult::Conflict if facts.ci.status == Status::Failed => {
                return red_stands(change, ci_recheck_reason(facts), facts, now);
            }
            EffectResult::Failed | EffectResult::Conflict if facts.ci.status == Status::Passed => {
                return repair_ended(change, head, now);
            }
            EffectResult::Failed
            | EffectResult::Conflict
            | EffectResult::None
            | EffectResult::Stale
            | EffectResult::Pending
            | EffectResult::Made => return wait_for_ci(change, head, facts, now, limits),
        }
    }
    match facts.ci.status {
        Status::Passed => {
            change.state = State::Gating { head, asked: Box::new([]) };
            change.since = now;
            Stepped { took: None, change, decision: Decision::None }
        }
        Status::Failed => {
            if let Some(stepped) = runner_ci(&change, head, facts, now, limits) {
                return stepped;
            }
            if facts.recheck == Recheck::Taken {
                return red_stands(change, ci_recheck_reason(facts), facts, now);
            }
            let why = if change.clean.is_empty() { Repair::Ci } else { Repair::Semantic };
            repair_due(change, why, facts, now, limits)
        }
        Status::Unknown | Status::Pending => {
            if facts.recheck == Recheck::Taken {
                return wait_for_ci(change, head, facts, now, limits);
            }
            if behind_unchecked(&change, facts, limits) {
                let ready_since = change.ready_since.unwrap_or(now);
                return update_due(change, head, facts.base_tip, ready_since, now, limits);
            }
            if has_eager_gate(&change) {
                change.state = State::Gating { head, asked: Box::new([]) };
                Stepped { took: None, change, decision: Decision::None }
            } else {
                wait_for_ci(change, head, facts, now, limits)
            }
        }
    }
}

fn wait_for_ci(mut change: Change, head: [u8; 32], facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    if let Some(reading) = facts.execution {
        let clock = crate::Clock::observe(change.clock, change.since, head, reading, now);
        change.clock = Some(clock);
        checking_clock(change, clock, facts, now, limits)
    } else {
        settling(change, facts, now, limits)
    }
}

/// Bound execution and runner wait without charging queued time to running
/// (temper/docs/design/domain/forge/change/stalls.md, 2.3).
fn checking_clock(change: Change, clock: crate::Clock, facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    let remaining = clock.remaining(now, limits);
    let stalled = remaining == Duration::ZERO;
    let due = Wall::from_nanos(now.as_nanos().saturating_add(remaining.as_nanos()));
    if let Some(end) = facts.settling
        && now < end.min(due)
    {
        return Stepped { took: None, change, decision: Decision::Wait { until: Some(end.min(due)) } };
    }
    if stalled {
        hold(change, Cause::Stalled, now)
    } else {
        Stepped { took: None, change, decision: Decision::Wait { until: Some(due) } }
    }
}

/// A head already behind takes its base before CI concludes on it, once for
/// a head a push gave it, and never with the change's last update
/// (temper/docs/design/domain/forge/change/README.md, 2, step 3).
fn behind_unchecked(change: &Change, facts: &Facts, limits: &Limits) -> bool {
    facts.contains_base == Status::Failed
        && change.clean.is_empty()
        && change.updates.saturating_add(1) < limits.updates
}

fn has_eager_gate(change: &Change) -> bool {
    for gate in &change.gates {
        if gate.eager {
            return true;
        }
    }
    false
}

fn valid(gate: Gate, report: crate::GateReport, head: [u8; 32], revision: u64, clean: &[[u8; 32]]) -> bool {
    if report.number != gate.number || !report.at_revision(gate.kind, revision) {
        return false;
    }
    if report.head == head {
        return true;
    }
    match gate.freshness {
        Freshness::Exact => false,
        Freshness::Clean => clean.contains(&report.head),
    }
}

fn verdict(change: &Change, gate: Gate, facts: &Facts, head: [u8; 32]) -> Status {
    gate_verdict(gate, &facts.gates, head, facts.revision, &change.clean)
}

fn gate_verdict(
    gate: Gate,
    reports: &[crate::GateReport],
    head: [u8; 32],
    revision: u64,
    clean: &[[u8; 32]],
) -> Status {
    let mut passed = false;
    let mut pending = false;
    let mut failed = false;
    for report in reports {
        if !valid(gate, *report, head, revision, clean) {
            continue;
        }
        match report.status {
            Status::Passed => passed = true,
            Status::Pending => pending = true,
            Status::Failed => failed = true,
            Status::Unknown => {}
        }
    }
    if failed {
        Status::Failed
    } else if pending {
        Status::Pending
    } else if passed {
        Status::Passed
    } else {
        Status::Unknown
    }
}

fn gating(mut change: Change, head: [u8; 32], asked: &[u64], facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    if facts.branch.is_none() {
        return waiting(change, facts, now, limits);
    }
    if change.way == Way::Queue && facts.ci.head == head && facts.ci.status == Status::Failed {
        if let Some(stepped) = runner_ci(&change, head, facts, now, limits) {
            return stepped;
        }
        if facts.live_gate == Some(head) {
            change.state = State::CancellingGates { head };
            change.owns_turn = false;
            return Stepped { took: None, change, decision: Decision::CancelGates { head } };
        }
        let why = if change.clean.is_empty() { Repair::Ci } else { Repair::Semantic };
        return repair_due(change, why, facts, now, limits);
    }
    for failure in &change.gate_failures {
        if failure.count >= 2 {
            for gate in &change.gates {
                if failure.number == gate.number {
                    return hold(change, Cause::Failed, now);
                }
            }
        }
    }
    let ci_passed = change.way == Way::Turn || (facts.ci.head == head && facts.ci.status == Status::Passed);
    let mut waiting_gate = false;
    for index in 0..change.gates.len() {
        let gate = *change.gates.get(index).expect("index is in bounds");
        if !ci_passed && !gate.eager {
            continue;
        }
        match verdict(&change, gate, facts, head) {
            Status::Failed if gate.blocking => match gate.kind {
                GateKind::Person => return hold(change, Cause::Rejected, now),
                GateKind::Agent | GateKind::Check | GateKind::Follow => {
                    return repair_due(change, Repair::Gate(gate.number), facts, now, limits);
                }
            },
            Status::Unknown
                if !asked.contains(&gate.number)
                    || (facts.live_gate.is_none() && stale_review(gate, &facts.gates, facts.revision)) =>
            {
                change.owns_turn = false;
                // The top owns one delegate slot: retain it until that gate ends.
                if facts.live_gate.is_some() {
                    return waiting(change, facts, now, limits);
                }
                let mut collected = List::with_capacity(limits.gates);
                for number in asked {
                    collected.push(*number).expect("asked gates fit the limit");
                }
                if !asked.contains(&gate.number) {
                    collected.push(gate.number).expect("gate admission checked");
                }
                change.state = State::Gating { head, asked: collected.into_boxed() };
                change.since = now;
                return Stepped {
                    took: None,
                    change,
                    decision: Decision::Delegate(Delegate::Gate { number: gate.number, head }),
                };
            }
            Status::Unknown | Status::Pending if gate.blocking => waiting_gate = true,
            Status::Failed | Status::Passed | Status::Unknown | Status::Pending => {}
        }
    }
    if !ci_passed || waiting_gate {
        return waiting(change, facts, now, limits);
    }
    if facts.kept {
        change.owns_turn = false;
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    let since = match change.ready_since {
        Some(since) => since,
        None => {
            change.ready_since = Some(now);
            now
        }
    };
    let mut took = None;
    if change.way == Way::Turn {
        change.state = State::Ready { head, ready_since: since };
        took = dropped_ask(head, facts);
    } else if change.owns_turn {
        change.state = State::First { head, ready_since: since };
    } else {
        change.state = State::Queued { head, ready_since: since };
    }
    change.since = now;
    Stepped { took, change, decision: Decision::Ready }
}

fn cancelling_gates(mut change: Change, head: [u8; 32], facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    if facts.branch.is_none() {
        return waiting(change, facts, now, limits);
    }
    if facts.ci.head == head
        && facts.ci.status == Status::Failed
        && let Some(stepped) = runner_ci(&change, head, facts, now, limits)
    {
        return stepped;
    }
    if facts.live_gate == Some(head) {
        return waiting(change, facts, now, limits);
    }
    if let Some(head) = green_head(facts) {
        return take_pushed_head(change, head, now);
    }
    if facts.ci.head != head || facts.ci.status == Status::Pending || facts.ci.status == Status::Unknown {
        return waiting(change, facts, now, limits);
    }
    change.owns_turn = false;
    let why = if change.clean.is_empty() { Repair::Ci } else { Repair::Semantic };
    repair_due(change, why, facts, now, limits)
}

fn queued(
    mut change: Change,
    head: [u8; 32],
    ready_since: Wall,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if facts.branch.is_none() {
        return waiting(change, facts, now, limits);
    }
    if let Some(reviewed) = standing.review_request {
        return repair_due(change, Repair::Review { head: reviewed }, facts, now, limits);
    }
    if facts.kept
        || facts.ci.head != head
        || facts.ci.status != Status::Passed
        || !blocking_gates_pass(&change, facts, head)
        || stale_reviews(&change, facts)
    {
        change.state = State::Gating { head, asked: Box::new([]) };
        change.owns_turn = false;
        return Stepped { took: None, change, decision: Decision::None };
    }
    if facts.first {
        change.state = State::First { head, ready_since };
        change.since = now;
        change.owns_turn = true;
        Stepped { took: None, change, decision: Decision::None }
    } else {
        Stepped { took: None, change, decision: Decision::Wait { until: None } }
    }
}

// Re-asking a stale review is not a failed review (gates.md, 1, Valid at a spec revision).
fn stale_review(gate: Gate, reports: &[crate::GateReport], revision: u64) -> bool {
    let mut stale = false;
    for report in reports {
        if report.number == gate.number {
            if report.at_revision(gate.kind, revision) {
                return false;
            }
            stale = true;
        }
    }
    stale
}

fn stale_reviews(change: &Change, facts: &Facts) -> bool {
    for gate in &change.gates {
        if !gate.blocking && facts.live_gate.is_some() {
            continue;
        }
        if stale_review(*gate, &facts.gates, facts.revision) {
            return true;
        }
    }
    false
}

pub(crate) fn blocking_gates_pass(change: &Change, facts: &Facts, head: [u8; 32]) -> bool {
    gates_pass(change, &facts.gates, head, facts.revision)
}

/// Every blocking gate has a passing verdict valid at this head (domain/forge/change/gates.md, 1).
#[must_use]
pub fn gates_pass(change: &Change, reports: &[crate::GateReport], head: [u8; 32], revision: u64) -> bool {
    for gate in &change.gates {
        if gate.blocking && gate_verdict(*gate, reports, head, revision, &change.clean) != Status::Passed {
            return false;
        }
    }
    true
}

fn first(
    mut change: Change,
    head: [u8; 32],
    ready_since: Wall,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if facts.branch.is_none() {
        return waiting(change, facts, now, limits);
    }
    if let Some(stepped) = contained_landing(&change, facts, standing, now) {
        return stepped;
    }
    if let Some(reviewed) = standing.review_request {
        return repair_due(change, Repair::Review { head: reviewed }, facts, now, limits);
    }
    if facts.kept
        || facts.ci.head != head
        || facts.ci.status != Status::Passed
        || !blocking_gates_pass(&change, facts, head)
        || stale_reviews(&change, facts)
    {
        change.state = State::Gating { head, asked: Box::new([]) };
        change.owns_turn = false;
        return Stepped { took: None, change, decision: Decision::None };
    }
    if facts.base_ci.head == facts.base_tip && facts.base_ci.status == Status::Failed {
        return if facts.queue_repair_active {
            Stepped { took: None, change, decision: Decision::Wait { until: None } }
        } else {
            Stepped { took: None, change, decision: Decision::QueueRepair }
        };
    }
    match facts.contains_base {
        Status::Failed => return update_due(change, head, facts.base_tip, ready_since, now, limits),
        Status::Unknown | Status::Pending => return waiting(change, facts, now, limits),
        Status::Passed => {}
    }
    match facts.mergeable {
        Status::Passed => {
            if !facts.may_merge {
                return Stepped { took: None, change, decision: Decision::Wait { until: None } };
            }
            change.state = State::Landing { head };
            change.since = now;
            Stepped {
                took: None,
                change,
                decision: Decision::Effect(Effect::Merge { head, base: facts.expected_base }),
            }
        }
        Status::Failed => conflict_due(change, facts.base_tip, Some(head), facts, standing, now, limits),
        Status::Unknown | Status::Pending => settling(change, facts, now, limits),
    }
}

#[expect(clippy::too_many_arguments, reason = "the pending update keeps both heads and the ready time")]
fn updating(
    mut change: Change,
    from: [u8; 32],
    base: [u8; 32],
    ready_since: Wall,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    match standing.effect {
        EffectResult::Stale => stale_update(change, facts, now, limits),
        EffectResult::Conflict => conflict_due(change, base, Some(from), facts, standing, now, limits),
        EffectResult::Failed => hold(change, Cause::Failed, now),
        EffectResult::Made => {
            let Some(head) = facts.branch else { return waiting(change, facts, now, limits) };
            if head == from {
                return waiting(change, facts, now, limits);
            }
            updated(change, from, head, now, limits)
        }
        EffectResult::None => {
            let Some(head) = facts.branch else { return waiting(change, facts, now, limits) };
            match facts.contains_base {
                Status::Unknown | Status::Pending => waiting(change, facts, now, limits),
                Status::Failed => {
                    if head == from {
                        update_due(change, from, facts.base_tip, ready_since, now, limits)
                    } else {
                        take_pushed_head(change, head, now)
                    }
                }
                Status::Passed => {
                    if head != from {
                        return updated(change, from, head, now, limits);
                    }
                    change.state = pushed(&change, head);
                    change.last_head = Some(head);
                    change.since = now;
                    Stepped { took: None, change, decision: Decision::None }
                }
            }
        }
        EffectResult::Pending => waiting(change, facts, now, limits),
    }
}

// A refusal before any send spends stall, not another update (domain/forge/updates.md, 1).
fn stale_update(mut change: Change, facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    let (from, pin, ready_since, preparing) = if let State::Preparing { from, main, ready_since } = change.state {
        (from, main, ready_since, true)
    } else {
        let State::Updating { from, base, ready_since } = change.state else { unreachable!("stale update") };
        (from, base, ready_since, false)
    };
    let stale = *change.stale.get_or_insert(now);
    let deadline = Wall::from_nanos(stale.as_nanos().saturating_add(limits.stall.as_nanos()));
    if now >= deadline {
        return hold(change, Cause::Stalled, now);
    }
    let Some(head) = facts.branch else {
        return Stepped { took: None, change, decision: Decision::Wait { until: Some(deadline) } };
    };
    if head != from {
        return take_pushed_head(change, head, now);
    }
    match facts.contains_base {
        Status::Unknown | Status::Pending => {
            Stepped { took: None, change, decision: Decision::Wait { until: Some(deadline) } }
        }
        Status::Passed => {
            if preparing {
                return prepared(change, from, pin, ready_since, facts, now, limits);
            }
            change.state = pushed(&change, head);
            change.last_head = Some(head);
            change.since = now;
            Stepped { took: None, change, decision: Decision::None }
        }
        Status::Failed => {
            if facts.base_read != pin {
                return send_update(change, from, facts.base_read, ready_since, now, preparing);
            }
            let delay = change.since.as_nanos().saturating_sub(stale.as_nanos()).clamp(1_000_000_000, 60_000_000_000);
            let retry = Wall::from_nanos(change.since.max(stale).as_nanos().saturating_add(delay));
            if now < retry {
                return Stepped { took: None, change, decision: Decision::Wait { until: Some(retry.min(deadline)) } };
            }
            send_update(change, from, pin, ready_since, now, preparing)
        }
    }
}

fn send_update(
    mut change: Change,
    head: [u8; 32],
    base: [u8; 32],
    ready_since: Wall,
    now: Wall,
    preparing: bool,
) -> Stepped {
    let effect = if preparing {
        change.state = State::Preparing { from: head, main: base, ready_since };
        Effect::Prepare { head, main: base }
    } else {
        change.state = State::Updating { from: head, base, ready_since };
        Effect::Update { head, base }
    };
    change.since = now;
    Stepped { took: None, change, decision: Decision::Effect(effect) }
}

fn updated(mut change: Change, from: [u8; 32], head: [u8; 32], now: Wall, limits: &Limits) -> Stepped {
    if change.clean.len() >= skein_lib::usize_of(limits.clean_heads) {
        return hold(change, Cause::Updates, now);
    }
    let mut clean = List::with_capacity(limits.clean_heads);
    for predecessor in &change.clean {
        clean.push(*predecessor).expect("clean history admitted");
    }
    clean.push(from).expect("clean history has room");
    change.clean = clean.into_boxed();
    change.state = pushed(&change, head);
    change.last_head = Some(head);
    change.since = now;
    Stepped { took: None, change, decision: Decision::None }
}

fn repair_due(mut change: Change, why: Repair, facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    change.owns_turn = false;
    if change.way == Way::Queue {
        match facts.contains_base {
            Status::Failed => match facts.branch {
                Some(head) => {
                    let ready_since = change.ready_since.unwrap_or(now);
                    return update_due(change, head, facts.base_tip, ready_since, now, limits);
                }
                None => return waiting(change, facts, now, limits),
            },
            Status::Unknown | Status::Pending => return waiting(change, facts, now, limits),
            Status::Passed => {}
        }
    }
    if change.repairs >= limits.repairs {
        return hold(change, Cause::Repairs, now);
    }
    change.repairs = change.repairs.checked_add(1).expect("repair count below limit");
    match why {
        Repair::Semantic => {}
        Repair::Ci | Repair::Gate(_) | Repair::LandingAt { .. } | Repair::Landing { .. } | Repair::Review { .. } => {
            change.clean = Box::new([]);
        }
    }
    change.state = State::Repairing { requested: true, why };
    change.since = now;
    Stepped { took: None, change, decision: Decision::Delegate(Delegate::Repair(why)) }
}

fn update_due(
    mut change: Change,
    head: [u8; 32],
    base: [u8; 32],
    ready_since: Wall,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if change.updates >= limits.updates {
        return hold(change, Cause::Updates, now);
    }
    change.updates = change.updates.checked_add(1).expect("update count below limit");
    send_update(change, head, base, ready_since, now, false)
}

// Each conflict counts once, including a job followed by a resolution (temper/docs/design/domain/forge/change/repairs.md).
fn conflict_due(
    change: Change,
    base: [u8; 32],
    from: Option<[u8; 32]>,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    let mut stepped = resolving_due(change, base, facts, standing, now, limits);
    if facts.merge_job && stepped.decision == Decision::Delegate(Delegate::Resolve { base }) {
        stepped.decision = Decision::Delegate(Delegate::Merge { base, from });
    }
    stepped
}

fn resolving_due(
    mut change: Change,
    base: [u8; 32],
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if standing.delegate_live {
        change.state = State::WaitingProducer { base };
        change.owns_turn = false;
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    change.last_head = facts.branch;
    if change.resolutions >= limits.resolutions {
        return hold(change, Cause::Resolutions, now);
    }
    change.resolutions = change.resolutions.checked_add(1).expect("resolution count below limit");
    change.owns_turn = false;
    change.clean = Box::new([]);
    change.state = State::Resolving { requested: true, base };
    change.released_resolution = false;
    change.since = now;
    Stepped { took: None, change, decision: Decision::Delegate(Delegate::Resolve { base }) }
}

fn repairing(
    change: Change,
    requested: bool,
    why: Repair,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if why.ci()
        && let Some(head) = green_head(facts)
    {
        if !requested
            || standing.delegate == Status::Failed
            || (standing.delegate == Status::Passed && facts.branch == change.last_head)
        {
            return repair_ended(change, head, now);
        }
        // Passed stands as a delivery below. An unmade task has nothing to cancel.
        if standing.delegate != Status::Passed
            && let Some(Delegate::Repair(_)) = facts.delegate
            && !standing.delegate_running
        {
            return Stepped { took: None, change, decision: Decision::CancelRepair { head } };
        }
    }
    if !requested {
        return repair_due(change, why, facts, now, limits);
    }
    if why.ci()
        && standing.delegate == Status::Passed
        && let Some(head) = facts.branch
        && Some(head) == facts.repair_head.or(change.last_head)
        && facts.ci.head == head
        && facts.ci.status == Status::Failed
    {
        let mut next = change;
        next.last_head = Some(head);
        if facts.rerun {
            next.state = State::Checking { head };
            next.clock = None;
            return rerun(next, head, now);
        }
        return red_stands(next, why, facts, now);
    }
    delegated(change, requested, Delegate::Repair(why), facts, standing, now, limits)
}

fn green_head(facts: &Facts) -> Option<[u8; 32]> {
    let head = facts.branch?;
    if facts.ci.head == head && facts.ci.status == Status::Passed { Some(head) } else { None }
}

fn repair_ended(change: Change, head: [u8; 32], now: Wall) -> Stepped {
    let mut stepped = take_pushed_head(change, head, now);
    stepped.decision = Decision::RepairEnded { head };
    stepped
}

fn ci_state(state: &State) -> bool {
    match state {
        State::Repairing { why, .. } => why.ci(),
        State::Checking { .. } | State::CancellingGates { .. } => true,
        State::Taking { .. }
        | State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Judging { .. }
        | State::Gating { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. }
        | State::NoChange { .. }
        | State::Held { .. }
        | State::Ended => false,
    }
}

fn waiting_producer(
    mut change: Change,
    base: [u8; 32],
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if standing.delegate_live {
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    let Some(head) = facts.branch else { return waiting(change, facts, now, limits) };
    change.last_head = Some(head);
    let released = change.released_resolution;
    if change.way == Way::Turn {
        change.released_resolution = false;
        // The pull targets the goal branch, not the preparation's main tip.
        // Retry that preparation at the final pushed head to judge its conflict.
        let ready_since = change.ready_since.unwrap_or(now);
        change.state = State::Preparing { from: head, main: base, ready_since };
        change.since = now;
        return Stepped { took: None, change, decision: Decision::Effect(Effect::Prepare { head, main: base }) };
    }
    match facts.mergeable {
        Status::Passed => take_pushed_head(change, head, now),
        Status::Unknown | Status::Pending => settling(change, facts, now, limits),
        Status::Failed if released => resolving_due(change, facts.base_tip, facts, standing, now, limits),
        Status::Failed => match facts.delegate {
            // A released resolution asks directly at the base now read, with no
            // merge job (temper/docs/design/domain/forge/change/repairs.md, A release).
            Some(Delegate::Resolve { .. } | Delegate::Merge { .. }) => {
                resolving_due(change, facts.base_tip, facts, standing, now, limits)
            }
            Some(Delegate::Produce | Delegate::Close | Delegate::Repair(_) | Delegate::Gate { .. }) | None => {
                conflict_due(change, base, Some(head), facts, standing, now, limits)
            }
        },
    }
}

fn resolving(
    mut change: Change,
    requested: bool,
    base: [u8; 32],
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if requested && standing.delegate == Status::Passed {
        match facts.delegate {
            Some(Delegate::Merge { from, .. }) if facts.branch == from => {
                change.since = now;
                return Stepped { took: None, change, decision: Decision::Delegate(Delegate::Resolve { base }) };
            }
            Some(
                Delegate::Produce
                | Delegate::Close
                | Delegate::Repair(_)
                | Delegate::Resolve { .. }
                | Delegate::Gate { .. }
                | Delegate::Merge { .. },
            )
            | None => {}
        }
    }
    delegated(change, requested, Delegate::Resolve { base }, facts, standing, now, limits)
}

fn delegated(
    mut change: Change,
    requested: bool,
    delegate: Delegate,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if !requested {
        change.state = match delegate {
            Delegate::Repair(why) => State::Repairing { requested: true, why },
            Delegate::Resolve { base } | Delegate::Merge { base, .. } => State::Resolving { requested: true, base },
            Delegate::Produce | Delegate::Close | Delegate::Gate { .. } => unreachable!("delegated phase has one kind"),
        };
        return Stepped { took: None, change, decision: Decision::Delegate(delegate) };
    }
    match standing.delegate {
        Status::Passed => {
            let Some(head) = facts.branch else { return waiting_on_agent(change, facts, standing, now, limits) };
            if !facts.regate
                && Some(head) == facts.repair_head.or(change.last_head)
                && let Delegate::Repair(why @ (Repair::Landing { .. } | Repair::LandingAt { .. })) = delegate
            {
                change.last_head = Some(head);
                return red_stands(change, why, facts, now);
            }
            take_pushed_head(change, head, now)
        }
        Status::Failed => hold(change, Cause::Failed, now),
        Status::Unknown | Status::Pending => waiting_on_agent(change, facts, standing, now, limits),
    }
}

fn take_pushed_head(mut change: Change, head: [u8; 32], now: Wall) -> Stepped {
    change.released_resolution = false;
    change.clean = Box::new([]);
    change.gate_failures = Box::new([]);
    change.clock = None;
    change.owns_turn = false;
    change.state = pushed(&change, head);
    change.last_head = Some(head);
    change.since = now;
    Stepped { took: None, change, decision: Decision::None }
}

/// Its turn's contained head lands at the tip the fresh ancestry read proved (change/README.md, 2).
fn contained_landing(change: &Change, facts: &Facts, standing: &Heard, now: Wall) -> Option<Stepped> {
    contained_landing_at(change, &change.state, facts, standing, now)
}

fn contained_landing_at(change: &Change, was: &State, facts: &Facts, standing: &Heard, now: Wall) -> Option<Stepped> {
    let (State::First { head, .. } | State::Landing { head }) = was else { return None };
    if (change.way == Way::Queue || facts.landing.is_none())
        && facts.branch == Some(*head)
        && facts.drift.is_none()
        && !facts.writer_taken
        && facts.contains_head == Status::Passed
    {
        return Some(record_landed(change, facts.base_tip, facts, standing, now));
    }
    None
}

fn landing(change: Change, head: [u8; 32], facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    if let Some(stepped) = contained_landing(&change, facts, standing, now) {
        return stepped;
    }
    match standing.effect {
        EffectResult::Conflict => conflict_due(change, facts.base_tip, Some(head), facts, standing, now, limits),
        EffectResult::Failed => hold(change, Cause::Failed, now),
        EffectResult::None | EffectResult::Stale | EffectResult::Pending | EffectResult::Made => {
            waiting(change, facts, now, limits)
        }
    }
}

/// Record a failed review without inventing a verdict (domain/forge/change/gates.md, 1).
/// The top consumes the child's result once before calling this transition.
#[must_use]
pub fn review_failed(change: &Change, number: u64, head: [u8; 32], now: Wall, limits: &Limits) -> Change {
    let mut next = change.clone();
    let (current, asked) = match &change.state {
        State::Gating { head, asked } => (*head, asked),
        State::Judging { target, asked, .. } => (target.prefix, asked),
        State::Taking { .. }
        | State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
        | State::Ready { .. }
        | State::Preparing { .. }
        | State::CancellingGates { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Updating { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. }
        | State::Landing { .. }
        | State::Landed { .. }
        | State::NoChange { .. }
        | State::Ended
        | State::Held { .. }
        | State::Red { .. }
        | State::Cleared { .. }
        | State::Uncleared { .. } => return next,
    };
    if current != head || !asked.contains(&number) {
        return next;
    }
    next.owns_turn = false;
    let mut failures = List::with_capacity(limits.gates);
    let mut count = 1;
    for failure in &change.gate_failures {
        if failure.number == number {
            count = failure.count.saturating_add(1).min(2);
        } else {
            failures.push(*failure).expect("admitted gate failures");
        }
    }
    failures.push(GateFailure { number, count }).expect("one count per admitted gate");
    next.gate_failures = failures.into_boxed();
    let mut remaining = List::with_capacity(limits.gates);
    for prior in asked {
        if *prior != number {
            remaining.push(*prior).expect("admitted gate requests");
        }
    }
    if let State::Judging { head, ready_since, target, .. } = &change.state {
        next.state = State::Judging {
            head: *head,
            ready_since: *ready_since,
            target: target.clone(),
            asked: remaining.into_boxed(),
        };
    } else {
        next.state = State::Gating { head, asked: remaining.into_boxed() };
    }
    next.since = now;
    next
}

/// The hold's reason, shared by the requester and change panel.
#[must_use]
pub fn authority_words(why: AuthorityHold) -> Box<[u8]> {
    authority_recovery(why, why.recovery.hold, &authority_account(why))
}

#[must_use]
pub fn authority_account(why: AuthorityHold) -> Box<[u8]> {
    let mut words = List::with_capacity(512);
    match why.lack {
        AuthorityLack::Tasks => {
            put(&mut words, b"its ");
            put(&mut words, skein_lib::Decimal::of(why.had).as_bytes());
            put(&mut words, b" tasks are made");
        }
        AuthorityLack::Depth => put(&mut words, b"its depth is used"),
        AuthorityLack::Spend => {
            put(&mut words, b"its spend is below a run's reserve (it has ");
            put(&mut words, skein_lib::Decimal::of(why.had).as_bytes());
            put(&mut words, b" left)");
        }
    }
    if why.released {
        put(&mut words, b"; released without more ");
        put(
            &mut words,
            match why.lack {
                AuthorityLack::Tasks => b"tasks",
                AuthorityLack::Depth => b"depth",
                AuthorityLack::Spend => b"spend",
            },
        );
    }
    put(&mut words, b"; cancel: task cancel ");
    put(&mut words, skein_lib::Decimal::of(why.recovery.task).as_bytes());
    put(&mut words, b" REASON");
    if why.lack == AuthorityLack::Tasks {
        task_amend_words(&mut words, why);
    } else {
        put(&mut words, b"; or ask afresh; bare release holds again");
    }
    words.into_boxed()
}

#[must_use]
pub fn authority_recovery(why: AuthorityHold, revision: u64, account: &[u8]) -> Box<[u8]> {
    if why.lack != AuthorityLack::Tasks {
        return copy_of(account);
    }
    let mut words = List::with_capacity(512);
    put(&mut words, account);
    put(&mut words, b"; then release it: task release ");
    put(&mut words, skein_lib::Decimal::of(why.recovery.task).as_bytes());
    put(&mut words, b" --hold ");
    put(&mut words, skein_lib::Decimal::of(revision).as_bytes());
    put(&mut words, b"; or ask afresh; bare release holds again");
    words.into_boxed()
}

fn task_amend_words(words: &mut List<u8>, why: AuthorityHold) {
    put(words, b"; grant more tasks: task amend ");
    put(words, skein_lib::Decimal::of(why.recovery.task).as_bytes());
    put(words, b" --rev ");
    put(words, skein_lib::Decimal::of(why.recovery.spec).as_bytes());
    put(words, b" --tasks ");
    put(words, skein_lib::Decimal::of(why.recovery.tasks.saturating_add(2)).as_bytes());
    put(words, b" REASON");
}

fn put(words: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        words.push(*byte).expect("words fit their fixed bound");
    }
}

fn missing(standing: &Heard, delegate: Delegate) -> Option<AuthorityHold> {
    match delegate {
        Delegate::Gate { number: crate::FOLLOW_CHECK_GATE, .. } => return None,
        Delegate::Gate { .. }
        | Delegate::Produce
        | Delegate::Close
        | Delegate::Repair(_)
        | Delegate::Resolve { .. }
        | Delegate::Merge { .. } => {}
    }

    if let Some(why) = standing.lacking {
        return Some(why);
    }
    let spend = standing.spend?;
    let reserve = match delegate {
        Delegate::Gate { .. } => spend.review,
        Delegate::Produce
        | Delegate::Close
        | Delegate::Repair(_)
        | Delegate::Resolve { .. }
        | Delegate::Merge { .. } => spend.coding,
    };
    (spend.left < reserve).then_some(AuthorityHold {
        recovery: spend.recovery,
        lack: AuthorityLack::Spend,
        had: spend.left,
        released: false,
    })
}
// A turn change judges every pushed head without reading CI (domain/landing.md, 3).
fn pushed(change: &Change, head: [u8; 32]) -> State {
    match change.way {
        Way::Queue => State::Checking { head },
        Way::Turn => State::Gating { head, asked: Box::new([]) },
    }
}

/// An ask the change cannot answer at its ready head (domain/landing-members.md, 3.4).
fn dropped_ask(head: [u8; 32], facts: &Facts) -> Option<Took> {
    let handed = facts.ask.as_ref()?;
    let unanswerable = match &handed.ask {
        Ask::Prepare { head: expected, .. } => *expected != head,
        Ask::Judge(target) => facts.landing != Some(target.landing),
        Ask::Repair { .. } | Ask::Resolve { .. } | Ask::Landed { .. } => false,
    };
    if unanswerable { Some(Took::Dropped(handed.number)) } else { None }
}

fn ready(
    mut change: Change,
    head: [u8; 32],
    ready_since: Wall,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if standing.delegate_live {
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    if facts.branch.is_none() {
        return waiting(change, facts, now, limits);
    }
    let dropped = dropped_ask(head, facts);
    let ask = if dropped.is_some() { None } else { facts.ask.as_ref() };
    // A ready change uses its existing gate-asking path after an amendment; a
    // landing's judge asks at its target instead (gates.md, 1, Valid at a spec revision).
    let judging = match ask {
        Some(Handed { ask: Ask::Judge(_), .. }) => true,
        Some(Handed {
            ask: Ask::Prepare { .. } | Ask::Repair { .. } | Ask::Resolve { .. } | Ask::Landed { .. },
            ..
        })
        | None => false,
    };
    if !judging && stale_reviews(&change, facts) {
        change.state = State::Gating { head, asked: Box::new([]) };
        change.owns_turn = false;
        return Stepped { took: None, change, decision: Decision::None };
    }
    let Some(handed) = ask else {
        return Stepped { took: dropped, change, decision: Decision::Wait { until: None } };
    };
    match &handed.ask {
        Ask::Judge(target) => {
            change.state = State::Judging { head, ready_since, target: target.clone(), asked: Box::new([]) };
            change.since = now;
            Stepped { took: Some(Took::Taken(handed.number)), change, decision: Decision::None }
        }
        Ask::Prepare { main, .. } => {
            let mut stepped = prepare_due(change, head, *main, ready_since, now, limits);
            if let Decision::Effect(Effect::Prepare { .. }) = stepped.decision {
                stepped.took = Some(Took::Taken(handed.number));
            }
            stepped
        }
        Ask::Repair { gate, candidate, below } => handed_repair(
            change,
            Repair::LandingAt { gate: *gate, candidate: *candidate, below: *below },
            handed.number,
            facts,
            now,
            limits,
        ),
        Ask::Resolve { base } => {
            take_delegate(conflict_due(change, *base, Some(head), facts, standing, now, limits), handed.number)
        }
        Ask::Landed { .. } => unreachable!("landed ask returned before the phase"),
    }
}

fn red_stands(mut change: Change, why: Repair, facts: &Facts, now: Wall) -> Stepped {
    change.state = State::Repairing { requested: false, why };
    let judged = match change.last_head {
        Some(head) => Some(Judged { head, spec: facts.revision }),
        None => None,
    };
    let mut stepped = hold(change, Cause::RedStands, now);
    if let State::Held { hold, .. } = &mut stepped.change.state {
        hold.judged = judged;
    }
    stepped
}

fn handed_repair(change: Change, why: Repair, number: u32, facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    match facts.recheck {
        Recheck::Taken => {
            let mut stepped = red_stands(change, why, facts, now);
            stepped.took = Some(Took::Taken(number));
            stepped
        }
        Recheck::Owed | Recheck::None => take_delegate(repair_due(change, why, facts, now, limits), number),
    }
}

fn take_delegate(mut stepped: Stepped, number: u32) -> Stepped {
    if let Decision::Delegate(Delegate::Repair(_) | Delegate::Resolve { .. } | Delegate::Merge { .. }) =
        stepped.decision
    {
        stepped.took = Some(Took::Taken(number));
    }
    stepped
}

fn prepare_due(
    mut change: Change,
    head: [u8; 32],
    main: [u8; 32],
    ready_since: Wall,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if change.updates >= limits.updates {
        return hold(change, Cause::Updates, now);
    }
    change.updates = change.updates.checked_add(1).expect("update count below limit");
    send_update(change, head, main, ready_since, now, true)
}

#[expect(clippy::too_many_arguments, reason = "the pending prepare keeps both heads and the ready time")]
fn preparing(
    change: Change,
    from: [u8; 32],
    main: [u8; 32],
    ready_since: Wall,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    match standing.effect {
        EffectResult::Stale => stale_update(change, facts, now, limits),
        EffectResult::Conflict => conflict_due(change, main, Some(from), facts, standing, now, limits),
        EffectResult::Failed => hold(change, Cause::Failed, now),
        EffectResult::Made => prepared(change, from, main, ready_since, facts, now, limits),
        EffectResult::None => {
            let Some(head) = facts.branch else { return waiting(change, facts, now, limits) };
            match facts.contains_base {
                Status::Unknown | Status::Pending => waiting(change, facts, now, limits),
                Status::Failed => {
                    if head == from {
                        prepare_due(change, from, main, ready_since, now, limits)
                    } else {
                        take_pushed_head(change, head, now)
                    }
                }
                Status::Passed => prepared(change, from, main, ready_since, facts, now, limits),
            }
        }
        EffectResult::Pending => waiting(change, facts, now, limits),
    }
}

fn prepared(
    mut change: Change,
    from: [u8; 32],
    main: [u8; 32],
    ready_since: Wall,
    facts: &Facts,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    let Some(head) = facts.branch else { return waiting(change, facts, now, limits) };
    if facts.contains_base != Status::Passed || facts.base_tip != main {
        return waiting(change, facts, now, limits);
    }
    if head != from {
        if change.clean.len() >= skein_lib::usize_of(limits.clean_heads) {
            return hold(change, Cause::Updates, now);
        }
        let mut clean = List::with_capacity(limits.clean_heads);
        for predecessor in &change.clean {
            clean.push(*predecessor).expect("clean history admitted");
        }
        clean.push(from).expect("clean history has room");
        change.clean = clean.into_boxed();
    }
    change.last_head = Some(head);
    change.ready_since = Some(ready_since);
    change.since = now;
    change.state = State::Gating { head, asked: Box::new([]) };
    // Clean verdicts carry; exact verdicts are sought at the candidate (domain/landing.md, 3).
    gating(change, head, &[], facts, now, limits)
}

/// A clean review given at the member's head carries to its conflict-free prefix.
#[must_use]
pub fn prefix_verdict(
    change: &Change,
    gate: Gate,
    reports: &[crate::GateReport],
    head: [u8; 32],
    prefix: [u8; 32],
    revision: u64,
) -> Status {
    let mut result = Status::Unknown;
    for report in reports {
        let clean = gate.kind != GateKind::Check
            && gate.freshness == Freshness::Clean
            && (report.head == head || change.clean.contains(&report.head));
        if report.number == gate.number && report.at_revision(gate.kind, revision) && (report.head == prefix || clean) {
            match report.status {
                Status::Failed => return Status::Failed,
                Status::Pending => result = Status::Pending,
                Status::Passed if result != Status::Pending => result = Status::Passed,
                Status::Unknown | Status::Passed => {}
            }
        }
    }
    result
}

fn judging(mut change: Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    let State::Judging { head, ready_since, target, asked } = change.state.clone() else { unreachable!("judging") };
    if facts.branch.is_none() {
        return waiting(change, facts, now, limits);
    }
    if let Some(Handed { number, ask: Ask::Repair { gate, candidate, below }, .. }) = facts.ask {
        if facts.live_gate.is_some() {
            return Stepped { took: None, change, decision: Decision::CancelGates { head: target.prefix } };
        }
        return handed_repair(change, Repair::LandingAt { gate, candidate, below }, number, facts, now, limits);
    }
    if let Some(Handed { number, ask: Ask::Resolve { base }, .. }) = facts.ask {
        if standing.delegate_live {
            return waiting(change, facts, now, limits);
        }
        return take_delegate(conflict_due(change, base, Some(head), facts, standing, now, limits), number);
    }
    for failure in &change.gate_failures {
        if failure.count >= 2 {
            return hold(change.clone(), Cause::Failed, now);
        }
    }
    let mut waiting_gate = false;
    for gate in &change.gates.clone() {
        if !gate.blocking && !stale_review(*gate, &facts.gates, facts.revision) {
            continue;
        }
        match prefix_verdict(&change, *gate, &facts.gates, head, target.prefix, facts.revision) {
            Status::Failed if gate.blocking => {
                return repair_due(change, Repair::Gate(gate.number), facts, now, limits);
            }
            Status::Unknown
                if facts.landing == Some(target.landing)
                    && (!asked.contains(&gate.number) || stale_review(*gate, &facts.gates, facts.revision))
                    && facts.live_gate.is_none() =>
            {
                let mut collected = List::with_capacity(limits.gates);
                for number in &asked {
                    collected.push(*number).expect("bounded gates");
                }
                if !asked.contains(&gate.number) {
                    collected.push(gate.number).expect("bounded gates");
                }
                change.state =
                    State::Judging { head, ready_since, target: target.clone(), asked: collected.into_boxed() };
                change.since = now;
                return Stepped {
                    took: None,
                    change,
                    decision: Decision::Delegate(Delegate::Gate { number: gate.number, head: target.prefix }),
                };
            }
            Status::Unknown | Status::Pending if gate.blocking => waiting_gate = true,
            Status::Unknown | Status::Pending | Status::Failed | Status::Passed => {}
        }
    }
    if facts.landing != Some(target.landing) {
        change.state = State::Ready { head, ready_since };
        return Stepped { took: None, change, decision: Decision::Wait { until: None } };
    }
    if waiting_gate {
        return waiting(change, facts, now, limits);
    }
    change.state = State::Ready { head, ready_since };
    Stepped { took: None, change, decision: Decision::Wait { until: None } }
}
