use super::{FITS, names, w};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Wall};
use temper_engine_domain_forge_change as d;
use temper_engine_domain_forge_vocabulary as top;
fn encode_status(value: d::Status) -> w::ChangeStatus {
    match value {
        d::Status::Unknown => w::ChangeStatus::Unknown,
        d::Status::Pending => w::ChangeStatus::Pending,
        d::Status::Passed => w::ChangeStatus::Passed,
        d::Status::Failed => w::ChangeStatus::Failed,
    }
}
fn decode_status(value: w::ChangeStatus) -> d::Status {
    match value {
        w::ChangeStatus::Unknown => d::Status::Unknown,
        w::ChangeStatus::Pending => d::Status::Pending,
        w::ChangeStatus::Passed => d::Status::Passed,
        w::ChangeStatus::Failed => d::Status::Failed,
    }
}
fn encode_freshness(value: d::Freshness) -> w::Freshness {
    match value {
        d::Freshness::Exact => w::Freshness::Exact,
        d::Freshness::Clean => w::Freshness::Clean,
    }
}
fn decode_freshness(value: w::Freshness) -> d::Freshness {
    match value {
        w::Freshness::Exact => d::Freshness::Exact,
        w::Freshness::Clean => d::Freshness::Clean,
    }
}
fn encode_gate_kind(value: d::GateKind) -> w::GateKind {
    match value {
        d::GateKind::Agent => w::GateKind::Agent,
        d::GateKind::Person => w::GateKind::Person,
        d::GateKind::Check => w::GateKind::Check,
        d::GateKind::Follow => w::GateKind::Follow,
    }
}
fn decode_gate_kind(value: w::GateKind) -> d::GateKind {
    match value {
        w::GateKind::Agent => d::GateKind::Agent,
        w::GateKind::Person => d::GateKind::Person,
        w::GateKind::Check => d::GateKind::Check,
        w::GateKind::Follow => d::GateKind::Follow,
    }
}
fn encode_hold(value: d::Hold) -> w::ChangeHold {
    match value {
        d::Hold::Stalled => w::ChangeHold::Stalled,
        d::Hold::Repairs => w::ChangeHold::Repairs,
        d::Hold::Questions => w::ChangeHold::Questions,
        d::Hold::Resolutions => w::ChangeHold::Resolutions,
        d::Hold::Updates => w::ChangeHold::Updates,
        d::Hold::BranchMoved => w::ChangeHold::BranchMoved,
        d::Hold::BranchMissing => w::ChangeHold::BranchMissing,
        d::Hold::PullClosed => w::ChangeHold::PullClosed,
        d::Hold::Retargeted => w::ChangeHold::Retargeted,
        d::Hold::Failed => w::ChangeHold::Failed,
        d::Hold::Flake => w::ChangeHold::Flake,
        d::Hold::NoTest => w::ChangeHold::NoTest,
        d::Hold::Rejected => w::ChangeHold::Rejected,
        d::Hold::Authority(cause) => w::ChangeHold::Authority(w::AuthorityHold {
            lack: match cause.lack {
                d::AuthorityLack::Tasks => w::AuthorityLack::Tasks,
                d::AuthorityLack::Depth => w::AuthorityLack::Depth,
                d::AuthorityLack::Spend => w::AuthorityLack::Spend,
            },
            had: cause.had,
            task: cause.recovery.task,
            spec: cause.recovery.spec,
            hold: cause.recovery.hold,
            tasks: cause.recovery.tasks,
            released: cause.released,
        }),
    }
}
fn decode_hold(value: w::ChangeHold) -> d::Hold {
    match value {
        w::ChangeHold::Stalled => d::Hold::Stalled,
        w::ChangeHold::Repairs => d::Hold::Repairs,
        w::ChangeHold::Questions => d::Hold::Questions,
        w::ChangeHold::Resolutions => d::Hold::Resolutions,
        w::ChangeHold::Updates => d::Hold::Updates,
        w::ChangeHold::BranchMoved => d::Hold::BranchMoved,
        w::ChangeHold::BranchMissing => d::Hold::BranchMissing,
        w::ChangeHold::PullClosed => d::Hold::PullClosed,
        w::ChangeHold::Retargeted => d::Hold::Retargeted,
        w::ChangeHold::Failed => d::Hold::Failed,
        w::ChangeHold::Flake => d::Hold::Flake,
        w::ChangeHold::NoTest => d::Hold::NoTest,
        w::ChangeHold::Rejected => d::Hold::Rejected,
        w::ChangeHold::Authority(cause) => d::Hold::Authority(d::AuthorityHold {
            lack: match cause.lack {
                w::AuthorityLack::Tasks => d::AuthorityLack::Tasks,
                w::AuthorityLack::Depth => d::AuthorityLack::Depth,
                w::AuthorityLack::Spend => d::AuthorityLack::Spend,
            },
            had: cause.had,
            recovery: d::AuthorityRecovery { task: cause.task, spec: cause.spec, hold: cause.hold, tasks: cause.tasks },
            released: cause.released,
        }),
    }
}
fn encode_effect_result(value: d::EffectResult) -> w::EffectResult {
    match value {
        d::EffectResult::None => w::EffectResult::None,
        d::EffectResult::Pending => w::EffectResult::Pending,
        d::EffectResult::Made => w::EffectResult::Made,
        d::EffectResult::Conflict => w::EffectResult::Conflict,
        d::EffectResult::Failed => w::EffectResult::Failed,
        d::EffectResult::Stale => w::EffectResult::Stale,
    }
}
fn decode_effect_result(value: w::EffectResult) -> d::EffectResult {
    match value {
        w::EffectResult::None => d::EffectResult::None,
        w::EffectResult::Pending => d::EffectResult::Pending,
        w::EffectResult::Made => d::EffectResult::Made,
        w::EffectResult::Conflict => d::EffectResult::Conflict,
        w::EffectResult::Failed => d::EffectResult::Failed,
        w::EffectResult::Stale => d::EffectResult::Stale,
    }
}
pub(super) fn encode_below(below: Option<d::Below>) -> Option<w::LandingBelow> {
    let below = below?;
    let mut changes = List::with_capacity(4);
    for landed in below.changes.into_iter().flatten() {
        changes.push(w::LandedBelow { change: landed.change, pull: landed.pull }).expect(FITS);
    }
    Some(w::LandingBelow { green: below.green, changes, by_test: below.by_test, top: below.top })
}

pub(super) fn decode_below(below: Option<w::LandingBelow>) -> Option<d::Below> {
    let w::LandingBelow { green, changes, by_test, top } = below?;
    let mut below = d::Below { green, changes: [None; 4], by_test, top };
    for (slot, landed) in below.changes.iter_mut().zip(&changes) {
        *slot = Some(d::Landed { change: landed.change, pull: landed.pull });
    }
    Some(below)
}

fn encode_repair(value: d::Repair) -> w::Repair {
    match value {
        d::Repair::LandingAt { gate, candidate, below } => {
            w::Repair::LandingAt(w::LandingRepairAt { gate, candidate, below: encode_below(below) })
        }
        d::Repair::Ci => w::Repair::Ci,
        d::Repair::Review { head } => w::Repair::Review(w::Commit { commit: head }),
        d::Repair::Gate(number) => w::Repair::Gate(w::Number { number }),
        d::Repair::Semantic => w::Repair::Semantic,
        d::Repair::Landing { gate } => w::Repair::Landing(w::LandingRepair { gate }),
    }
}
fn decode_repair(value: w::Repair) -> d::Repair {
    match value {
        w::Repair::LandingAt(value) => {
            let w::LandingRepairAt { gate, candidate, below } = value;
            d::Repair::LandingAt { gate, candidate, below: decode_below(below) }
        }
        w::Repair::Ci => d::Repair::Ci,
        w::Repair::Review(value) => d::Repair::Review { head: value.commit },
        w::Repair::Gate(value) => d::Repair::Gate(value.number),
        w::Repair::Semantic => d::Repair::Semantic,
        w::Repair::Landing(value) => d::Repair::Landing { gate: value.gate },
    }
}
fn encode_delegate(value: d::Delegate) -> w::Delegate {
    match value {
        d::Delegate::Produce => w::Delegate::Produce,
        d::Delegate::Close => w::Delegate::Close,
        d::Delegate::Repair(why) => w::Delegate::Repair(w::RepairValue { why: encode_repair(why) }),
        d::Delegate::Resolve { base } => w::Delegate::Resolve(w::Commit { commit: base }),
        d::Delegate::Merge { base, from } => {
            w::Delegate::Merge(w::MergeDelegate { base: w::Commit { commit: base }, from })
        }
        d::Delegate::Gate { number, head } => w::Delegate::Gate(w::DelegateGate { number, head }),
    }
}
fn decode_delegate(value: w::Delegate) -> d::Delegate {
    match value {
        w::Delegate::Produce => d::Delegate::Produce,
        w::Delegate::Close => d::Delegate::Close,
        w::Delegate::Repair(value) => d::Delegate::Repair(decode_repair(value.why)),
        w::Delegate::Resolve(value) => d::Delegate::Resolve { base: value.commit },
        w::Delegate::Merge(value) => {
            let w::MergeDelegate { base, from } = value;
            d::Delegate::Merge { base: base.commit, from }
        }
        w::Delegate::Gate(value) => {
            let w::DelegateGate { number, head } = value;
            d::Delegate::Gate { number, head }
        }
    }
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
    let d::Gate { number, kind, blocking, freshness, eager } = value;
    w::Gate {
        number: *number,
        kind: encode_gate_kind(*kind),
        blocking: *blocking,
        freshness: encode_freshness(*freshness),
        eager: *eager,
    }
}
pub(super) fn decode_gate(value: w::Gate) -> d::Gate {
    let w::Gate { number, kind, blocking, freshness, eager } = value;
    d::Gate { number, kind: decode_gate_kind(kind), blocking, freshness: decode_freshness(freshness), eager }
}
pub(super) fn encode_gate_report(value: &d::GateReport) -> w::GateReport {
    let d::GateReport { number, head, status } = value;
    w::GateReport { number: *number, head: *head, status: encode_status(*status) }
}
pub(super) fn decode_gate_report(value: w::GateReport) -> d::GateReport {
    let w::GateReport { number, head, status } = value;
    d::GateReport { number, head, status: decode_status(status) }
}
pub(super) fn encode_gate_remark(value: &top::GateRemark) -> w::GateRemark {
    let top::GateRemark { number, head, words } = value;
    w::GateRemark { number: *number, head: *head, words: words.clone() }
}
pub(super) fn decode_gate_remark(value: w::GateRemark) -> top::GateRemark {
    let w::GateRemark { number, head, words } = value;
    top::GateRemark { number, head, words }
}
fn encode_early(value: &d::Early) -> w::Early {
    let heads = match value {
        d::Early::None => return w::Early::None,
        d::Early::Started(heads) | d::Early::Spent(heads) => heads,
    };
    let mut encoded = List::with_capacity(u32::try_from(heads.len()).expect(FITS));
    for head in heads {
        encoded.push(w::StartedFrom { task: head.task, head: head.head }).expect(FITS);
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
        decoded.push(d::StartedFrom { task: head.task, head: head.head }).expect(FITS);
    }
    if spent { d::Early::Spent(decoded.into_boxed()) } else { d::Early::Started(decoded.into_boxed()) }
}

pub(super) fn encode_clock(clock: Option<d::Clock>) -> Option<w::ExecutionClock> {
    let clock = clock?;
    Some(w::ExecutionClock {
        since: clock.since.as_nanos(),
        head: clock.head,
        run: clock.run,
        attempt: clock.attempt,
        ran: clock.ran.as_nanos(),
        waited: clock.waited.as_nanos(),
        unknown: clock.unknown.as_nanos(),
        at: clock.at.as_nanos(),
        seen: match clock.seen {
            d::Execution::Unknown => w::Execution::Unknown,
            d::Execution::Waiting => w::Execution::Waiting,
            d::Execution::Running => w::Execution::Running,
        },
    })
}

pub(super) fn decode_clock(clock: Option<w::ExecutionClock>) -> Option<d::Clock> {
    let clock = clock?;
    Some(d::Clock {
        since: Wall::from_nanos(clock.since),
        head: clock.head,
        run: clock.run,
        attempt: clock.attempt,
        ran: Duration::from_nanos(clock.ran),
        waited: Duration::from_nanos(clock.waited),
        unknown: Duration::from_nanos(clock.unknown),
        at: Wall::from_nanos(clock.at),
        seen: match clock.seen {
            w::Execution::Unknown => d::Execution::Unknown,
            w::Execution::Waiting => d::Execution::Waiting,
            w::Execution::Running => d::Execution::Running,
        },
    })
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
        way: match way {
            d::Way::Queue => w::Way::Queue,
            d::Way::Turn => w::Way::Turn,
        },
        landing: *landing,
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
        way: match way {
            w::Way::Queue => d::Way::Queue,
            w::Way::Turn => d::Way::Turn,
        },
        landing,
    }
}
pub(super) fn encode_change_cold(value: &top::ChangeCold) -> w::ChangeCold {
    let top::ChangeCold {
        source,
        task,
        repository,
        branch,
        base,
        goal,
        milestone,
        base_repair,
        producing_charter,
        review_charter,
        producing_llm,
        review_llm,
        repairs,
        catch_up,
        close,
        title,
        body,
        follows,
        report,
    } = value;
    w::ChangeCold {
        source: encode_source(source.as_ref()),
        task: *task,
        repository: names::encode_repository(*repository),
        branch: branch.clone(),
        base: base.clone(),
        title: title.clone(),
        body: body.clone(),
        base_repair: *base_repair,
        producing_charter: *producing_charter,
        review_charter: *review_charter,
        producing_llm: encode_llm(producing_llm.as_ref()),
        review_llm: encode_llm(review_llm.as_ref()),
        report: report.clone(),
        repairs: *repairs,
        follows: match follows {
            Some(value) => Some(w::ChangeFollows { follow: value.follow, commits: encode_commits(&value.commits) }),
            None => None,
        },
        catch_up: *catch_up,
        close: close.clone(),
        goal: *goal,
        milestone: *milestone,
    }
}
pub(super) fn decode_change_cold(value: w::ChangeCold) -> top::ChangeCold {
    let w::ChangeCold {
        source,
        task,
        repository,
        branch,
        base,
        goal,
        milestone,
        base_repair,
        producing_charter,
        review_charter,
        producing_llm,
        review_llm,
        repairs,
        catch_up,
        close,
        title,
        body,
        follows,
        report,
    } = value;
    top::ChangeCold {
        source: decode_source(source),
        goal,
        milestone,
        task,
        repository: names::decode_repository(repository),
        branch,
        base,
        title,
        body,
        base_repair,
        producing_charter,
        review_charter,
        producing_llm: decode_llm(producing_llm),
        review_llm: decode_llm(review_llm),
        report,
        repairs,
        follows: match follows {
            Some(value) => {
                let w::ChangeFollows { follow, commits } = value;
                Some(top::ChangeFollows { follow, commits: commits.into_boxed() })
            }
            None => None,
        },
        catch_up,
        close,
    }
}
pub(super) fn encode_change_hot(value: &top::ChangeHot) -> w::ChangeHot {
    let top::ChangeHot {
        regate,
        room,
        kept,
        task_release,
        job,
        signed,
        ending,
        repair_tasks,
        resolution_tasks,
        result,
        task,
        priority,
        change,
        ci,
        pull,
        pending,
        effect,
        delegate,
        delegate_status,
        delegate_flake,
        delegate_no_test,
        verdicts,
        gate_remarks,
        drift,
        queue_repair,
        queue_repairs,
        effect_account,
        producer_failure,
        questions,
        follow_check,
        merge_result,
        history,
        delegate_head,
    } = value;
    w::ChangeHot {
        regate: match regate {
            Some(value) => {
                Some(w::Regate { receipt: super::landing::encode_receipt_key(&value.receipt), run: value.run })
            }
            None => None,
        },
        room: *room,
        kept: *kept,
        task_release: match task_release {
            Some(release) => Some(w::TaskRelease { revision: release.revision, at: release.at.as_nanos() }),
            None => None,
        },
        job: super::history::encode_job_record(job.as_ref()),
        signed: super::landing_log::encode_signed(signed),
        result: encode_result(result),
        task: *task,
        priority: u32::from_be_bytes(priority.to_be_bytes()),
        change: encode_change(change),
        ci: match ci {
            Some(read) => Some(w::ChangeCi { head: read.head, status: encode_status(read.status) }),
            None => None,
        },
        pull: *pull,
        pending: *pending,
        effect: encode_effect_result(*effect),
        delegate: match delegate {
            Some((task, request)) => Some(w::DelegateRef { task: *task, request: encode_delegate(*request) }),
            None => None,
        },
        delegate_status: encode_status(*delegate_status),
        delegate_flake: *delegate_flake,
        delegate_no_test: *delegate_no_test,
        delegate_head: *delegate_head,
        verdicts: encode_gate_reports(verdicts),
        gate_remarks: encode_gate_remarks(gate_remarks),
        drift: match drift {
            Some(value) => Some(encode_hold(*value)),
            None => None,
        },
        queue_repair: *queue_repair,
        queue_repairs: *queue_repairs,
        questions: encode_questions(questions),
        producer_failure: producer_failure.clone(),
        follow_check: match follow_check {
            Some(check) => Some(encode_check(check)),
            None => None,
        },
        merge_result: merge_result.clone(),
        history: match history {
            Some(value) => Some(super::history::encode_change_history(value)),
            None => None,
        },
        ending: encode_ending(ending.as_ref()),
        repair_tasks: encode_numbers(repair_tasks),
        resolution_tasks: encode_numbers(resolution_tasks),
        effect_account: match effect_account {
            Some(account) => Some(w::EffectAccount { entry: account.entry, words: account.words.clone() }),
            None => None,
        },
    }
}
pub(super) fn decode_change_hot(value: w::ChangeHot) -> Result<top::ChangeHot, super::Undecodable> {
    let w::ChangeHot {
        regate,
        room,
        kept,
        task_release,
        job,
        signed,
        ending,
        repair_tasks,
        resolution_tasks,
        result,
        task,
        priority,
        change,
        ci,
        pull,
        pending,
        effect,
        delegate,
        delegate_status,
        delegate_flake,
        delegate_no_test,
        verdicts,
        gate_remarks,
        drift,
        queue_repair,
        queue_repairs,
        effect_account,
        producer_failure,
        questions,
        follow_check,
        merge_result,
        history,
        delegate_head,
    } = value;
    Ok(top::ChangeHot {
        regate: decode_regate(regate),
        room,
        kept,
        task_release: match task_release {
            Some(release) => {
                let w::TaskRelease { revision, at } = release;
                Some(top::TaskRelease { revision, at: Wall::from_nanos(at) })
            }
            None => None,
        },
        job: super::history::decode_job_record(job),
        signed: super::landing_log::decode_signed(signed),
        ending: decode_ending(ending),
        repair_tasks: repair_tasks.into_boxed(),
        resolution_tasks: resolution_tasks.into_boxed(),
        result: decode_result(result),
        task,
        priority: i32::from_be_bytes(priority.to_be_bytes()),
        change: decode_change(change),
        ci: match ci {
            Some(read) => {
                let w::ChangeCi { head, status } = read;
                Some(d::Ci { head, status: decode_status(status) })
            }
            None => None,
        },
        pull,
        pending,
        effect: decode_effect_result(effect),
        delegate: match delegate {
            Some(value) => {
                let w::DelegateRef { task, request } = value;
                Some((task, decode_delegate(request)))
            }
            None => None,
        },
        delegate_status: decode_status(delegate_status),
        delegate_flake,
        delegate_no_test,
        delegate_head,
        verdicts: decode_gate_reports(verdicts),
        gate_remarks: decode_gate_remarks(gate_remarks),
        drift: match drift {
            Some(value) => Some(decode_hold(value)),
            None => None,
        },
        queue_repair,
        queue_repairs,
        questions: decode_questions(questions)?,
        producer_failure,
        follow_check: match follow_check {
            Some(check) => Some(decode_check(check)),
            None => None,
        },
        merge_result,
        history: match history {
            Some(value) => Some(super::history::decode_change_history(value)),
            None => None,
        },
        effect_account: decode_effect_account(effect_account),
    })
}
fn decode_effect_account(value: Option<w::EffectAccount>) -> Option<top::EffectAccount> {
    let w::EffectAccount { entry, words } = value?;
    Some(top::EffectAccount { entry, words })
}
fn decode_regate(value: Option<w::Regate>) -> Option<top::Regate> {
    let w::Regate { receipt, run } = value?;
    Some(top::Regate { receipt: super::landing::decode_receipt_key(receipt), run })
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
fn encode_gate_reports(values: &[d::GateReport]) -> List<w::GateReport> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let item = encode_gate_report(value);
        result.push(item).expect("same count");
    }
    result
}
fn decode_gate_reports(values: List<w::GateReport>) -> Box<[d::GateReport]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let item = decode_gate_report(value);
        result.push(item).expect("same count");
    }
    result.into_boxed()
}
fn encode_gate_remarks(values: &[top::GateRemark]) -> List<w::GateRemark> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let item = encode_gate_remark(value);
        result.push(item).expect("same count");
    }
    result
}
fn decode_gate_remarks(values: List<w::GateRemark>) -> Box<[top::GateRemark]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let item = decode_gate_remark(value);
        result.push(item).expect("same count");
    }
    result.into_boxed()
}

fn encode_gate_failures(values: &[d::GateFailure]) -> List<w::GateFailure> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let d::GateFailure { number, count } = value;
        result.push(w::GateFailure { number: *number, count: *count }).expect("same count");
    }
    result
}
fn decode_gate_failures(values: List<w::GateFailure>) -> Box<[d::GateFailure]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let w::GateFailure { number, count } = value;
        result.push(d::GateFailure { number, count }).expect("same count");
    }
    result.into_boxed()
}

fn encode_result_text(value: &top::ResultText) -> w::ResultText {
    w::ResultText { bytes: value.bytes.clone(), cut: value.cut }
}
fn decode_result_text(value: w::ResultText) -> top::ResultText {
    let w::ResultText { bytes, cut } = value;
    top::ResultText { bytes, cut }
}
fn encode_result(value: &top::ChangeResult) -> w::ChangeResult {
    w::ChangeResult {
        head: value.head,
        checks: encode_result_text(&value.checks),
        failed: encode_result_text(&value.failed),
        repairs: encode_result_text(&value.repairs),
        repair_count: value.repair_count,
        reviews: encode_result_text(&value.reviews),
    }
}
fn decode_result(value: w::ChangeResult) -> top::ChangeResult {
    let w::ChangeResult { head, checks, failed, repairs, repair_count, reviews } = value;
    top::ChangeResult {
        head,
        checks: decode_result_text(checks),
        failed: decode_result_text(failed),
        repairs: decode_result_text(repairs),
        repair_count,
        reviews: decode_result_text(reviews),
    }
}

fn encode_model_effort(value: jig_domain_tasks::ModelEffort) -> w::ModelEffort {
    let jig_domain_tasks::ModelEffort { model, effort } = value;
    w::ModelEffort { model, effort }
}
fn decode_model_effort(value: w::ModelEffort) -> jig_domain_tasks::ModelEffort {
    let w::ModelEffort { model, effort } = value;
    jig_domain_tasks::ModelEffort { model, effort }
}
fn encode_llm(value: Option<&jig_domain_tasks::LlmChoice>) -> Option<w::LlmChoice> {
    let jig_domain_tasks::LlmChoice { main, subagents, account, .. } = value?;
    let mut models = List::with_capacity(u32::try_from(subagents.len()).expect("admitted count"));
    for model in subagents {
        models.push(encode_model_effort(*model)).expect("same count");
    }
    Some(w::LlmChoice { main: encode_model_effort(*main), subagents: models, account: *account })
}
fn decode_llm(value: Option<w::LlmChoice>) -> Option<jig_domain_tasks::LlmChoice> {
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

fn encode_questions(value: &d::Questions) -> w::Questions {
    let mut open = List::with_capacity(u32::try_from(value.open.len()).expect("admitted count"));
    for question in &value.open {
        let stands = match &question.stands {
            d::Stands::Taken => w::Stands::Taken,
            d::Stands::Asked => w::Stands::Asked,
            d::Stands::Held { answer } => w::Stands::Held(w::Held { answer: answer.clone() }),
        };
        open.push(w::Open {
            asker: question.asker,
            number: question.number,
            line: question.line.clone(),
            words: question.words.clone(),
            stands,
        })
        .expect("same count");
    }
    let mut waiting = List::with_capacity(u32::try_from(value.waiting.len()).expect("admitted count"));
    for word in &value.waiting {
        waiting
            .push(w::Waiting { from: word.from, number: word.number, words: word.words.clone(), passed: word.passed })
            .expect("same count");
    }
    let mut answered = List::with_capacity(u32::try_from(value.answered.len()).expect("admitted count"));
    for answer in &value.answered {
        answered
            .push(w::Answered {
                asker: answer.asker,
                number: answer.number,
                line: answer.line.clone(),
                answer: answer.answer.clone(),
            })
            .expect("same count");
    }
    w::Questions {
        route: match value.route {
            d::Route::None => w::Route::None,
            d::Route::Up => w::Route::Up,
        },
        taken: value.taken,
        open,
        waiting,
        answered,
    }
}

fn decode_questions(value: w::Questions) -> Result<d::Questions, super::Undecodable> {
    let w::Questions { route, taken, open, waiting, answered } = value;
    let mut answers = List::with_capacity(answered.len());
    for answer in answered.into_boxed() {
        let w::Answered { asker, number, line, answer } = answer;
        answers.push(d::Answered { asker, number, line, answer }).expect("same count");
    }
    let mut words_waiting = List::with_capacity(waiting.len());
    for word in waiting.into_boxed() {
        let w::Waiting { from, number, words, passed } = word;
        words_waiting.push(d::Waiting { from, number, words, passed }).expect("same count");
    }
    let mut questions = List::with_capacity(open.len());
    for question in open.into_boxed() {
        let w::Open { asker, number, line, words, stands } = question;
        let stands = match stands {
            w::Stands::Taken => d::Stands::Taken,
            w::Stands::Retired => return Err(super::Undecodable::Malformed),
            w::Stands::Asked => d::Stands::Asked,
            w::Stands::Held(value) => d::Stands::Held { answer: value.answer },
        };
        questions.push(d::Open { asker, number, line, words, stands }).expect("same count");
    }
    Ok(d::Questions {
        route: match route {
            w::Route::Retired => return Err(super::Undecodable::Malformed),
            w::Route::None => d::Route::None,
            w::Route::Up => d::Route::Up,
        },
        taken,
        open: questions.into_boxed(),
        waiting: words_waiting.into_boxed(),
        answered: answers.into_boxed(),
    })
}

fn encode_ending(ending: Option<&top::Ending>) -> Option<w::ChangeEnding> {
    Some(match ending? {
        top::Ending::Landed => w::ChangeEnding::Landed,
        top::Ending::NoChange => w::ChangeEnding::NoChange,
        top::Ending::Cancelled => w::ChangeEnding::Cancelled,
        top::Ending::Failed { line } => w::ChangeEnding::Failed(w::ChangeFailure { line: line.clone() }),
    })
}
fn decode_ending(ending: Option<w::ChangeEnding>) -> Option<top::Ending> {
    Some(match ending? {
        w::ChangeEnding::Landed => top::Ending::Landed,
        w::ChangeEnding::NoChange => top::Ending::NoChange,
        w::ChangeEnding::Cancelled => top::Ending::Cancelled,
        w::ChangeEnding::Failed(line) => top::Ending::Failed { line: line.line },
    })
}

pub(super) fn encode_judgement(value: &d::Judgement) -> w::Judgement {
    w::Judgement {
        landing: value.landing,
        round: value.round,
        pull: value.pull,
        branch: value.branch.clone(),
        base: value.base.clone(),
        prefix: value.prefix,
    }
}
fn encode_check(check: &top::FollowCheck) -> w::FollowCheckProgress {
    w::FollowCheckProgress {
        follow: check.follow,
        source: check.source,
        head: check.head,
        definition: check.definition.clone(),
        key: check.key,
        flight: match &check.flight {
            Some(flight) => Some(super::tiers::encode_flight(flight)),
            None => None,
        },
        page: check.page,
        faults: check.faults,
        held: check.held.clone(),
    }
}
pub(super) fn decode_judgement(value: w::Judgement) -> d::Judgement {
    let w::Judgement { landing, round, pull, branch, base, prefix } = value;
    d::Judgement { landing, round, pull, branch, base, prefix }
}
fn decode_check(check: w::FollowCheckProgress) -> top::FollowCheck {
    let w::FollowCheckProgress { follow, source, head, definition, key, flight, page, faults, held } = check;
    top::FollowCheck {
        follow,
        source,
        head,
        definition,
        key,
        flight: match flight {
            Some(flight) => Some(super::tiers::decode_flight(flight)),
            None => None,
        },
        page,
        faults,
        held,
    }
}

fn encode_source(value: Option<&top::ChangeSource>) -> Option<w::ChangeSource> {
    let value = value?;
    Some(w::ChangeSource { branch: value.branch.clone(), at: value.at })
}
fn decode_source(value: Option<w::ChangeSource>) -> Option<top::ChangeSource> {
    let w::ChangeSource { branch, at } = value?;
    Some(top::ChangeSource { branch, at })
}
