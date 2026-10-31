//! Shared completion admission, reservations and monotonic usage.
//! The gate retains no separate lifecycle; it accounts against its run and follows
//! every run transition. Contract: smith/docs/design/domain/run.md, section 9.

use core::mem;

use skein_lib::{Duration, Id, Queue, Time, Token};

use super::{Conversation, Phase, Run, State, follow, hard_stop, standing};
use crate::charter::{self, Mode};
use crate::facts::FactKind;
use crate::wind_down::Cause;
use crate::{CompletionPermit, Domain, Exhausted, Failure, Request, Spend, prompt};

/// Session own prices are monotonic activation totals. Stale names are inert;
/// subtree updates do not add global spend.
#[must_use]
pub fn completion_overflow(domain: &Domain, conversation: Token, price: u64, usage: Spend) -> Option<Failure> {
    let conversation =
        domain.conversations.get(Id::from_token(conversation)).expect("provider owner has a live run conversation");
    let run = domain.runs.get(conversation.run).expect("conversation retains its run");
    if run.spent.units.checked_add(price).is_none() {
        return Some(Failure::Budget {
            exhausted: Exhausted::Overflow(crate::Overflow::Spend),
            used: None,
            bound: None,
        });
    }
    let raw_total = Spend { units: 0, ..run.spent };
    let raw_increment = Spend { units: 0, ..usage };
    if raw_total.accumulate(raw_increment).is_none() {
        return Some(Failure::Budget {
            exhausted: Exhausted::Overflow(crate::Overflow::Usage),
            used: None,
            bound: None,
        });
    }
    None
}

/// Session own prices are monotonic activation totals. Stale names are inert;
/// subtree updates do not add global spend.
pub(crate) fn priced(domain: &mut Domain, conversation: Token, own: u64, subtree: u64, out: &mut Queue<Request>) {
    let id = Id::<Conversation>::from_token(conversation);
    let Some(conversation) = domain.conversations.get_mut(id) else {
        return;
    };
    match conversation.phase {
        Phase::Running { .. } | Phase::Closing => {}
        Phase::Pending | Phase::Opening | Phase::Unwanted | Phase::Closed => return,
    }
    let run_id = conversation.run;
    if own < conversation.spent.units || subtree < conversation.subtree_spent {
        return;
    }
    let delta = own.checked_sub(conversation.spent.units).expect("checked monotonic own total");
    let run = domain.runs.get_mut(run_id).expect("conversation retains run");
    let Some(total) = run.spent.accumulate(Spend { units: delta, ..Spend::ZERO }) else {
        let state = mem::replace(&mut run.state, State::Closed);
        run.state = hard_stop(
            state,
            &mut domain.conversations,
            Failure::Budget { exhausted: Exhausted::Overflow(crate::Overflow::Spend), used: None, bound: None },
            out,
        );
        follow(&mut domain.runs, &mut domain.alarms, &mut domain.facts, &mut domain.ready, run_id);
        return;
    };
    conversation.spent.units = own;
    conversation.subtree_spent = subtree;
    run.spent = total;
    if account_state(run, &mut domain.ready, run_id, domain.facts.now()) {
        domain.facts.about(run_id.token());
        domain.facts.push(FactKind::Budget {
            run: run_id.token(),
            standing: standing(run, domain.facts.now()),
            call: None,
        });
    }
    follow(&mut domain.runs, &mut domain.alarms, &mut domain.facts, &mut domain.ready, run_id);
}

pub(crate) fn used(domain: &mut Domain, conversation: Token, spend: Spend, out: &mut Queue<Request>) {
    let id = Id::<Conversation>::from_token(conversation);
    let Some(conversation) = domain.conversations.get_mut(id) else {
        return;
    };
    match conversation.phase {
        Phase::Running { .. } | Phase::Closing => {}
        Phase::Pending | Phase::Opening | Phase::Unwanted | Phase::Closed => return,
    }
    let run_id = conversation.run;
    let raw = Spend { units: 0, ..spend };
    let run = domain.runs.get_mut(run_id).expect("conversation retains run");
    let added = conversation.spent.accumulate(raw).zip(run.spent.accumulate(raw));
    let Some((conversation_total, run_total)) = added else {
        let state = mem::replace(&mut run.state, State::Closed);
        run.state = hard_stop(
            state,
            &mut domain.conversations,
            Failure::Budget { exhausted: Exhausted::Overflow(crate::Overflow::Usage), used: None, bound: None },
            out,
        );
        follow(&mut domain.runs, &mut domain.alarms, &mut domain.facts, &mut domain.ready, run_id);
        return;
    };
    conversation.spent = conversation_total;
    run.spent = run_total;
    account_state(run, &mut domain.ready, run_id, domain.facts.now());
    domain.facts.about(run_id.token());
    domain.facts.push(FactKind::Budget {
        run: run_id.token(),
        standing: standing(run, domain.facts.now()),
        call: None,
    });
    follow(&mut domain.runs, &mut domain.alarms, &mut domain.facts, &mut domain.ready, run_id);
}

pub(super) fn account_state(run: &mut Run, ready: &mut crate::wind_down::Ready, id: Id<Run>, now: Time) -> bool {
    match run.state {
        State::Working { .. } => {}
        State::Awaiting { .. }
        | State::Preparing { .. }
        | State::Stopping { .. }
        | State::Listening { .. }
        | State::Waiting { .. }
        | State::Winding { .. }
        | State::Closed => return false,
    }
    let exhausted = gate(run, now, None);
    match exhausted {
        Some(exhausted) => begin_wind_down(run, ready, id, Cause::Budget(exhausted)),
        None => false,
    }
}

fn ordinary_left(run: &Run, now: Time) -> crate::Budget {
    let reserved_spend =
        crate::budget::reserve_spend(run.charter.reserve, &run.charter.llm).expect("admission checked reserve spend");
    crate::budget::ordinary(
        &run.charter.budget,
        run.charter.reserve,
        reserved_spend,
        run.spent,
        run.deadline.saturating_since(now),
    )
}

pub(super) fn begin_wind_down(run: &mut Run, ready: &mut crate::wind_down::Ready, id: Id<Run>, cause: Cause) -> bool {
    match run.wind_down {
        Some(_) => false,
        None => {
            run.wind_down = Some(crate::wind_down::WindDown::new(run.children.len(), cause));
            match run.charter.mode {
                Mode::Open => ready.keep(id),
                Mode::Finite => {
                    if !run.children.is_empty() {
                        ready.keep(id);
                    }
                }
            }
            true
        }
    }
}

/// The completion's spend bound, supplied by the root (smith/docs/design/protocol/programs.md 5.7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Most {
    /// The direct backend's priced worst case, released at settlement.
    Bounded(u64),
    /// A program's unknown cost, charged in full at settlement.
    Unreserved,
}

impl Most {
    fn reserved(self) -> u64 {
        match self {
            Self::Bounded(amount) => amount,
            Self::Unreserved => 0,
        }
    }
}

/// Admit unsent work, starting the finishing phase before any provider effect.
#[must_use]
pub fn completion_permit(domain: &mut Domain, conversation: Token, now: Time, most: Most) -> CompletionPermit {
    let id = Id::<Conversation>::from_token(conversation);
    let Some(conversation) = domain.conversations.get(id) else {
        return CompletionPermit::Closing;
    };
    match conversation.phase {
        Phase::Running { .. } => {}
        Phase::Pending | Phase::Opening | Phase::Unwanted | Phase::Closing | Phase::Closed => {
            return CompletionPermit::Closing;
        }
    }
    let run_id = conversation.run;
    let child = conversation.asker.is_some();
    let run = domain.runs.get_mut(run_id).expect("conversation retains run");
    match run.state {
        State::Working { .. } => {}
        State::Listening { .. }
        | State::Waiting { .. }
        | State::Winding { .. }
        | State::Awaiting { .. }
        | State::Preparing { .. }
        | State::Stopping { .. }
        | State::Closed => return CompletionPermit::Closing,
    }
    let changed = match gate(run, now, Some(most)) {
        Some(exhausted) => begin_wind_down(run, &mut domain.ready, run_id, Cause::Budget(exhausted)),
        None => false,
    };
    if changed {
        domain.facts.begin(now);
        domain.facts.about(run_id.token());
        domain.facts.push(FactKind::Budget { run: run_id.token(), standing: standing(run, now), call: None });
    }
    let permit = working_permit(run, child, now);
    if child && permit == CompletionPermit::Closing {
        let exhausted = run.wind_down.as_ref().expect("closing child is winding down").cause().exhausted();
        domain.conversations.get_mut(id).expect("live child").budget_close = exhausted;
    }
    follow(&mut domain.runs, &mut domain.alarms, &mut domain.facts, &mut domain.ready, run_id);
    permit
}

fn gate(run: &Run, now: Time, completion: Option<Most>) -> Option<Exhausted> {
    let ordinary = ordinary_left(run, now);
    let held_turns = match completion {
        Some(_) => run.reserved_turns,
        None => 0,
    };
    let spend_exhausted = match completion {
        Some(Most::Bounded(amount)) => amount > ordinary.spend.saturating_sub(run.reserved),
        Some(Most::Unreserved) => ordinary.spend <= run.reserved,
        None => ordinary.spend == 0,
    };
    if ordinary.turns <= held_turns {
        Some(Exhausted::Turns)
    } else if spend_exhausted {
        Some(Exhausted::Spend)
    } else if ordinary.time == Duration::ZERO {
        Some(Exhausted::Time)
    } else {
        None
    }
}

fn final_turn_failure(run: &Run) -> Option<Exhausted> {
    match &run.wind_down {
        Some(crate::wind_down::WindDown::Finishing { used, cause, .. }) if *used >= run.charter.reserve.turns => {
            Some(if run.charter.reserve.turns == 0 && *cause == Cause::Budget(Exhausted::Spend) {
                Exhausted::Spend
            } else {
                Exhausted::Turns
            })
        }
        Some(crate::wind_down::WindDown::Finishing { .. } | crate::wind_down::WindDown::Settling { .. }) | None => None,
    }
}

fn working_permit(run: &mut Run, child: bool, now: Time) -> CompletionPermit {
    if child && run.wind_down.is_some() {
        return CompletionPermit::Closing;
    }
    if now >= run.deadline {
        return CompletionPermit::Denied(Exhausted::Time);
    }
    if child && ordinary_left(run, now).spend <= run.reserved {
        return CompletionPermit::Denied(Exhausted::Spend);
    }
    if let Some(exhausted) = run.charter.budget.exhausted(run.spent) {
        return CompletionPermit::Denied(exhausted);
    }
    if !child && completion_held(run) {
        return CompletionPermit::Held;
    }
    if let Some(reason) = final_turn_failure(run) {
        return CompletionPermit::Denied(reason);
    }
    match &mut run.wind_down {
        Some(crate::wind_down::WindDown::Settling { .. }) => CompletionPermit::Held,
        Some(crate::wind_down::WindDown::Finishing { used: _, noted, cause }) => {
            let note = if *noted {
                None
            } else {
                *noted = true;
                Some(prompt::wind_down_note(
                    run.charter.mode,
                    *cause,
                    charter::Reserve {
                        time: run.charter.reserve.time.min(run.deadline.saturating_since(now)),
                        ..run.charter.reserve
                    },
                ))
            };
            CompletionPermit::Final { note }
        }
        None => CompletionPermit::Allowed,
    }
}

pub(super) fn final_remaining(run: &Run, now: Time) -> Result<(), Exhausted> {
    if let Some(exhausted) = run.charter.budget.exhausted(run.spent) {
        return Err(exhausted);
    }
    if now >= run.deadline {
        return Err(Exhausted::Time);
    }
    match final_turn_failure(run) {
        Some(reason) => Err(reason),
        None => Ok(()),
    }
}

fn completion_held(run: &Run) -> bool {
    let pending = run.turns.checked_sub(run.acknowledged).expect("acknowledged prefix never exceeds turns");
    let next_bytes = match u64::from(pending).checked_add(1) {
        Some(count) => count.checked_mul(run.window.largest_turn),
        None => None,
    };
    pending >= run.window.turns
        || match next_bytes {
            Some(bytes) => bytes > run.window.bytes,
            None => true,
        }
}

/// Reserve one completion against the run's shared unspent ceiling before the
/// parent publishes it. Every live conversation can hold at most one.
pub fn reserve(domain: &mut Domain, conversation: Token, owner: Token, most: Most) -> Result<(), Exhausted> {
    let id = Id::<Conversation>::from_token(conversation);
    let conversation = domain.conversations.get_mut(id).expect("a provider request retains its conversation");
    assert!(conversation.reservation.is_none(), "a conversation sends one completion at a time");
    let run = domain.runs.get_mut(conversation.run).expect("conversation retains run");
    let committed = run.spent.units.checked_add(run.reserved).ok_or(Exhausted::Overflow(crate::Overflow::Spend))?;
    let total = committed.checked_add(most.reserved()).ok_or(Exhausted::Overflow(crate::Overflow::Spend))?;
    let final_turn = match &run.wind_down {
        Some(crate::wind_down::WindDown::Finishing { used, .. }) if conversation.asker.is_none() => {
            if *used >= run.charter.reserve.turns {
                return Err(Exhausted::Turns);
            }
            true
        }
        Some(_) => return Err(Exhausted::Turns),
        None => false,
    };
    if !final_turn {
        let turn_ceiling = run.charter.budget.turns.checked_sub(run.charter.reserve.turns).expect("admitted reserve");
        let next_turn = match run.spent.turns.checked_add(run.reserved_turns) {
            Some(turns) => turns.checked_add(1),
            None => None,
        }
        .ok_or(Exhausted::Overflow(crate::Overflow::Usage))?;
        if next_turn > turn_ceiling {
            return Err(Exhausted::Turns);
        }
    }
    let spend_ceiling = if final_turn {
        run.charter.budget.spend
    } else {
        let reserved = crate::budget::reserve_spend(run.charter.reserve, &run.charter.llm).expect("admitted reserve");
        run.charter.budget.spend.checked_sub(reserved).expect("admitted reserve")
    };
    let exhausted = match most {
        Most::Bounded(_) => total > spend_ceiling,
        Most::Unreserved => committed >= spend_ceiling,
    };
    if exhausted {
        return Err(Exhausted::Spend);
    }
    if final_turn {
        match &mut run.wind_down {
            Some(crate::wind_down::WindDown::Finishing { used, .. }) => {
                *used = used.checked_add(1).expect("bounded final completions");
            }
            Some(crate::wind_down::WindDown::Settling { .. }) | None => unreachable!("final turn admitted"),
        }
    }
    run.reserved = run.reserved.checked_add(most.reserved()).expect("checked with total above");
    run.reserved_turns = run.reserved_turns.checked_add(1).expect("one bounded reservation per conversation");
    conversation.reservation = Some((owner, most));
    Ok(())
}

/// Return unused credit after the provider's terminal. An overbound charge is
/// rejected by the caller before it can enter the session's spend record.
pub fn settle_reservation(domain: &mut Domain, conversation: Token, owner: Token, charge: u64) -> bool {
    let id = Id::<Conversation>::from_token(conversation);
    let conversation = domain.conversations.get_mut(id).expect("a provider terminal retains its conversation");
    let (reserved_owner, most) = conversation.reservation.take().expect("every sent completion was reserved");
    assert!(reserved_owner == owner, "provider terminal settles its own reservation");
    let run = domain.runs.get_mut(conversation.run).expect("conversation retains run");
    run.reserved = run.reserved.checked_sub(most.reserved()).expect("reservation belongs to this run");
    run.reserved_turns = run.reserved_turns.checked_sub(1).expect("reservation belongs to this run");
    match most {
        Most::Bounded(amount) => charge <= amount,
        Most::Unreserved => true,
    }
}
