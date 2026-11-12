//! One level-triggered change step (domain/forge/change/README.md).
//! Updates with no result or outstanding write decide again (temper/docs/design/domain/forge/README.md, 6).
//!
//! The top keeps the returned state and commits each decision before it sends
//! a delegate or effect. The top reports Pending while a write is outstanding.
//! This policy never reads the forge or commits an outbox entry itself.
use alloc::boxed::Box;
use skein_lib::{List, Wall, bytes::copy_of};

use crate::{
    Ask, AuthorityHold, AuthorityLack, Change, Decision, Delegate, Early, Effect, EffectResult, Facts, Freshness, Gate,
    GateFailure, GateKind, Heard, Hold, Limits, PendingKind, Pull, Repair, StartedFrom, State, Status, Stepped,
    Verification, Way,
};

/// Decide one change from current forge and task facts, independent of its wake.
#[must_use]
pub fn step(change: &Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    let stepped = advance(change, facts, standing, now, limits);
    if let Decision::Delegate(delegate) = stepped.decision
        && let Some(why) = missing(standing, delegate)
    {
        // Keep the unrequested phase and its counters: no delegate was made.
        return hold(change.clone(), Hold::Authority(why));
    }
    stepped
}

#[expect(clippy::too_many_lines, reason = "the ordered guards include requester-branch moves before phase dispatch")]
fn advance(change: &Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    let mut next = change.clone();
    if facts.kept {
        next.owns_turn = false;
    }
    if change.gates.len() > skein_lib::usize_of(limits.gates)
        || change.clean.len() > skein_lib::usize_of(limits.clean_heads)
        || change.gate_failures.len() > skein_lib::usize_of(limits.gates)
    {
        return hold(next, Hold::Failed);
    }
    if let Some(head) = facts.branch
        && change.last_head != Some(head)
    {
        next.gate_failures = Box::new([]);
    }
    if let Some(stepped) = after_landing(&next, facts, standing, now) {
        return stepped;
    }
    match facts.pull {
        Pull::Merged { commit } => {
            next.state = State::Landed { merge: commit, since: now };
            next.owns_turn = false;
            next.released = None;
            return verifying(next, commit, now, facts, standing.cancelled, now);
        }
        Pull::Missing | Pull::Open { .. } | Pull::Closed => {}
    }
    if standing.cancelled {
        if let State::Held { why, .. } = &next.state {
            next.ended_held = Some(*why);
        }
        next.state = State::Ended;
        next.released = None;
        next.owns_turn = false;
        return Stepped { change: next, decision: Decision::Cancel };
    }
    if standing.released {
        next = released(&next, now);
    }
    if let Some(why) = next.released.take() {
        return released_step(next, why, facts, standing);
    }
    if facts.contains_head == Status::Passed
        && !standing.delegate_live
        && !facts.writer_taken
        && facts.drift.is_none()
        && head_may_end(&next.state)
        && let Some(head) = facts.branch
    {
        next.state = State::NoChange { words: contained_words(b"its head ", head) };
        next.owns_turn = false;
        return Stepped { change: next, decision: Decision::NoChange };
    }
    if let State::Held { was, why } = &change.state {
        if held_on_ci(&change.state)
            && green_head(facts).is_some()
            && !standing.delegate_running
            && facts.drift.is_none()
            && !facts.writer_taken
            && facts.pull != Pull::Closed
        {
            next.state = released_state(was, *why, facts.pull);
            next.since = now;
            next.owns_turn = false;
            return Stepped { change: next, decision: Decision::EndHold };
        }
        return Stepped { change: next, decision: Decision::None };
    }
    // A conflicting preparation cancels its producer before waiting for that
    // producer's writer slot (domain/forge/change/questions.md, 1).
    let start_conflicted = match next.early {
        Early::None => false,
        Early::Started(_) | Early::Spent(_) => standing.start_conflict.is_some(),
    };
    if facts.writer_taken && !start_conflicted {
        return waiting_on_agent(next, facts, standing, now, limits);
    }
    if let Some(why) = facts.drift {
        return hold(next, why);
    }
    // The connector reports a changed target branch through facts.drift.
    // A pull's base commit can lag the landing tip, including our own merge.
    if facts.pull == Pull::Closed && closed_pull_holds(&change.state) {
        return hold(next, Hold::PullClosed);
    }
    if facts.requester_branch
        && let Some(head) = facts.branch
    {
        let (prior_head, needs_opening) = match &change.state {
            State::Opening { .. } => (change.last_head, facts.pull == Pull::Missing),
            State::Checking { head }
            | State::Gating { head, .. }
            | State::Queued { head, .. }
            | State::First { head, .. }
            | State::Ready { head, .. } => (Some(*head), false),
            State::Producing { .. }
            | State::Recreating { .. }
            | State::Reopening { .. }
            | State::CancellingGates { .. }
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
            | State::Uncleared { .. } => (None, false),
        };
        if let Some(prior) = prior_head
            && prior != head
        {
            if needs_opening {
                next.clean = Box::new([]);
                next.owns_turn = false;
                next.since = now;
                next.state = State::Opening { requested: false };
                return opening(next, false, facts, standing, now, limits);
            }
            return take_pushed_head(next, head, now);
        }
    }
    advance_phase(change, next, facts, standing, now, limits)
}

/// No containment decision during an update, preparation, merge or hold (domain/forge/change/README.md, 2).
fn head_may_end(state: &State) -> bool {
    match state {
        State::Held { .. }
        | State::Updating { .. }
        | State::Preparing { .. }
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
        | State::Gating { .. }
        | State::CancellingGates { .. }
        | State::Queued { .. }
        | State::First { .. }
        | State::Ready { .. }
        | State::WaitingProducer { .. }
        | State::Resolving { .. }
        | State::Repairing { .. } => true,
    }
}

fn after_landing(change: &Change, facts: &Facts, standing: &Heard, now: Wall) -> Option<Stepped> {
    let decision = match &change.state {
        State::Landed { merge, .. } | State::Red { merge, .. } if standing.cancelled => {
            let mut next = change.clone();
            next.state = State::Uncleared { merge: *merge };
            return Some(Stepped { change: next, decision: Decision::Finish { merge: *merge } });
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
                    Decision::Wait { until: None }
                }
            };
            return Some(Stepped { change: next, decision });
        }
        State::Cleared { merge } | State::Uncleared { merge } => Decision::Finish { merge: *merge },
        State::NoChange { .. } => Decision::NoChange,
        State::Ended => Decision::Cancel,
        State::Held { .. }
        | State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
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
    Some(Stepped { change: change.clone(), decision })
}

fn verifying(mut change: Change, merge: [u8; 32], since: Wall, facts: &Facts, cancelled: bool, now: Wall) -> Stepped {
    let until = Wall::from_nanos(since.as_nanos().saturating_add(facts.verification_bound.as_nanos()));
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
        Verification::Red => {
            change.state = State::Red { merge, since };
            Decision::Wait { until: None }
        }
        Verification::Verifying if now >= until => {
            change.state = State::Uncleared { merge };
            Decision::Finish { merge }
        }
        Verification::Verifying => Decision::Wait { until: Some(until) },
    };
    Stepped { change, decision }
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
        State::Producing { requested } => producing(next, *requested, facts, standing, now, limits),
        State::Opening { requested } => opening(next, *requested, facts, standing, now, limits),
        State::Recreating { head } => recreating(next, *head, facts, now, limits),
        State::Reopening { requested, retarget } => {
            reopening(next, *requested, *retarget, facts, standing, now, limits)
        }
        State::Checking { .. } | State::Gating { .. } | State::CancellingGates { .. }
            if standing.effect == EffectResult::Pending =>
        {
            waiting(next, now, limits)
        }
        State::Checking { .. } | State::Gating { .. } | State::CancellingGates { .. }
            if standing.effect == EffectResult::Failed =>
        {
            hold(next, Hold::Failed)
        }
        State::Checking { head } => checking(next, *head, facts, standing, now, limits),
        State::Gating { head, asked } => gating(next, *head, asked, facts, now, limits),
        State::Ready { head, ready_since } => ready(next, *head, *ready_since, facts, now, limits),
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

fn closed_pull_holds(state: &State) -> bool {
    match state {
        State::Reopening { .. } | State::Recreating { .. } => false,
        State::Producing { .. }
        | State::Opening { .. }
        | State::Checking { .. }
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
        | State::Ended
        | State::Held { .. } => true,
    }
}

/// A hold whose reason is CI failed at the head (temper/docs/design/domain/forge/change/repairs.md).
/// `was` `Repairing` with a CI reason: `Failed`, `Stalled`, `Repairs` or `Authority`.
/// `was` `Checking` or `CancellingGates`: `Repairs` or `Authority`.
/// Every other pair is false: `Gating`, drift, `Questions`, `Rejected`, `Resolutions`, `Updates`.
#[must_use]
pub fn held_on_ci(state: &State) -> bool {
    let State::Held { was, why } = state else { return false };
    match why {
        Hold::Repairs | Hold::Authority(_) => ci_state(was),
        Hold::Failed | Hold::Stalled => {
            if let State::Repairing { why, .. } = was.as_ref() {
                ci_reason(*why)
            } else {
                false
            }
        }
        Hold::BranchMoved
        | Hold::BranchMissing
        | Hold::PullClosed
        | Hold::Retargeted
        | Hold::Resolutions
        | Hold::Updates
        | Hold::Rejected
        | Hold::Questions => false,
    }
}

/// Lift a task's own change hold in the same decision as the task's release.
/// The persisted cause lets the next fresh snapshot perform drift recovery.
#[must_use]
pub fn released(change: &Change, now: Wall) -> Change {
    let mut next = change.clone();
    match &change.state {
        State::Held { was, why } => {
            next.state = *was.clone();
            next.released = Some(*why);
            next.gate_failures = Box::new([]);
            next.repairs = 0;
            next.resolutions = 0;
            next.updates = 0;
            next.since = now;
            next.owns_turn = false;
        }
        State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
        | State::Checking { .. }
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
        | State::Ended => {}
    }
    next
}

fn released_step(change: Change, why: Hold, facts: &Facts, standing: &Heard) -> Stepped {
    let delegate = if let State::Gating { head, .. } = change.state {
        Delegate::Gate { number: 0, head }
    } else {
        Delegate::Produce
    };
    if !(ci_state(&change.state) && green_head(facts).is_some())
        && let Hold::Authority(prior) = why
        && let Some(current) = missing(standing, delegate)
        && current.lack == prior.lack
    {
        return hold(change, Hold::Authority(AuthorityHold { released: true, ..current }));
    }
    let was = change.state.clone();
    release(change, &was, why, facts)
}

fn release(mut change: Change, was: &State, why: Hold, facts: &Facts) -> Stepped {
    match why {
        Hold::PullClosed => {
            change.state = State::Reopening { requested: true, retarget: false };
            Stepped { change, decision: Decision::Effect(Effect::Reopen) }
        }
        Hold::Retargeted => {
            change.state = State::Reopening { requested: true, retarget: true };
            Stepped { change, decision: Decision::Effect(Effect::Retarget) }
        }
        Hold::BranchMissing => match change.last_head {
            Some(head) => {
                change.state = State::Recreating { head };
                Stepped { change, decision: Decision::Effect(Effect::CreateBranch { head }) }
            }
            None => {
                change.state = if facts.requester_branch {
                    State::Opening { requested: false }
                } else {
                    State::Producing { requested: false }
                };
                Stepped { change, decision: Decision::None }
            }
        },
        Hold::BranchMoved => {
            change.clean = Box::new([]);
            change.state = match facts.branch {
                Some(head) => {
                    change.last_head = Some(head);
                    match facts.pull {
                        Pull::Open { .. } => pushed(&change, head),
                        Pull::Missing | Pull::Closed | Pull::Merged { .. } => State::Opening { requested: false },
                    }
                }
                None => State::Producing { requested: false },
            };
            Stepped { change, decision: Decision::None }
        }
        Hold::Stalled
        | Hold::Repairs
        | Hold::Questions
        | Hold::Resolutions
        | Hold::Updates
        | Hold::Failed
        | Hold::Rejected
        | Hold::Authority(_) => {
            change.state = released_state(was, why, facts.pull);
            Stepped { change, decision: Decision::None }
        }
    }
}

fn released_state(was: &State, why: Hold, pull: Pull) -> State {
    let open = match pull {
        Pull::Open { .. } => true,
        Pull::Missing | Pull::Closed | Pull::Merged { .. } => false,
    };
    match was {
        State::Checking { .. }
        | State::Gating { .. }
        | State::Ready { .. }
        | State::Queued { .. }
        | State::First { .. }
            if !open =>
        {
            State::Opening { requested: false }
        }
        State::Ready { head, .. }
        | State::Queued { head, .. }
        | State::First { head, .. }
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
        State::Resolving { base, .. } if why == Hold::Failed => State::WaitingProducer { base: *base },
        State::Producing { requested: true } if why == Hold::Failed => State::Producing { requested: false },
        State::Repairing { why: repair, .. } if why == Hold::Failed => {
            State::Repairing { requested: false, why: *repair }
        }
        State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
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

fn waiting(change: Change, now: Wall, limits: &Limits) -> Stepped {
    let until = Wall::from_nanos(change.since.as_nanos().saturating_add(limits.stall.as_nanos()));
    if now >= until {
        hold(change, Hold::Stalled)
    } else {
        Stepped { change, decision: Decision::Wait { until: Some(until) } }
    }
}

fn waiting_on_agent(mut change: Change, facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    if facts.asking {
        change.since = now;
        Stepped { change, decision: Decision::Wait { until: None } }
    } else if standing.delegate_live {
        change.since = now;
        let until = Wall::from_nanos(now.as_nanos().saturating_add(limits.stall.as_nanos()));
        Stepped { change, decision: Decision::Wait { until: Some(until) } }
    } else {
        waiting(change, now, limits)
    }
}

fn settling(change: Change, facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    let stall = Wall::from_nanos(change.since.as_nanos().saturating_add(limits.stall.as_nanos()));
    if let Some(end) = facts.settling
        && now < end.min(stall)
    {
        Stepped { change, decision: Decision::Wait { until: Some(end.min(stall)) } }
    } else {
        waiting(change, now, limits)
    }
}

fn hold(mut change: Change, why: Hold) -> Stepped {
    if change.state == State::Ended {
        return Stepped { change, decision: Decision::Cancel };
    }
    if change.owns_turn {
        change.owns_turn = false;
    }
    change.state = State::Held { was: Box::new(change.state.clone()), why };
    Stepped { change, decision: Decision::Hold(why) }
}

fn producing(
    mut change: Change,
    requested: bool,
    facts: &Facts,
    standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    if !requested && facts.follows && !facts.pending.is_empty() {
        return Stepped { change, decision: Decision::Wait { until: None } };
    }
    match &change.early {
        Early::Started(heads) => {
            if standing.start_conflict.is_some() {
                change.early = Early::Spent(heads.clone());
                change.state = State::Producing { requested: false };
                return Stepped { change, decision: Decision::CancelProducer };
            }
            if requested && standing.delegate == Status::Failed {
                change.early = Early::Spent(heads.clone());
                change.state = State::Producing { requested: false };
                return Stepped { change, decision: Decision::Wait { until: None } };
            }
        }
        Early::Spent(_) => {
            if !requested {
                return request_producer(change, facts, standing, now);
            }
            match standing.delegate {
                Status::Failed => return hold(change, Hold::Failed),
                Status::Unknown | Status::Pending => return waiting_on_agent(change, facts, standing, now, limits),
                Status::Passed => {}
            }
        }
        Early::None => {}
    }
    // A failed producer holds whatever it delivered (domain/forge/change/repairs.md).
    if requested && standing.delegate == Status::Failed {
        return hold(change, Hold::Failed);
    }
    match facts.branch {
        Some(head) if facts.titled && (!facts.close || standing.delegate == Status::Passed) => {
            change.state = State::Opening { requested: false };
            change.last_head = Some(head);
            change.since = now;
            Stepped { change, decision: Decision::None }
        }
        Some(_) if !requested => request_producer(change, facts, standing, now),
        Some(_) => waiting_on_agent(change, facts, standing, now, limits),
        None if !requested => {
            if facts.close {
                change.state = State::Producing { requested: true };
                change.since = now;
                return Stepped { change, decision: Decision::Delegate(Delegate::Close) };
            }
            if let Some(commit) = facts.catch_up {
                return catch_up_due(change, commit, facts, now, limits);
            }
            request_producer(change, facts, standing, now)
        }
        None => match standing.delegate {
            Status::Passed => match facts.delegate {
                Some(Delegate::Close) => {
                    change.state = State::NoChange { words: copy_of(b"Nothing to settle.") };
                    Stepped { change, decision: Decision::NoChange }
                }
                Some(Delegate::Merge { base, .. }) => {
                    let mut stepped = resolving_due(change, base, facts, standing, now, limits);
                    if stepped.decision == Decision::Delegate(Delegate::Resolve { base }) {
                        stepped.change.state = State::Producing { requested: true };
                    }
                    stepped
                }
                Some(Delegate::Produce | Delegate::Repair(_) | Delegate::Resolve { .. } | Delegate::Gate { .. })
                | None => waiting(change, now, limits),
            },
            Status::Failed => hold(change, Hold::Failed),
            Status::Unknown | Status::Pending => waiting_on_agent(change, facts, standing, now, limits),
        },
    }
}

// A catch-up's producer is the merge job (temper/docs/design/domain/forge/change/README.md, 2, temper/docs/design/domain/forge/change/repairs.md).
fn catch_up_due(mut change: Change, commit: [u8; 32], facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    match facts.contains_catch_up {
        Status::Passed => {
            change.state = State::NoChange { words: contained_words(b"commit ", commit) };
            Stepped { change, decision: Decision::NoChange }
        }
        Status::Failed => {
            change.state = State::Producing { requested: true };
            Stepped { change, decision: Decision::Delegate(Delegate::Merge { base: commit, from: None }) }
        }
        Status::Unknown | Status::Pending => waiting(change, now, limits),
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
        return Stepped { change, decision: Decision::Wait { until: None } };
    }
    change.state = State::Producing { requested: true };
    change.since = now;
    Stepped { change, decision: Decision::Delegate(Delegate::Produce) }
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
            PendingKind::Change { head: Some(head), kept: false } => {
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
        return hold(change, Hold::BranchMissing);
    };
    change.last_head = Some(head);
    match facts.pull {
        Pull::Open { head: pull_head, .. } => {
            if head != pull_head {
                return hold(change, Hold::BranchMoved);
            }
            change.state = pushed(&change, head);
            change.since = now;
            Stepped { change, decision: Decision::None }
        }
        Pull::Missing if !requested => {
            change.state = State::Opening { requested: true };
            change.since = now;
            Stepped { change, decision: Decision::Effect(Effect::Open) }
        }
        Pull::Missing => match standing.effect {
            EffectResult::Failed | EffectResult::Conflict => hold(change, Hold::Failed),
            EffectResult::None | EffectResult::Pending | EffectResult::Made => waiting(change, now, limits),
        },
        Pull::Closed | Pull::Merged { .. } => unreachable!("closed and merged pulls returned above"),
    }
}

fn recreating(mut change: Change, head: [u8; 32], facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    match facts.branch {
        Some(observed) if observed == head => {
            change.state = match facts.pull {
                Pull::Closed => State::Reopening { requested: false, retarget: false },
                Pull::Missing | Pull::Open { .. } | Pull::Merged { .. } => State::Opening { requested: false },
            };
            Stepped { change, decision: Decision::None }
        }
        Some(_) => hold(change, Hold::BranchMoved),
        None => waiting(change, now, limits),
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
        return hold(change, Hold::BranchMissing);
    };
    match facts.pull {
        Pull::Open { head: pull_head, .. } if pull_head == head => {
            change.state = pushed(&change, head);
            change.since = now;
            Stepped { change, decision: Decision::None }
        }
        Pull::Open { .. } | Pull::Closed | Pull::Missing if !requested => {
            change.state = State::Reopening { requested: true, retarget };
            let effect = if retarget { Effect::Retarget } else { Effect::Reopen };
            Stepped { change, decision: Decision::Effect(effect) }
        }
        Pull::Open { .. } | Pull::Closed | Pull::Missing => match standing.effect {
            EffectResult::Failed | EffectResult::Conflict => hold(change, Hold::Failed),
            EffectResult::None | EffectResult::Pending | EffectResult::Made => waiting(change, now, limits),
        },
        Pull::Merged { .. } => unreachable!("merged pull returned above"),
    }
}

fn rerun_due(facts: &Facts, limits: &Limits) -> bool {
    match facts.fault {
        crate::Fault::Runner { reruns } => reruns < limits.reruns,
        crate::Fault::Change => false,
    }
}

fn rerun(mut change: Change, head: [u8; 32], now: Wall) -> Stepped {
    change.since = now;
    Stepped { change, decision: Decision::Effect(Effect::Rerun { head }) }
}

fn checking(
    mut change: Change,
    head: [u8; 32],
    facts: &Facts,
    _standing: &Heard,
    now: Wall,
    limits: &Limits,
) -> Stepped {
    let Some(branch) = facts.branch else {
        return hold(change, Hold::BranchMissing);
    };
    if branch != head {
        return hold(change, Hold::BranchMoved);
    }
    change.last_head = Some(head);
    if facts.ci.head != head {
        return waiting(change, now, limits);
    }
    match facts.ci.status {
        Status::Passed => {
            change.state = State::Gating { head, asked: Box::new([]) };
            change.since = now;
            Stepped { change, decision: Decision::None }
        }
        Status::Failed => {
            if rerun_due(facts, limits) {
                return rerun(change, head, now);
            }
            let why = if change.clean.is_empty() { Repair::Ci } else { Repair::Semantic };
            repair_due(change, why, facts, now, limits)
        }
        Status::Unknown | Status::Pending => {
            if behind_unchecked(&change, facts, limits) {
                let ready_since = change.ready_since.unwrap_or(now);
                return update_due(change, head, facts.base_tip, ready_since, now, limits);
            }
            if has_eager_gate(&change) {
                change.state = State::Gating { head, asked: Box::new([]) };
                Stepped { change, decision: Decision::None }
            } else {
                settling(change, facts, now, limits)
            }
        }
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

fn valid(change: &Change, gate: Gate, report: crate::GateReport, head: [u8; 32]) -> bool {
    if report.number != gate.number {
        return false;
    }
    if report.head == head {
        return true;
    }
    match gate.freshness {
        Freshness::Exact => false,
        Freshness::Clean => change.clean.contains(&report.head),
    }
}

fn verdict(change: &Change, gate: Gate, facts: &Facts, head: [u8; 32]) -> Status {
    let mut passed = false;
    let mut pending = false;
    let mut failed = false;
    for report in &facts.gates {
        if !valid(change, gate, *report, head) {
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
    if facts.branch != Some(head) {
        return hold(change, Hold::BranchMoved);
    }
    if change.way == Way::Queue && facts.ci.head == head && facts.ci.status == Status::Failed {
        if rerun_due(facts, limits) {
            return rerun(change, head, now);
        }
        if facts.live_gate == Some(head) {
            change.state = State::CancellingGates { head };
            change.owns_turn = false;
            return Stepped { change, decision: Decision::CancelGates { head } };
        }
        let why = if change.clean.is_empty() { Repair::Ci } else { Repair::Semantic };
        return repair_due(change, why, facts, now, limits);
    }
    for failure in &change.gate_failures {
        if failure.count >= 2 {
            for gate in &change.gates {
                if failure.number == gate.number {
                    return hold(change, Hold::Failed);
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
                GateKind::Person => return hold(change, Hold::Rejected),
                GateKind::Agent | GateKind::Check | GateKind::Follow => {
                    return repair_due(change, Repair::Gate(gate.number), facts, now, limits);
                }
            },
            Status::Unknown if !asked.contains(&gate.number) => {
                change.owns_turn = false;
                // The top owns one delegate slot: retain it until that gate ends.
                if facts.live_gate.is_some() {
                    return waiting(change, now, limits);
                }
                let mut collected = List::with_capacity(limits.gates);
                for number in asked {
                    collected.push(*number).expect("asked gates fit the limit");
                }
                collected.push(gate.number).expect("gate admission checked");
                change.state = State::Gating { head, asked: collected.into_boxed() };
                change.since = now;
                return Stepped { change, decision: Decision::Delegate(Delegate::Gate { number: gate.number, head }) };
            }
            Status::Unknown | Status::Pending if gate.blocking => waiting_gate = true,
            Status::Failed | Status::Passed | Status::Unknown | Status::Pending => {}
        }
    }
    if !ci_passed || waiting_gate {
        return waiting(change, now, limits);
    }
    if facts.kept {
        change.owns_turn = false;
        return Stepped { change, decision: Decision::Wait { until: None } };
    }
    let since = match change.ready_since {
        Some(since) => since,
        None => {
            change.ready_since = Some(now);
            now
        }
    };
    if change.way == Way::Turn {
        change.state = State::Ready { head, ready_since: since };
    } else if change.owns_turn {
        change.state = State::First { head, ready_since: since };
    } else {
        change.state = State::Queued { head, ready_since: since };
    }
    change.since = now;
    Stepped { change, decision: Decision::Ready }
}

fn cancelling_gates(mut change: Change, head: [u8; 32], facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    if facts.branch == Some(head)
        && facts.ci.head == head
        && facts.ci.status == Status::Failed
        && rerun_due(facts, limits)
    {
        return rerun(change, head, now);
    }
    if facts.branch != Some(head) {
        return hold(change, Hold::BranchMoved);
    }
    if facts.live_gate == Some(head) {
        return waiting(change, now, limits);
    }
    if let Some(head) = green_head(facts) {
        return take_pushed_head(change, head, now);
    }
    if facts.ci.head != head || facts.ci.status == Status::Pending || facts.ci.status == Status::Unknown {
        return waiting(change, now, limits);
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
    if facts.branch != Some(head) {
        return hold(change, Hold::BranchMoved);
    }
    if let Some(reviewed) = standing.review_request {
        return repair_due(change, Repair::Review { head: reviewed }, facts, now, limits);
    }
    if facts.kept
        || facts.ci.head != head
        || facts.ci.status != Status::Passed
        || !blocking_gates_pass(&change, facts, head)
    {
        change.state = State::Gating { head, asked: Box::new([]) };
        change.owns_turn = false;
        return Stepped { change, decision: Decision::None };
    }
    if facts.first {
        change.state = State::First { head, ready_since };
        change.since = now;
        change.owns_turn = true;
        Stepped { change, decision: Decision::None }
    } else {
        Stepped { change, decision: Decision::Wait { until: None } }
    }
}

fn blocking_gates_pass(change: &Change, facts: &Facts, head: [u8; 32]) -> bool {
    for gate in &change.gates {
        if gate.blocking && verdict(change, *gate, facts, head) != Status::Passed {
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
    if facts.branch != Some(head) {
        return hold(change, Hold::BranchMoved);
    }
    if let Some(reviewed) = standing.review_request {
        return repair_due(change, Repair::Review { head: reviewed }, facts, now, limits);
    }
    if facts.kept
        || facts.ci.head != head
        || facts.ci.status != Status::Passed
        || !blocking_gates_pass(&change, facts, head)
    {
        change.state = State::Gating { head, asked: Box::new([]) };
        change.owns_turn = false;
        return Stepped { change, decision: Decision::None };
    }
    if facts.base_ci.head == facts.base_tip && facts.base_ci.status == Status::Failed {
        return if facts.queue_repair_active {
            Stepped { change, decision: Decision::Wait { until: None } }
        } else {
            Stepped { change, decision: Decision::QueueRepair }
        };
    }
    match facts.contains_base {
        Status::Failed => return update_due(change, head, facts.base_tip, ready_since, now, limits),
        Status::Unknown | Status::Pending => return waiting(change, now, limits),
        Status::Passed => {}
    }
    match facts.mergeable {
        Status::Passed => {
            if !facts.may_merge {
                return Stepped { change, decision: Decision::Wait { until: None } };
            }
            change.state = State::Landing { head };
            change.since = now;
            Stepped { change, decision: Decision::Effect(Effect::Merge { head, base: facts.expected_base }) }
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
        EffectResult::Conflict => conflict_due(change, base, Some(from), facts, standing, now, limits),
        EffectResult::Failed => hold(change, Hold::Failed),
        EffectResult::Made => {
            let Some(head) = facts.branch else { return waiting(change, now, limits) };
            if head == from {
                return waiting(change, now, limits);
            }
            updated(change, from, head, now, limits)
        }
        EffectResult::None => {
            let Some(head) = facts.branch else { return waiting(change, now, limits) };
            match facts.contains_base {
                Status::Unknown | Status::Pending => waiting(change, now, limits),
                Status::Failed => {
                    if head == from {
                        update_due(change, from, facts.base_tip, ready_since, now, limits)
                    } else {
                        hold(change, Hold::BranchMoved)
                    }
                }
                Status::Passed => {
                    if head != from {
                        return updated(change, from, head, now, limits);
                    }
                    change.state = pushed(&change, head);
                    change.last_head = Some(head);
                    change.since = now;
                    Stepped { change, decision: Decision::None }
                }
            }
        }
        EffectResult::Pending => waiting(change, now, limits),
    }
}

fn updated(mut change: Change, from: [u8; 32], head: [u8; 32], now: Wall, limits: &Limits) -> Stepped {
    if change.clean.len() >= skein_lib::usize_of(limits.clean_heads) {
        return hold(change, Hold::Updates);
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
    Stepped { change, decision: Decision::None }
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
                None => return hold(change, Hold::BranchMissing),
            },
            Status::Unknown | Status::Pending => return waiting(change, now, limits),
            Status::Passed => {}
        }
    }
    if change.repairs >= limits.repairs {
        return hold(change, Hold::Repairs);
    }
    change.repairs = change.repairs.checked_add(1).expect("repair count below limit");
    match why {
        Repair::Semantic => {}
        Repair::Ci | Repair::Gate(_) | Repair::Landing { .. } | Repair::Review { .. } => change.clean = Box::new([]),
    }
    change.state = State::Repairing { requested: true, why };
    change.since = now;
    Stepped { change, decision: Decision::Delegate(Delegate::Repair(why)) }
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
        return hold(change, Hold::Updates);
    }
    change.updates = change.updates.checked_add(1).expect("update count below limit");
    change.state = State::Updating { from: head, base, ready_since };
    change.since = now;
    Stepped { change, decision: Decision::Effect(Effect::Update { head, base }) }
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
        return Stepped { change, decision: Decision::Wait { until: None } };
    }
    change.last_head = facts.branch;
    if change.resolutions >= limits.resolutions {
        return hold(change, Hold::Resolutions);
    }
    change.resolutions = change.resolutions.checked_add(1).expect("resolution count below limit");
    change.owns_turn = false;
    change.clean = Box::new([]);
    change.state = State::Resolving { requested: true, base };
    change.since = now;
    Stepped { change, decision: Decision::Delegate(Delegate::Resolve { base }) }
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
    if ci_reason(why)
        && let Some(head) = green_head(facts)
    {
        if !requested || standing.delegate == Status::Failed {
            return repair_ended(change, head, now);
        }
        // Passed stands as a delivery below. An unmade task has nothing to cancel.
        if standing.delegate != Status::Passed
            && let Some(Delegate::Repair(_)) = facts.delegate
            && !standing.delegate_running
        {
            return Stepped { change, decision: Decision::CancelRepair { head } };
        }
    }
    if !requested {
        return repair_due(change, why, facts, now, limits);
    }
    delegated(change, requested, Delegate::Repair(why), facts, standing, now, limits)
}

fn ci_reason(why: Repair) -> bool {
    match why {
        Repair::Ci | Repair::Semantic => true,
        Repair::Gate(_) | Repair::Landing { .. } | Repair::Review { .. } => false,
    }
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
        State::Repairing { why, .. } => ci_reason(*why),
        State::Checking { .. } | State::CancellingGates { .. } => true,
        State::Producing { .. }
        | State::Opening { .. }
        | State::Recreating { .. }
        | State::Reopening { .. }
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
        return Stepped { change, decision: Decision::Wait { until: None } };
    }
    let Some(head) = facts.branch else { return hold(change, Hold::BranchMissing) };
    change.last_head = Some(head);
    if change.way == Way::Turn {
        // The pull targets the goal branch, not the preparation's main tip.
        // Retry that preparation at the final pushed head to judge its conflict.
        let ready_since = change.ready_since.unwrap_or(now);
        change.state = State::Preparing { from: head, main: base, ready_since };
        change.since = now;
        return Stepped { change, decision: Decision::Effect(Effect::Prepare { head, main: base }) };
    }
    match facts.mergeable {
        Status::Passed => take_pushed_head(change, head, now),
        Status::Unknown | Status::Pending => settling(change, facts, now, limits),
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
                return Stepped { change, decision: Decision::Delegate(Delegate::Resolve { base }) };
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
        return Stepped { change, decision: Decision::Delegate(delegate) };
    }
    match standing.delegate {
        Status::Passed => {
            let Some(head) = facts.branch else { return waiting_on_agent(change, facts, standing, now, limits) };
            take_pushed_head(change, head, now)
        }
        Status::Failed => hold(change, Hold::Failed),
        Status::Unknown | Status::Pending => waiting_on_agent(change, facts, standing, now, limits),
    }
}

fn take_pushed_head(mut change: Change, head: [u8; 32], now: Wall) -> Stepped {
    change.clean = Box::new([]);
    change.owns_turn = false;
    change.state = pushed(&change, head);
    change.last_head = Some(head);
    change.since = now;
    Stepped { change, decision: Decision::None }
}

fn landing(change: Change, head: [u8; 32], facts: &Facts, standing: &Heard, now: Wall, limits: &Limits) -> Stepped {
    match standing.effect {
        EffectResult::Conflict => conflict_due(change, facts.base_tip, Some(head), facts, standing, now, limits),
        EffectResult::Failed => hold(change, Hold::Failed),
        EffectResult::None | EffectResult::Pending | EffectResult::Made => waiting(change, now, limits),
    }
}

/// Record a failed review without inventing a verdict (domain/forge/change/gates.md, 1).
/// The top consumes the child's result once before calling this transition.
#[must_use]
pub fn review_failed(change: &Change, number: u64, head: [u8; 32], now: Wall, limits: &Limits) -> Change {
    let mut next = change.clone();
    let State::Gating { head: current, asked } = &change.state else { return next };
    if *current != head || !asked.contains(&number) {
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
    next.state = State::Gating { head, asked: remaining.into_boxed() };
    next.since = now;
    next
}

/// The hold's reason, shared by the requester and change panel.
#[must_use]
pub fn authority_words(why: AuthorityHold) -> Box<[u8]> {
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
        task_recovery_words(&mut words, why);
    }
    put(&mut words, b"; or ask afresh; bare release holds again");
    words.into_boxed()
}

fn task_recovery_words(words: &mut List<u8>, why: AuthorityHold) {
    put(words, b"; grant more tasks: task amend ");
    put(words, skein_lib::Decimal::of(why.recovery.task).as_bytes());
    put(words, b" --rev ");
    put(words, skein_lib::Decimal::of(why.recovery.spec).as_bytes());
    put(words, b" --tasks ");
    put(words, skein_lib::Decimal::of(why.recovery.tasks.saturating_add(2)).as_bytes());
    put(words, b" REASON; then release it: task release ");
    put(words, skein_lib::Decimal::of(why.recovery.task).as_bytes());
    put(words, b" --hold ");
    put(words, skein_lib::Decimal::of(why.recovery.hold).as_bytes());
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

fn ready(change: Change, head: [u8; 32], ready_since: Wall, facts: &Facts, now: Wall, limits: &Limits) -> Stepped {
    if facts.branch != Some(head) {
        return hold(change, Hold::BranchMoved);
    }
    match facts.ask {
        None => Stepped { change, decision: Decision::Wait { until: None } },
        Some(Ask::Prepare { head: expected, main }) => {
            if head != expected {
                return hold(change, Hold::BranchMoved);
            }
            prepare_due(change, head, main, ready_since, now, limits)
        }
        Some(Ask::Repair { gate, candidate: _ }) => repair_due(change, Repair::Landing { gate }, facts, now, limits),
    }
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
        return hold(change, Hold::Updates);
    }
    change.updates = change.updates.checked_add(1).expect("update count below limit");
    change.state = State::Preparing { from: head, main, ready_since };
    change.since = now;
    Stepped { change, decision: Decision::Effect(Effect::Prepare { head, main }) }
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
        EffectResult::Conflict => resolving_due(change, main, facts, standing, now, limits),
        EffectResult::Failed => hold(change, Hold::Failed),
        EffectResult::Made => prepared(change, from, main, ready_since, facts, now, limits),
        EffectResult::None => {
            let Some(head) = facts.branch else { return waiting(change, now, limits) };
            match facts.contains_base {
                Status::Unknown | Status::Pending => waiting(change, now, limits),
                Status::Failed => {
                    if head == from {
                        prepare_due(change, from, main, ready_since, now, limits)
                    } else {
                        hold(change, Hold::BranchMoved)
                    }
                }
                Status::Passed => prepared(change, from, main, ready_since, facts, now, limits),
            }
        }
        EffectResult::Pending => waiting(change, now, limits),
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
    let Some(head) = facts.branch else { return waiting(change, now, limits) };
    if facts.contains_base != Status::Passed || facts.base_tip != main {
        return waiting(change, now, limits);
    }
    if head != from {
        if change.clean.len() >= skein_lib::usize_of(limits.clean_heads) {
            return hold(change, Hold::Updates);
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
