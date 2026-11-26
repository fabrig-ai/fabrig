use super::{FITS, w};
use crate::generated::store as map;
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_change as d;
use temper_engine_domain_forge_vocabulary as top;
fn encode_hold(value: d::Hold) -> w::ChangeHold {
    crate::generated::store::change_hold_to_record(&value)
}
fn decode_hold(value: w::ChangeHold) -> d::Hold {
    crate::generated::store::change_hold_to_domain(value)
}
pub(super) fn encode_below(below: Option<d::Below>) -> Option<w::LandingBelow> {
    let below = below?;
    let mut changes = List::with_capacity(4);
    for landed in below.changes.into_iter().flatten() {
        changes.push(crate::generated::store::landed_below_to_record(&landed)).expect(FITS);
    }
    Some(w::LandingBelow { green: below.green, changes, by_test: below.by_test, top: below.top })
}

pub(super) fn decode_below(below: Option<w::LandingBelow>) -> Option<d::Below> {
    let w::LandingBelow { green, changes, by_test, top } = below?;
    let mut below = d::Below { green, changes: [None; 4], by_test, top };
    for (slot, landed) in below.changes.iter_mut().zip(&changes) {
        *slot = Some(crate::generated::store::landed_below_to_domain(landed.clone()));
    }
    Some(below)
}

fn encode_repair(value: d::Repair) -> w::Repair {
    crate::generated::store::repair_to_record(&value)
}
fn decode_repair(value: w::Repair) -> d::Repair {
    crate::generated::store::repair_to_domain(value)
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
    crate::generated::store::gate_to_record(value)
}
pub(super) fn decode_gate(value: w::Gate) -> d::Gate {
    crate::generated::store::gate_to_domain(value)
}
fn encode_early(value: &d::Early) -> w::Early {
    let heads = match value {
        d::Early::None => return w::Early::None,
        d::Early::Started(heads) | d::Early::Spent(heads) => heads,
    };
    let mut encoded = List::with_capacity(u32::try_from(heads.len()).expect(FITS));
    for head in heads {
        encoded.push(crate::generated::store::started_from_to_record(head)).expect(FITS);
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
        decoded.push(crate::generated::store::started_from_to_domain(head.clone())).expect(FITS);
    }
    if spent { d::Early::Spent(decoded.into_boxed()) } else { d::Early::Started(decoded.into_boxed()) }
}

pub(super) fn encode_clock(clock: Option<d::Clock>) -> Option<w::ExecutionClock> {
    let clock = clock?;
    Some(crate::generated::store::execution_clock_to_record(&clock))
}

pub(super) fn decode_clock(clock: Option<w::ExecutionClock>) -> Option<d::Clock> {
    let clock = clock?;
    Some(crate::generated::store::execution_clock_to_domain(clock))
}

fn encode_change(value: &d::Change, previous: Option<&top::PreviousLink>) -> w::Change {
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
        ended_held,
    } = value;
    let (landing, landing_ask) = match previous {
        Some(previous) => (
            previous.landing,
            match &previous.ask {
                Some(ask) => Some(map::landing_ask_to_record(ask)),
                None => None,
            },
        ),
        None => (None, None),
    };
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
        way: crate::generated::store::way_to_record(way),
        landing,
        landing_ask,
    }
}

fn decode_change(value: w::Change) -> (d::Change, Option<top::PreviousLink>) {
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
    let previous = if landing.is_none() && landing_ask.is_none() {
        None
    } else {
        let ask = match landing_ask {
            Some(ask) => Some(map::landing_ask_to_domain(ask)),
            None => None,
        };
        Some(top::PreviousLink { landing, ask })
    };
    let change = d::Change {
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
        way: crate::generated::store::way_to_domain(way),
    };
    (change, previous)
}
pub(super) fn encode_change_cold(value: &top::ChangeCold) -> w::ChangeCold {
    crate::generated::store::change_cold_to_record(value).expect(FITS)
}
pub(super) fn decode_change_cold(value: w::ChangeCold) -> top::ChangeCold {
    crate::generated::store::change_cold_to_domain(value)
}
// The wire nests the compatibility fields inside Change; the domain
// keeps them beside its policy state (temper/docs/design/domain/landing-members.md, 5).
pub(super) fn encode_change_hot(value: &top::ChangeHot) -> w::ChangeHot {
    let mut signed = List::with_capacity(u32::try_from(value.signed.len()).expect(FITS));
    for sign in &value.signed {
        signed.push(map::sign_off_to_record(sign)).expect(FITS);
    }
    let mut verdicts = List::with_capacity(u32::try_from(value.verdicts.len()).expect(FITS));
    for report in &value.verdicts {
        verdicts.push(map::gate_report_to_record(report)).expect(FITS);
    }
    let mut gate_remarks = List::with_capacity(u32::try_from(value.gate_remarks.len()).expect(FITS));
    for remark in &value.gate_remarks {
        gate_remarks.push(map::gate_remark_to_record(remark)).expect(FITS);
    }
    let mut asks = List::with_capacity(u32::try_from(value.asks.len()).expect(FITS));
    for ask in &value.asks {
        asks.push(map::landing_handed_to_record(ask)).expect(FITS);
    }
    w::ChangeHot {
        runner_ci: match &value.runner_ci {
            Some(ci) => Some(map::runner_ci_to_record(ci).expect(FITS)),
            None => None,
        },
        settling: value.settling,
        close_at: match value.close_at.as_ref() {
            Some(value) => Some(map::close_at_to_record(value)),
            None => None,
        },
        room: value.room,
        delegate_head: value.delegate_head,
        delegate_flake: value.delegate_flake,
        delegate_no_test: value.delegate_no_test,
        regate: match value.regate.as_ref() {
            Some(value) => Some(map::regate_to_record(value)),
            None => None,
        },
        kept: value.kept,
        task_release: match value.task_release.as_ref() {
            Some(value) => Some(map::task_release_to_record(value)),
            None => None,
        },
        job: match value.job.as_ref() {
            Some(value) => Some(super::by_hand::history_job_record_to_record(value)),
            None => None,
        },
        signed,
        ending: match value.ending.as_ref() {
            Some(value) => Some(map::change_ending_to_record(value)),
            None => None,
        },
        repair_tasks: numbers(&value.repair_tasks),
        resolution_tasks: numbers(&value.resolution_tasks),
        result: map::change_result_to_record(&value.result),
        task: value.task,
        priority: crate::converters::signed_to_record(&value.priority),
        change: encode_change(&value.change, value.previous.as_ref()),
        ci: match value.ci.as_ref() {
            Some(value) => Some(map::change_ci_to_record(value)),
            None => None,
        },
        pull: value.pull,
        pending: value.pending,
        effect: map::effect_result_to_record(&value.effect),
        delegate: match value.delegate.as_ref() {
            Some(value) => Some(delegate_ref_to_record(value)),
            None => None,
        },
        delegate_status: map::change_status_to_record(&value.delegate_status),
        spec_revision: value.spec_revision,
        gate_revision: value.gate_revision,
        verdicts,
        gate_remarks,
        drift: match value.drift.as_ref() {
            Some(value) => Some(map::change_hold_to_record(value)),
            None => None,
        },
        queue_repair: value.queue_repair,
        queue_repairs: value.queue_repairs,
        effect_account: match value.effect_account.as_ref() {
            Some(value) => Some(map::effect_account_to_record(value)),
            None => None,
        },
        producer_failure: value.producer_failure.clone(),
        questions: map::questions_to_record(&value.questions).expect(FITS),
        follow_check: match &value.follow_check {
            Some(value) => Some(map::follow_check_progress_to_record(value).expect(FITS)),
            None => None,
        },
        merge_result: value.merge_result.clone(),
        history: match value.history.as_ref() {
            Some(value) => Some(map::change_history_to_record(value)),
            None => None,
        },
        asks,
        handed: value.handed,
    }
}

fn numbers(values: &[u64]) -> List<u64> {
    let mut numbers = List::with_capacity(u32::try_from(values.len()).expect(FITS));
    for number in values {
        numbers.push(*number).expect(FITS);
    }
    numbers
}

pub(super) fn decode_change_hot(value: w::ChangeHot) -> Result<top::ChangeHot, super::Undecodable> {
    let (change, previous) = decode_change(value.change);
    let mut signed = List::with_capacity(value.signed.len());
    for sign in value.signed.into_boxed() {
        signed.push(map::sign_off_to_domain(sign)).expect(FITS);
    }
    let mut verdicts = List::with_capacity(value.verdicts.len());
    for report in value.verdicts.into_boxed() {
        verdicts.push(map::gate_report_to_domain(report)).expect(FITS);
    }
    let mut gate_remarks = List::with_capacity(value.gate_remarks.len());
    for remark in value.gate_remarks.into_boxed() {
        gate_remarks.push(map::gate_remark_to_domain(remark)).expect(FITS);
    }
    let mut asks = List::with_capacity(value.asks.len());
    for ask in value.asks.into_boxed() {
        asks.push(map::landing_handed_to_domain(ask)).expect(FITS);
    }
    Ok(top::ChangeHot {
        runner_ci: match value.runner_ci {
            Some(ci) => Some(map::runner_ci_to_domain(ci)),
            None => None,
        },
        settling: value.settling,
        close_at: match value.close_at {
            Some(value) => Some(map::close_at_to_domain(value)),
            None => None,
        },
        room: value.room,
        delegate_head: value.delegate_head,
        delegate_flake: value.delegate_flake,
        delegate_no_test: value.delegate_no_test,
        regate: match value.regate {
            Some(value) => Some(map::regate_to_domain(value)),
            None => None,
        },
        kept: value.kept,
        task_release: match value.task_release {
            Some(value) => Some(map::task_release_to_domain(value)),
            None => None,
        },
        job: match value.job {
            Some(value) => Some(super::by_hand::history_job_record_to_domain(value)),
            None => None,
        },
        signed: signed.into_boxed(),
        ending: match value.ending {
            Some(value) => Some(map::change_ending_to_domain(value)),
            None => None,
        },
        repair_tasks: value.repair_tasks.into_boxed(),
        resolution_tasks: value.resolution_tasks.into_boxed(),
        result: map::change_result_to_domain(value.result),
        task: value.task,
        priority: crate::converters::signed_to_domain(value.priority),
        change,
        ci: match value.ci {
            Some(value) => Some(map::change_ci_to_domain(value)),
            None => None,
        },
        pull: value.pull,
        pending: value.pending,
        effect: map::effect_result_to_domain(value.effect),
        delegate: match value.delegate {
            Some(value) => Some(delegate_ref_to_domain(value)),
            None => None,
        },
        delegate_status: map::change_status_to_domain(value.delegate_status),
        spec_revision: value.spec_revision,
        gate_revision: value.gate_revision,
        verdicts: verdicts.into_boxed(),
        gate_remarks: gate_remarks.into_boxed(),
        drift: match value.drift {
            Some(value) => Some(map::change_hold_to_domain(value)),
            None => None,
        },
        queue_repair: value.queue_repair,
        queue_repairs: value.queue_repairs,
        effect_account: match value.effect_account {
            Some(value) => Some(map::effect_account_to_domain(value)),
            None => None,
        },
        producer_failure: value.producer_failure,
        questions: map::questions_to_domain(value.questions).or(Err(super::Undecodable::Malformed))?,
        follow_check: match value.follow_check {
            Some(value) => Some(map::follow_check_progress_to_domain(value)),
            None => None,
        },
        merge_result: value.merge_result,
        history: match value.history {
            Some(value) => Some(map::change_history_to_domain(value)),
            None => None,
        },
        asks: asks.into_boxed(),
        handed: value.handed,
        previous,
    })
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
        result.push(crate::generated::store::gate_failure_to_record(value)).expect("same count");
    }
    result
}
fn decode_gate_failures(values: List<w::GateFailure>) -> Box<[d::GateFailure]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        result.push(crate::generated::store::gate_failure_to_domain(value)).expect("same count");
    }
    result.into_boxed()
}

fn encode_model_effort(value: jig_domain_tasks::ModelEffort) -> w::ModelEffort {
    crate::generated::store::model_effort_to_record(&value)
}
fn decode_model_effort(value: w::ModelEffort) -> jig_domain_tasks::ModelEffort {
    crate::generated::store::model_effort_to_domain(value)
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
    crate::generated::store::judgement_to_record(value)
}
pub(super) fn decode_judgement(value: w::Judgement) -> d::Judgement {
    crate::generated::store::judgement_to_domain(value)
}
#[must_use]
pub fn authority_hold_to_record(value: &d::AuthorityHold) -> w::AuthorityHold {
    w::AuthorityHold {
        lack: crate::generated::store::authority_lack_to_record(&value.lack),
        had: value.had,
        released: value.released,
        task: value.recovery.task,
        spec: value.recovery.spec,
        hold: value.recovery.hold,
        tasks: value.recovery.tasks,
    }
}

#[must_use]
pub fn authority_hold_to_domain(value: w::AuthorityHold) -> d::AuthorityHold {
    d::AuthorityHold {
        lack: crate::generated::store::authority_lack_to_domain(value.lack),
        had: value.had,
        released: value.released,
        recovery: d::AuthorityRecovery { task: value.task, spec: value.spec, hold: value.hold, tasks: value.tasks },
    }
}

#[must_use]
pub fn landing_below_to_record(value: &d::Below) -> w::LandingBelow {
    encode_below(Some(*value)).expect("present below")
}

#[must_use]
pub fn landing_below_to_domain(value: w::LandingBelow) -> d::Below {
    decode_below(Some(value)).expect("present below")
}

#[must_use]
pub fn llm_choice_to_record(value: &jig_domain_tasks::LlmChoice) -> w::LlmChoice {
    encode_llm(Some(value)).expect("present choice")
}

#[must_use]
pub fn llm_choice_to_domain(value: w::LlmChoice) -> jig_domain_tasks::LlmChoice {
    decode_llm(Some(value)).expect("present choice")
}

#[must_use]
pub fn delegate_ref_to_record(value: &(u64, d::Delegate)) -> w::DelegateRef {
    w::DelegateRef { task: value.0, request: crate::generated::store::delegate_to_record(&value.1) }
}

#[must_use]
pub fn delegate_ref_to_domain(value: w::DelegateRef) -> (u64, d::Delegate) {
    (value.task, crate::generated::store::delegate_to_domain(value.request))
}

#[must_use]
pub fn route_to_record(value: &d::Route) -> w::Route {
    match value {
        d::Route::None => w::Route::None,
        d::Route::Up => w::Route::Up,
    }
}

pub fn route_to_domain(value: w::Route) -> Result<d::Route, w::Problem> {
    match value {
        w::Route::None => Ok(d::Route::None),
        w::Route::Up => Ok(d::Route::Up),
        w::Route::Retired => Err(w::paths::ROUTE_TAG.problem(skein_codec::Reason::Malformed)),
    }
}

#[must_use]
pub fn stands_to_record(value: &d::Stands) -> w::Stands {
    match value {
        d::Stands::Taken => w::Stands::Taken,
        d::Stands::Asked => w::Stands::Asked,
        d::Stands::Held { answer, question, blocked } => {
            let (dependency, since) = match blocked {
                Some(blocked) => (blocked.dependency, Some(blocked.since.as_nanos())),
                None => (None, None),
            };
            w::Stands::Held(w::Held { answer: answer.clone(), question: *question, dependency, since })
        }
    }
}

pub fn stands_to_domain(value: w::Stands) -> Result<d::Stands, w::Problem> {
    match value {
        w::Stands::Taken => Ok(d::Stands::Taken),
        w::Stands::Asked => Ok(d::Stands::Asked),
        w::Stands::Held(value) => Ok(d::Stands::Held {
            answer: value.answer,
            question: value.question,
            blocked: match value.since {
                Some(since) => Some(d::Blocked { dependency: value.dependency, since: Wall::from_nanos(since) }),
                None => None,
            },
        }),
        w::Stands::Retired => Err(w::paths::STANDS_TAG.problem(skein_codec::Reason::Malformed)),
    }
}
