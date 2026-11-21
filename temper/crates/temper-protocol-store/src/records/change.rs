use super::{FITS, w};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_change as d;
use temper_engine_domain_forge_vocabulary as top;
fn encode_hold(value: d::Hold) -> w::ChangeHold {
    crate::generated::store::change_hold_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_hold(value: w::ChangeHold) -> d::Hold {
    crate::generated::store::change_hold_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_below(below: Option<d::Below>) -> Option<w::LandingBelow> {
    let below = below?;
    let mut changes = List::with_capacity(4);
    for landed in below.changes.into_iter().flatten() {
        changes
            .push(crate::generated::store::landed_below_to_record(&landed, &super::Context::UNBOUNDED).expect(FITS))
            .expect(FITS);
    }
    Some(w::LandingBelow { green: below.green, changes, by_test: below.by_test, top: below.top })
}

pub(super) fn decode_below(below: Option<w::LandingBelow>) -> Option<d::Below> {
    let w::LandingBelow { green, changes, by_test, top } = below?;
    let mut below = d::Below { green, changes: [None; 4], by_test, top };
    for (slot, landed) in below.changes.iter_mut().zip(&changes) {
        *slot = Some(
            crate::generated::store::landed_below_to_domain(landed.clone(), &super::Context::UNBOUNDED).expect(FITS),
        );
    }
    Some(below)
}

fn encode_repair(value: d::Repair) -> w::Repair {
    crate::generated::store::repair_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_repair(value: w::Repair) -> d::Repair {
    crate::generated::store::repair_to_domain(value, &super::Context::UNBOUNDED).expect("infallible store translation")
}
fn encode_live(value: &d::State) -> w::ChangeLive {
    match value {
        d::State::NoChange { words } => w::ChangeLive::NoChange(w::NoChange { words: words.clone() }),
        d::State::Ended => w::ChangeLive::Ended,
        d::State::Taking { requested } => w::ChangeLive::Taking(w::Producing { requested: *requested }),
        d::State::Producing { requested } => w::ChangeLive::Producing(w::Producing { requested: *requested }),
        d::State::Opening { requested } => w::ChangeLive::Opening(w::Producing { requested: *requested }),
        d::State::Recreating { head } => w::ChangeLive::Recreating(w::ChangeHead { head: *head }),
        d::State::Reopening { requested, retarget } => {
            w::ChangeLive::Reopening(w::Reopening { requested: *requested, retarget: *retarget })
        }
        d::State::Checking { head } => w::ChangeLive::Checking(w::ChangeHead { head: *head }),
        d::State::CancellingGates { head } => w::ChangeLive::CancellingGates(w::ChangeHead { head: *head }),
        d::State::Judging { head, ready_since, target, asked } => w::ChangeLive::Judging(w::Judging {
            head: *head,
            ready_since: ready_since.as_nanos(),
            target: encode_judgement(target),
            asked: encode_numbers(asked),
        }),
        d::State::Gating { head, asked } => {
            w::ChangeLive::Gating(w::Gating { head: *head, asked: encode_numbers(asked) })
        }
        pending @ (d::State::Ready { .. }
        | d::State::Preparing { .. }
        | d::State::Queued { .. }
        | d::State::First { .. }
        | d::State::Updating { .. }
        | d::State::WaitingProducer { .. }
        | d::State::Resolving { .. }
        | d::State::Repairing { .. }
        | d::State::Landing { .. }
        | d::State::Landed { .. }
        | d::State::Red { .. }
        | d::State::Cleared { .. }
        | d::State::Uncleared { .. }
        | d::State::Held { .. }) => encode_progress(pending),
    }
}

fn encode_progress(value: &d::State) -> w::ChangeLive {
    match value {
        d::State::Ready { head, ready_since } => {
            w::ChangeLive::Ready(w::ReadyHead { head: *head, ready_since: ready_since.as_nanos() })
        }
        d::State::Preparing { from, main, ready_since } => {
            w::ChangeLive::Preparing(w::Preparing { from: *from, main: *main, ready_since: ready_since.as_nanos() })
        }
        d::State::Queued { head, ready_since } => {
            w::ChangeLive::Queued(w::ReadyHead { head: *head, ready_since: ready_since.as_nanos() })
        }
        d::State::First { head, ready_since } => {
            w::ChangeLive::First(w::ReadyHead { head: *head, ready_since: ready_since.as_nanos() })
        }
        d::State::Updating { from, base, ready_since } => {
            w::ChangeLive::Updating(w::Updating { from: *from, base: *base, ready_since: ready_since.as_nanos() })
        }
        d::State::WaitingProducer { base } => w::ChangeLive::WaitingProducer(w::WaitingProducer { base: *base }),
        d::State::Resolving { requested, base } => {
            w::ChangeLive::Resolving(w::Resolving { requested: *requested, base: *base })
        }
        d::State::Repairing { requested, why } => {
            w::ChangeLive::Repairing(w::Repairing { requested: *requested, why: encode_repair(*why) })
        }
        d::State::Landing { head } => w::ChangeLive::Landing(w::ChangeHead { head: *head }),
        d::State::Landed { merge, since } => {
            w::ChangeLive::Landed(w::LandedChange { merge: *merge, since: since.as_nanos() })
        }
        d::State::Red { merge, since } => {
            w::ChangeLive::Red(w::LandedChange { merge: *merge, since: since.as_nanos() })
        }
        d::State::Cleared { merge } => w::ChangeLive::Cleared(w::MergeChange { merge: *merge }),
        d::State::Uncleared { merge } => w::ChangeLive::Uncleared(w::MergeChange { merge: *merge }),
        d::State::Held { .. } => unreachable!("the bounded loop unwrapped every hold"),
        d::State::NoChange { .. }
        | d::State::Ended
        | d::State::Taking { .. }
        | d::State::Producing { .. }
        | d::State::Opening { .. }
        | d::State::Recreating { .. }
        | d::State::Reopening { .. }
        | d::State::Checking { .. }
        | d::State::CancellingGates { .. }
        | d::State::Judging { .. }
        | d::State::Gating { .. } => unreachable!("preparation phases translated above"),
    }
}

fn decode_live(value: w::ChangeLive) -> d::State {
    match value {
        w::ChangeLive::NoChange(value) => d::State::NoChange { words: value.words },
        w::ChangeLive::Ended => d::State::Ended,
        w::ChangeLive::Taking(value) => d::State::Taking { requested: value.requested },
        w::ChangeLive::Producing(value) => {
            let w::Producing { requested } = value;
            d::State::Producing { requested }
        }
        w::ChangeLive::Opening(value) => {
            let w::Producing { requested } = value;
            d::State::Opening { requested }
        }
        w::ChangeLive::Recreating(value) => {
            let w::ChangeHead { head } = value;
            d::State::Recreating { head }
        }
        w::ChangeLive::Reopening(value) => {
            let w::Reopening { requested, retarget } = value;
            d::State::Reopening { requested, retarget }
        }
        w::ChangeLive::Checking(value) => {
            let w::ChangeHead { head } = value;
            d::State::Checking { head }
        }
        w::ChangeLive::CancellingGates(value) => {
            let w::ChangeHead { head } = value;
            d::State::CancellingGates { head }
        }
        w::ChangeLive::Judging(value) => {
            let w::Judging { head, ready_since, target, asked } = value;
            d::State::Judging {
                head,
                ready_since: Wall::from_nanos(ready_since),
                target: decode_judgement(target),
                asked: asked.into_boxed(),
            }
        }
        w::ChangeLive::Gating(value) => {
            let w::Gating { head, asked } = value;
            d::State::Gating { head, asked: asked.into_boxed() }
        }
        pending @ (w::ChangeLive::Ready(_)
        | w::ChangeLive::Preparing(_)
        | w::ChangeLive::Queued(_)
        | w::ChangeLive::First(_)
        | w::ChangeLive::Updating(_)
        | w::ChangeLive::WaitingProducer(_)
        | w::ChangeLive::Resolving(_)
        | w::ChangeLive::Repairing(_)
        | w::ChangeLive::Landing(_)
        | w::ChangeLive::Landed(_)
        | w::ChangeLive::Red(_)
        | w::ChangeLive::Cleared(_)
        | w::ChangeLive::Uncleared(_)) => decode_progress(pending),
    }
}

fn decode_progress(value: w::ChangeLive) -> d::State {
    match value {
        w::ChangeLive::Ready(value) => {
            let w::ReadyHead { head, ready_since } = value;
            d::State::Ready { head, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::Preparing(value) => {
            let w::Preparing { from, main, ready_since } = value;
            d::State::Preparing { from, main, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::Queued(value) => {
            let w::ReadyHead { head, ready_since } = value;
            d::State::Queued { head, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::First(value) => {
            let w::ReadyHead { head, ready_since } = value;
            d::State::First { head, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::Updating(value) => {
            let w::Updating { from, base, ready_since } = value;
            d::State::Updating { from, base, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::WaitingProducer(value) => d::State::WaitingProducer { base: value.base },
        w::ChangeLive::Resolving(value) => {
            let w::Resolving { requested, base } = value;
            d::State::Resolving { requested, base }
        }
        w::ChangeLive::Repairing(value) => {
            let w::Repairing { requested, why } = value;
            d::State::Repairing { requested, why: decode_repair(why) }
        }
        w::ChangeLive::Landing(value) => {
            let w::ChangeHead { head } = value;
            d::State::Landing { head }
        }
        w::ChangeLive::Landed(value) => {
            let w::LandedChange { merge, since } = value;
            d::State::Landed { merge, since: Wall::from_nanos(since) }
        }
        w::ChangeLive::Red(value) => {
            let w::LandedChange { merge, since } = value;
            d::State::Red { merge, since: Wall::from_nanos(since) }
        }
        w::ChangeLive::Cleared(value) => d::State::Cleared { merge: value.merge },
        w::ChangeLive::Uncleared(value) => d::State::Uncleared { merge: value.merge },
        w::ChangeLive::NoChange(_)
        | w::ChangeLive::Ended
        | w::ChangeLive::Taking(_)
        | w::ChangeLive::Producing(_)
        | w::ChangeLive::Opening(_)
        | w::ChangeLive::Recreating(_)
        | w::ChangeLive::Reopening(_)
        | w::ChangeLive::Checking(_)
        | w::ChangeLive::CancellingGates(_)
        | w::ChangeLive::Judging(_)
        | w::ChangeLive::Gating(_) => unreachable!("preparation phases translated above"),
    }
}

// An ended live state's single hold stores its ending cause (temper/docs/design/domain/forge/branches.md, 2).
fn encode_change_state(state: &d::State, ended_held: Option<d::Hold>) -> w::ChangeState {
    let mut holds = List::with_capacity(8);
    if *state == d::State::Ended
        && let Some(why) = ended_held
    {
        holds.push(encode_hold(why)).expect("one ending hold");
    }
    let mut current = state;
    for _ in 0..=8_u32 {
        let d::State::Held { was, why } = current else {
            break;
        };
        holds.push(encode_hold(*why)).expect("at most 8 holds (change_state_holds)");
        current = was;
    }
    w::ChangeState { live: encode_live(current), holds }
}
fn decode_change_state(value: w::ChangeState) -> (d::State, Option<d::Hold>) {
    let w::ChangeState { live, holds } = value;
    let mut state = decode_live(live);
    if state == d::State::Ended
        && let [why] = holds.as_slice()
    {
        return (state, Some(decode_hold(why.clone())));
    }
    for why in holds.into_boxed().into_iter().rev() {
        state = d::State::Held { was: Box::new(state), why: decode_hold(why) };
    }
    (state, None)
}
pub(super) fn encode_gate(value: &d::Gate) -> w::Gate {
    crate::generated::store::gate_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_gate(value: w::Gate) -> d::Gate {
    crate::generated::store::gate_to_domain(value, &super::Context::UNBOUNDED).expect("infallible store translation")
}
fn encode_early(value: &d::Early) -> w::Early {
    let heads = match value {
        d::Early::None => return w::Early::None,
        d::Early::Started(heads) | d::Early::Spent(heads) => heads,
    };
    let mut encoded = List::with_capacity(u32::try_from(heads.len()).expect(FITS));
    for head in heads {
        encoded
            .push(crate::generated::store::started_from_to_record(head, &super::Context::UNBOUNDED).expect(FITS))
            .expect(FITS);
    }
    let heads = w::EarlyHeads { heads: encoded };
    match value {
        d::Early::None => unreachable!("no early heads returned above"),
        d::Early::Started(_) => w::Early::Started(heads),
        d::Early::Spent(_) => w::Early::Spent(heads),
    }
}

fn decode_early(value: w::Early) -> d::Early {
    let (heads, spent) = match value {
        w::Early::None => return d::Early::None,
        w::Early::Started(heads) => (heads, false),
        w::Early::Spent(heads) => (heads, true),
    };
    let heads = heads.heads;
    let mut decoded = List::with_capacity(heads.len());
    for head in &heads {
        decoded
            .push(
                crate::generated::store::started_from_to_domain(head.clone(), &super::Context::UNBOUNDED).expect(FITS),
            )
            .expect(FITS);
    }
    if spent { d::Early::Spent(decoded.into_boxed()) } else { d::Early::Started(decoded.into_boxed()) }
}

pub(super) fn encode_clock(clock: Option<d::Clock>) -> Option<w::ExecutionClock> {
    let clock = clock?;
    Some(crate::generated::store::execution_clock_to_record(&clock, &super::Context::UNBOUNDED).expect(FITS))
}

pub(super) fn decode_clock(clock: Option<w::ExecutionClock>) -> Option<d::Clock> {
    let clock = clock?;
    Some(
        crate::generated::store::execution_clock_to_domain(clock, &super::Context::UNBOUNDED)
            .expect("infallible store translation"),
    )
}

pub(super) fn encode_change(value: &d::Change) -> w::Change {
    let d::Change {
        early,
        task,
        state,
        gates,
        clean,
        repairs,
        resolutions,
        updates,
        stale,
        since,
        clock,
        ready_since,
        owns_turn,
        last_head,
        last_checked,
        last_checked_by,
        gate_failures,
        released,
        released_resolution,
        way,
        landing,
        landing_ask,
        ended_held,
    } = value;
    w::Change {
        early: encode_early(early),
        task: *task,
        state: encode_change_state(state, *ended_held),
        gates: encode_gates(gates),
        clean: encode_commits(clean),
        repairs: *repairs,
        resolutions: *resolutions,
        updates: *updates,
        stale: match stale {
            Some(value) => Some(value.as_nanos()),
            None => None,
        },
        since: since.as_nanos(),
        clock: encode_clock(*clock),
        ready_since: match ready_since {
            Some(value) => Some(value.as_nanos()),
            None => None,
        },
        owns_turn: *owns_turn,
        last_head: *last_head,
        last_checked: *last_checked,
        last_checked_by: *last_checked_by,
        gate_failures: encode_gate_failures(gate_failures),
        released_resolution: *released_resolution,
        released: match released {
            Some(why) => Some(encode_hold(*why)),
            None => None,
        },
        way: crate::generated::store::way_to_record(way, &super::Context::UNBOUNDED).expect(FITS),
        landing: *landing,
        landing_ask: match landing_ask {
            Some(ask) => Some(super::landing::encode_ask(ask.as_ref().clone())),
            None => None,
        },
    }
}
pub(super) fn decode_change(value: w::Change) -> d::Change {
    let w::Change {
        early,
        task,
        state,
        gates,
        clean,
        repairs,
        resolutions,
        updates,
        stale,
        since,
        clock,
        ready_since,
        owns_turn,
        last_head,
        last_checked,
        last_checked_by,
        gate_failures,
        released,
        released_resolution,
        way,
        landing,
        landing_ask,
    } = value;
    let (state, ended_held) = decode_change_state(state);
    d::Change {
        early: decode_early(early),
        task,
        state,
        ended_held,
        gates: decode_gates(gates),
        clean: decode_commits(clean),
        repairs,
        resolutions,
        updates,
        stale: match stale {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
        since: Wall::from_nanos(since),
        clock: decode_clock(clock),
        ready_since: match ready_since {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
        owns_turn,
        last_head,
        last_checked,
        last_checked_by,
        gate_failures: decode_gate_failures(gate_failures),
        released_resolution,
        released: match released {
            Some(why) => Some(decode_hold(why)),
            None => None,
        },
        way: crate::generated::store::way_to_domain(way, &super::Context::UNBOUNDED).expect(FITS),
        landing,
        landing_ask: match landing_ask {
            Some(ask) => Some(Box::new(super::landing::decode_ask(ask))),
            None => None,
        },
    }
}
pub(super) fn encode_change_cold(value: &top::ChangeCold) -> w::ChangeCold {
    crate::generated::store::change_cold_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_change_cold(value: w::ChangeCold) -> top::ChangeCold {
    crate::generated::store::change_cold_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_change_hot(value: &top::ChangeHot) -> w::ChangeHot {
    crate::generated::store::change_hot_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_change_hot(value: w::ChangeHot) -> Result<top::ChangeHot, super::Undecodable> {
    crate::generated::store::change_hot_to_domain(value, &super::Context::UNBOUNDED)
        .or(Err(super::Undecodable::Malformed))
}
fn encode_numbers(values: &[u64]) -> List<u64> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let item = *value;
        result.push(item).expect("same count");
    }
    result
}
fn encode_commits(values: &[[u8; 32]]) -> List<[u8; 32]> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let item = *value;
        result.push(item).expect("same count");
    }
    result
}
fn decode_commits(values: List<[u8; 32]>) -> Box<[[u8; 32]]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let item = value;
        result.push(item).expect("same count");
    }
    result.into_boxed()
}
fn encode_gates(values: &[d::Gate]) -> List<w::Gate> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let item = encode_gate(value);
        result.push(item).expect("same count");
    }
    result
}
fn decode_gates(values: List<w::Gate>) -> Box<[d::Gate]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let item = decode_gate(value);
        result.push(item).expect("same count");
    }
    result.into_boxed()
}
fn encode_gate_failures(values: &[d::GateFailure]) -> List<w::GateFailure> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        result
            .push(crate::generated::store::gate_failure_to_record(value, &super::Context::UNBOUNDED).expect(FITS))
            .expect("same count");
    }
    result
}
fn decode_gate_failures(values: List<w::GateFailure>) -> Box<[d::GateFailure]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result
            .push(crate::generated::store::gate_failure_to_domain(value, &super::Context::UNBOUNDED).expect(FITS))
            .expect("same count");
    }
    result.into_boxed()
}

fn encode_model_effort(value: jig_domain_tasks::ModelEffort) -> w::ModelEffort {
    crate::generated::store::model_effort_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_model_effort(value: w::ModelEffort) -> jig_domain_tasks::ModelEffort {
    crate::generated::store::model_effort_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub(super) fn encode_llm(value: Option<&jig_domain_tasks::LlmChoice>) -> Option<w::LlmChoice> {
    let jig_domain_tasks::LlmChoice { main, subagents, account, .. } = value?;
    let mut models = List::with_capacity(u32::try_from(subagents.len()).expect("admitted count"));
    for model in subagents {
        models.push(encode_model_effort(*model)).expect("same count");
    }
    Some(w::LlmChoice { main: encode_model_effort(*main), subagents: models, account: *account })
}
pub(super) fn decode_llm(value: Option<w::LlmChoice>) -> Option<jig_domain_tasks::LlmChoice> {
    let w::LlmChoice { main, subagents, account } = value?;
    let mut models = List::with_capacity(subagents.len());
    for model in subagents.into_boxed() {
        models.push(decode_model_effort(model)).expect("same count");
    }
    Some(jig_domain_tasks::LlmChoice {
        later: None,
        main: decode_model_effort(main),
        subagents: models.into_boxed(),
        account,
    })
}

pub(super) fn encode_judgement(value: &d::Judgement) -> w::Judgement {
    crate::generated::store::judgement_to_record(value, &super::Context::UNBOUNDED).expect(FITS)
}
pub(super) fn decode_judgement(value: w::Judgement) -> d::Judgement {
    crate::generated::store::judgement_to_domain(value, &super::Context::UNBOUNDED)
        .expect("infallible store translation")
}
pub fn authority_hold_to_record(
    value: &d::AuthorityHold,
    context: &super::Context,
) -> Result<w::AuthorityHold, w::Problem> {
    Ok(w::AuthorityHold {
        lack: crate::generated::store::authority_lack_to_record(&value.lack, context)?,
        had: value.had,
        released: value.released,
        task: value.recovery.task,
        spec: value.recovery.spec,
        hold: value.recovery.hold,
        tasks: value.recovery.tasks,
    })
}

pub fn authority_hold_to_domain(
    value: w::AuthorityHold,
    context: &super::Context,
) -> Result<d::AuthorityHold, w::Problem> {
    Ok(d::AuthorityHold {
        lack: crate::generated::store::authority_lack_to_domain(value.lack, context)?,
        had: value.had,
        released: value.released,
        recovery: d::AuthorityRecovery { task: value.task, spec: value.spec, hold: value.hold, tasks: value.tasks },
    })
}

pub fn landing_below_to_record(
    value: &d::Below,
    _context: &super::Context,
) -> Result<w::LandingBelow, w::Problem> {
    Ok(encode_below(Some(*value)).expect("present below"))
}

pub fn landing_below_to_domain(
    value: w::LandingBelow,
    _context: &super::Context,
) -> Result<d::Below, w::Problem> {
    Ok(decode_below(Some(value)).expect("present below"))
}

pub fn change_to_record(value: &d::Change, _context: &super::Context) -> Result<w::Change, w::Problem> {
    Ok(encode_change(value))
}

pub fn change_to_domain(value: w::Change, _context: &super::Context) -> Result<d::Change, w::Problem> {
    Ok(decode_change(value))
}

pub fn llm_choice_to_record(
    value: &jig_domain_tasks::LlmChoice,
    _context: &super::Context,
) -> Result<w::LlmChoice, w::Problem> {
    Ok(encode_llm(Some(value)).expect("present choice"))
}

pub fn llm_choice_to_domain(
    value: w::LlmChoice,
    _context: &super::Context,
) -> Result<jig_domain_tasks::LlmChoice, w::Problem> {
    Ok(decode_llm(Some(value)).expect("present choice"))
}

pub fn delegate_ref_to_record(
    value: &(u64, d::Delegate),
    context: &super::Context,
) -> Result<w::DelegateRef, w::Problem> {
    Ok(w::DelegateRef { task: value.0, request: crate::generated::store::delegate_to_record(&value.1, context)? })
}

pub fn delegate_ref_to_domain(
    value: w::DelegateRef,
    context: &super::Context,
) -> Result<(u64, d::Delegate), w::Problem> {
    Ok((value.task, crate::generated::store::delegate_to_domain(value.request, context)?))
}

pub fn route_to_record(value: &d::Route, _context: &super::Context) -> Result<w::Route, w::Problem> {
    Ok(match value {
        d::Route::None => w::Route::None,
        d::Route::Up => w::Route::Up,
    })
}

pub fn route_to_domain(value: w::Route, _context: &super::Context) -> Result<d::Route, w::Problem> {
    match value {
        w::Route::None => Ok(d::Route::None),
        w::Route::Up => Ok(d::Route::Up),
        w::Route::Retired => Err(w::paths::ROUTE_TAG.problem(skein_codec::Reason::Malformed)),
    }
}

pub fn stands_to_record(value: &d::Stands, _context: &super::Context) -> Result<w::Stands, w::Problem> {
    Ok(match value {
        d::Stands::Taken => w::Stands::Taken,
        d::Stands::Asked => w::Stands::Asked,
        d::Stands::Held { answer } => w::Stands::Held(w::Held { answer: answer.clone() }),
    })
}

pub fn stands_to_domain(value: w::Stands, _context: &super::Context) -> Result<d::Stands, w::Problem> {
    match value {
        w::Stands::Taken => Ok(d::Stands::Taken),
        w::Stands::Asked => Ok(d::Stands::Asked),
        w::Stands::Held(value) => Ok(d::Stands::Held { answer: value.answer }),
        w::Stands::Retired => Err(w::paths::STANDS_TAG.problem(skein_codec::Reason::Malformed)),
    }
}
