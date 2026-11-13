use super::{FITS, names, w};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
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
        d::Hold::Rejected => w::ChangeHold::Rejected,
        d::Hold::Authority(cause) => w::ChangeHold::Authority(
            w::AuthorityHold::new(
                &w::CEILINGS,
                w::AuthorityHoldParts {
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
                },
            )
            .expect(FITS),
        ),
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
        w::ChangeHold::Rejected => d::Hold::Rejected,
        w::ChangeHold::Authority(cause) => d::Hold::Authority(d::AuthorityHold {
            lack: match cause.lack() {
                w::AuthorityLack::Tasks => d::AuthorityLack::Tasks,
                w::AuthorityLack::Depth => d::AuthorityLack::Depth,
                w::AuthorityLack::Spend => d::AuthorityLack::Spend,
            },
            had: cause.had(),
            recovery: d::AuthorityRecovery {
                task: cause.task(),
                spec: cause.spec(),
                hold: cause.hold(),
                tasks: cause.tasks(),
            },
            released: cause.released(),
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
    }
}
fn decode_effect_result(value: w::EffectResult) -> d::EffectResult {
    match value {
        w::EffectResult::None => d::EffectResult::None,
        w::EffectResult::Pending => d::EffectResult::Pending,
        w::EffectResult::Made => d::EffectResult::Made,
        w::EffectResult::Conflict => d::EffectResult::Conflict,
        w::EffectResult::Failed => d::EffectResult::Failed,
    }
}
pub(super) fn encode_below(below: Option<d::Below>) -> Option<w::LandingBelow> {
    let below = below?;
    let mut changes = List::with_capacity(4);
    for landed in below.changes.into_iter().flatten() {
        changes
            .push(
                w::LandedBelow::new(&w::CEILINGS, w::LandedBelowParts { change: landed.change, pull: landed.pull })
                    .expect(FITS),
            )
            .expect(FITS);
    }
    Some(w::LandingBelow::new(&w::CEILINGS, w::LandingBelowParts { green: below.green, changes }).expect(FITS))
}

pub(super) fn decode_below(below: Option<w::LandingBelow>) -> Option<d::Below> {
    let w::LandingBelowParts { green, changes } = below?.into_parts();
    let mut below = d::Below { green, changes: [None; 4] };
    for (slot, landed) in below.changes.iter_mut().zip(&changes) {
        *slot = Some(d::Landed { change: landed.change(), pull: landed.pull() });
    }
    Some(below)
}

fn encode_repair(value: d::Repair) -> w::Repair {
    match value {
        d::Repair::LandingAt { gate, candidate, below } => w::Repair::LandingAt(
            w::LandingRepairAt::new(
                &w::CEILINGS,
                w::LandingRepairAtParts { gate, candidate, below: encode_below(below) },
            )
            .expect(FITS),
        ),
        d::Repair::Ci => w::Repair::Ci,
        d::Repair::Review { head } => {
            w::Repair::Review(w::Commit::new(&w::CEILINGS, w::CommitParts { commit: head }).expect(FITS))
        }
        d::Repair::Gate(number) => {
            w::Repair::Gate(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS))
        }
        d::Repair::Semantic => w::Repair::Semantic,
        d::Repair::Landing { gate } => {
            w::Repair::Landing(w::LandingRepair::new(&w::CEILINGS, w::LandingRepairParts { gate }).expect(FITS))
        }
    }
}
fn decode_repair(value: w::Repair) -> d::Repair {
    match value {
        w::Repair::LandingAt(value) => {
            let w::LandingRepairAtParts { gate, candidate, below } = value.into_parts();
            d::Repair::LandingAt { gate, candidate, below: decode_below(below) }
        }
        w::Repair::Ci => d::Repair::Ci,
        w::Repair::Review(value) => d::Repair::Review { head: *value.commit() },
        w::Repair::Gate(value) => d::Repair::Gate(value.number()),
        w::Repair::Semantic => d::Repair::Semantic,
        w::Repair::Landing(value) => d::Repair::Landing { gate: value.gate() },
    }
}
fn encode_delegate(value: d::Delegate) -> w::Delegate {
    match value {
        d::Delegate::Produce => w::Delegate::Produce,
        d::Delegate::Close => w::Delegate::Close,
        d::Delegate::Repair(why) => w::Delegate::Repair(
            w::RepairValue::new(&w::CEILINGS, w::RepairValueParts { why: encode_repair(why) }).expect(FITS),
        ),
        d::Delegate::Resolve { base } => {
            w::Delegate::Resolve(w::Commit::new(&w::CEILINGS, w::CommitParts { commit: base }).expect(FITS))
        }
        d::Delegate::Merge { base, from } => w::Delegate::Merge(
            w::MergeDelegate::new(
                &w::CEILINGS,
                w::MergeDelegateParts {
                    base: w::Commit::new(&w::CEILINGS, w::CommitParts { commit: base }).expect(FITS),
                    from,
                },
            )
            .expect(FITS),
        ),
        d::Delegate::Gate { number, head } => {
            w::Delegate::Gate(w::DelegateGate::new(&w::CEILINGS, w::DelegateGateParts { number, head }).expect(FITS))
        }
    }
}
fn decode_delegate(value: w::Delegate) -> d::Delegate {
    match value {
        w::Delegate::Produce => d::Delegate::Produce,
        w::Delegate::Close => d::Delegate::Close,
        w::Delegate::Repair(value) => d::Delegate::Repair(decode_repair(value.into_parts().why)),
        w::Delegate::Resolve(value) => d::Delegate::Resolve { base: value.into_parts().commit },
        w::Delegate::Merge(value) => {
            let w::MergeDelegateParts { base, from } = value.into_parts();
            d::Delegate::Merge { base: base.into_parts().commit, from }
        }
        w::Delegate::Gate(value) => {
            let w::DelegateGateParts { number, head } = value.into_parts();
            d::Delegate::Gate { number, head }
        }
    }
}
fn encode_live(value: &d::State) -> w::ChangeLive {
    match value {
        d::State::NoChange { words } => w::ChangeLive::NoChange(
            w::NoChange::new(&w::CEILINGS, w::NoChangeParts { words: words.clone() }).expect(FITS),
        ),
        d::State::Ended => w::ChangeLive::Ended,
        d::State::Producing { requested } => w::ChangeLive::Producing(
            w::Producing::new(&w::CEILINGS, w::ProducingParts { requested: *requested }).expect(FITS),
        ),
        d::State::Opening { requested } => w::ChangeLive::Opening(
            w::Producing::new(&w::CEILINGS, w::ProducingParts { requested: *requested }).expect(FITS),
        ),
        d::State::Recreating { head } => {
            w::ChangeLive::Recreating(w::ChangeHead::new(&w::CEILINGS, w::ChangeHeadParts { head: *head }).expect(FITS))
        }
        d::State::Reopening { requested, retarget } => w::ChangeLive::Reopening(
            w::Reopening::new(&w::CEILINGS, w::ReopeningParts { requested: *requested, retarget: *retarget })
                .expect(FITS),
        ),
        d::State::Checking { head } => {
            w::ChangeLive::Checking(w::ChangeHead::new(&w::CEILINGS, w::ChangeHeadParts { head: *head }).expect(FITS))
        }
        d::State::CancellingGates { head } => w::ChangeLive::CancellingGates(
            w::ChangeHead::new(&w::CEILINGS, w::ChangeHeadParts { head: *head }).expect(FITS),
        ),
        d::State::Judging { head, ready_since, target, asked } => w::ChangeLive::Judging(
            w::Judging::new(
                &w::CEILINGS,
                w::JudgingParts {
                    head: *head,
                    ready_since: ready_since.as_nanos(),
                    target: encode_judgement(target),
                    asked: encode_numbers(asked),
                },
            )
            .expect(FITS),
        ),
        d::State::Gating { head, asked } => w::ChangeLive::Gating(
            w::Gating::new(&w::CEILINGS, w::GatingParts { head: *head, asked: encode_numbers(asked) }).expect(FITS),
        ),
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
        d::State::Ready { head, ready_since } => w::ChangeLive::Ready(
            w::ReadyHead::new(&w::CEILINGS, w::ReadyHeadParts { head: *head, ready_since: ready_since.as_nanos() })
                .expect(FITS),
        ),
        d::State::Preparing { from, main, ready_since } => w::ChangeLive::Preparing(
            w::Preparing::new(
                &w::CEILINGS,
                w::PreparingParts { from: *from, main: *main, ready_since: ready_since.as_nanos() },
            )
            .expect(FITS),
        ),
        d::State::Queued { head, ready_since } => w::ChangeLive::Queued(
            w::ReadyHead::new(&w::CEILINGS, w::ReadyHeadParts { head: *head, ready_since: ready_since.as_nanos() })
                .expect(FITS),
        ),
        d::State::First { head, ready_since } => w::ChangeLive::First(
            w::ReadyHead::new(&w::CEILINGS, w::ReadyHeadParts { head: *head, ready_since: ready_since.as_nanos() })
                .expect(FITS),
        ),
        d::State::Updating { from, base, ready_since } => w::ChangeLive::Updating(
            w::Updating::new(
                &w::CEILINGS,
                w::UpdatingParts { from: *from, base: *base, ready_since: ready_since.as_nanos() },
            )
            .expect(FITS),
        ),
        d::State::WaitingProducer { base } => w::ChangeLive::WaitingProducer(
            w::WaitingProducer::new(&w::CEILINGS, w::WaitingProducerParts { base: *base }).expect(FITS),
        ),
        d::State::Resolving { requested, base } => w::ChangeLive::Resolving(
            w::Resolving::new(&w::CEILINGS, w::ResolvingParts { requested: *requested, base: *base }).expect(FITS),
        ),
        d::State::Repairing { requested, why } => w::ChangeLive::Repairing(
            w::Repairing::new(&w::CEILINGS, w::RepairingParts { requested: *requested, why: encode_repair(*why) })
                .expect(FITS),
        ),
        d::State::Landing { head } => {
            w::ChangeLive::Landing(w::ChangeHead::new(&w::CEILINGS, w::ChangeHeadParts { head: *head }).expect(FITS))
        }
        d::State::Landed { merge, since } => w::ChangeLive::Landed(
            w::LandedChange::new(&w::CEILINGS, w::LandedChangeParts { merge: *merge, since: since.as_nanos() })
                .expect(FITS),
        ),
        d::State::Red { merge, since } => w::ChangeLive::Red(
            w::LandedChange::new(&w::CEILINGS, w::LandedChangeParts { merge: *merge, since: since.as_nanos() })
                .expect(FITS),
        ),
        d::State::Cleared { merge } => w::ChangeLive::Cleared(
            w::MergeChange::new(&w::CEILINGS, w::MergeChangeParts { merge: *merge }).expect(FITS),
        ),
        d::State::Uncleared { merge } => w::ChangeLive::Uncleared(
            w::MergeChange::new(&w::CEILINGS, w::MergeChangeParts { merge: *merge }).expect(FITS),
        ),
        d::State::Held { .. } => unreachable!("the bounded loop unwrapped every hold"),
        d::State::NoChange { .. }
        | d::State::Ended
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
        w::ChangeLive::NoChange(value) => d::State::NoChange { words: value.into_parts().words },
        w::ChangeLive::Ended => d::State::Ended,
        w::ChangeLive::Producing(value) => {
            let w::ProducingParts { requested } = value.into_parts();
            d::State::Producing { requested }
        }
        w::ChangeLive::Opening(value) => {
            let w::ProducingParts { requested } = value.into_parts();
            d::State::Opening { requested }
        }
        w::ChangeLive::Recreating(value) => {
            let w::ChangeHeadParts { head } = value.into_parts();
            d::State::Recreating { head }
        }
        w::ChangeLive::Reopening(value) => {
            let w::ReopeningParts { requested, retarget } = value.into_parts();
            d::State::Reopening { requested, retarget }
        }
        w::ChangeLive::Checking(value) => {
            let w::ChangeHeadParts { head } = value.into_parts();
            d::State::Checking { head }
        }
        w::ChangeLive::CancellingGates(value) => {
            let w::ChangeHeadParts { head } = value.into_parts();
            d::State::CancellingGates { head }
        }
        w::ChangeLive::Judging(value) => {
            let w::JudgingParts { head, ready_since, target, asked } = value.into_parts();
            d::State::Judging {
                head,
                ready_since: Wall::from_nanos(ready_since),
                target: decode_judgement(target),
                asked: asked.into_boxed(),
            }
        }
        w::ChangeLive::Gating(value) => {
            let w::GatingParts { head, asked } = value.into_parts();
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
            let w::ReadyHeadParts { head, ready_since } = value.into_parts();
            d::State::Ready { head, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::Preparing(value) => {
            let w::PreparingParts { from, main, ready_since } = value.into_parts();
            d::State::Preparing { from, main, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::Queued(value) => {
            let w::ReadyHeadParts { head, ready_since } = value.into_parts();
            d::State::Queued { head, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::First(value) => {
            let w::ReadyHeadParts { head, ready_since } = value.into_parts();
            d::State::First { head, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::Updating(value) => {
            let w::UpdatingParts { from, base, ready_since } = value.into_parts();
            d::State::Updating { from, base, ready_since: Wall::from_nanos(ready_since) }
        }
        w::ChangeLive::WaitingProducer(value) => d::State::WaitingProducer { base: value.into_parts().base },
        w::ChangeLive::Resolving(value) => {
            let w::ResolvingParts { requested, base } = value.into_parts();
            d::State::Resolving { requested, base }
        }
        w::ChangeLive::Repairing(value) => {
            let w::RepairingParts { requested, why } = value.into_parts();
            d::State::Repairing { requested, why: decode_repair(why) }
        }
        w::ChangeLive::Landing(value) => {
            let w::ChangeHeadParts { head } = value.into_parts();
            d::State::Landing { head }
        }
        w::ChangeLive::Landed(value) => {
            let w::LandedChangeParts { merge, since } = value.into_parts();
            d::State::Landed { merge, since: Wall::from_nanos(since) }
        }
        w::ChangeLive::Red(value) => {
            let w::LandedChangeParts { merge, since } = value.into_parts();
            d::State::Red { merge, since: Wall::from_nanos(since) }
        }
        w::ChangeLive::Cleared(value) => d::State::Cleared { merge: value.into_parts().merge },
        w::ChangeLive::Uncleared(value) => d::State::Uncleared { merge: value.into_parts().merge },
        w::ChangeLive::NoChange(_)
        | w::ChangeLive::Ended
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
    w::ChangeState::new(&w::CEILINGS, w::ChangeStateParts { live: encode_live(current), holds }).expect(FITS)
}
fn decode_change_state(value: w::ChangeState) -> (d::State, Option<d::Hold>) {
    let w::ChangeStateParts { live, holds } = value.into_parts();
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
    w::Gate::new(
        &w::CEILINGS,
        w::GateParts {
            number: *number,
            kind: encode_gate_kind(*kind),
            blocking: *blocking,
            freshness: encode_freshness(*freshness),
            eager: *eager,
        },
    )
    .expect(FITS)
}
pub(super) fn decode_gate(value: w::Gate) -> d::Gate {
    let w::GateParts { number, kind, blocking, freshness, eager } = value.into_parts();
    d::Gate { number, kind: decode_gate_kind(kind), blocking, freshness: decode_freshness(freshness), eager }
}
pub(super) fn encode_gate_report(value: &d::GateReport) -> w::GateReport {
    let d::GateReport { number, head, status } = value;
    w::GateReport::new(
        &w::CEILINGS,
        w::GateReportParts { number: *number, head: *head, status: encode_status(*status) },
    )
    .expect(FITS)
}
pub(super) fn decode_gate_report(value: w::GateReport) -> d::GateReport {
    let w::GateReportParts { number, head, status } = value.into_parts();
    d::GateReport { number, head, status: decode_status(status) }
}
pub(super) fn encode_gate_remark(value: &top::GateRemark) -> w::GateRemark {
    let top::GateRemark { number, head, words } = value;
    w::GateRemark::new(&w::CEILINGS, w::GateRemarkParts { number: *number, head: *head, words: words.clone() })
        .expect(FITS)
}
pub(super) fn decode_gate_remark(value: w::GateRemark) -> top::GateRemark {
    let w::GateRemarkParts { number, head, words } = value.into_parts();
    top::GateRemark { number, head, words }
}
fn encode_early(value: &d::Early) -> w::Early {
    let heads = match value {
        d::Early::None => return w::Early::None,
        d::Early::Started(heads) | d::Early::Spent(heads) => heads,
    };
    let mut encoded = List::with_capacity(u32::try_from(heads.len()).expect(FITS));
    for head in heads {
        encoded
            .push(
                w::StartedFrom::new(&w::CEILINGS, w::StartedFromParts { task: head.task, head: head.head })
                    .expect(FITS),
            )
            .expect(FITS);
    }
    let heads = w::EarlyHeads::new(&w::CEILINGS, w::EarlyHeadsParts { heads: encoded }).expect(FITS);
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
    let heads = heads.into_parts().heads;
    let mut decoded = List::with_capacity(heads.len());
    for head in &heads {
        decoded.push(d::StartedFrom { task: head.task(), head: *head.head() }).expect(FITS);
    }
    if spent { d::Early::Spent(decoded.into_boxed()) } else { d::Early::Started(decoded.into_boxed()) }
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
        since,
        ready_since,
        owns_turn,
        last_head,
        gate_failures,
        released,
        way,
        landing,
        ended_held,
    } = value;
    w::Change::new(
        &w::CEILINGS,
        w::ChangeParts {
            early: encode_early(early),
            task: *task,
            state: encode_change_state(state, *ended_held),
            gates: encode_gates(gates),
            clean: encode_commits(clean),
            repairs: *repairs,
            resolutions: *resolutions,
            updates: *updates,
            since: since.as_nanos(),
            ready_since: match ready_since {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
            owns_turn: *owns_turn,
            last_head: *last_head,
            gate_failures: encode_gate_failures(gate_failures),
            released: match released {
                Some(why) => Some(encode_hold(*why)),
                None => None,
            },
            way: match way {
                d::Way::Queue => w::Way::Queue,
                d::Way::Turn => w::Way::Turn,
            },
            landing: *landing,
        },
    )
    .expect(FITS)
}
pub(super) fn decode_change(value: w::Change) -> d::Change {
    let w::ChangeParts {
        early,
        task,
        state,
        gates,
        clean,
        repairs,
        resolutions,
        updates,
        since,
        ready_since,
        owns_turn,
        last_head,
        gate_failures,
        released,
        way,
        landing,
    } = value.into_parts();
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
        since: Wall::from_nanos(since),
        ready_since: match ready_since {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
        owns_turn,
        last_head,
        gate_failures: decode_gate_failures(gate_failures),
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
#[expect(
    clippy::too_many_lines,
    reason = "one stored change maps its captured role choices and question relay together"
)]
pub(super) fn encode_change_row(value: &top::ChangeRow) -> w::ChangeRow {
    let top::ChangeRow {
        job,
        signed,
        task,
        repository,
        branch,
        base,
        title,
        body,
        priority,
        change,
        ci,
        pull,
        pending,
        effect,
        delegate,
        delegate_status,
        verdicts,
        gate_remarks,
        drift,
        base_repair,
        queue_repair,
        queue_repairs,
        producing_charter,
        review_charter,
        producing_llm,
        review_llm,
        questions,
        result,
        effect_account,
        producer_failure,
        report,
        repairs,
        follows,
        follow_check,
        catch_up,
        close,
        merge_result,
        history,
        goal,
        ending,
        repair_tasks,
        resolution_tasks,
    } = value;
    w::ChangeRow::new(
        &w::CEILINGS,
        w::ChangeRowParts {
            job: super::history::encode_job_record(job.as_ref()),
            signed: super::landing_log::encode_signed(signed),
            result: encode_result(result),
            task: *task,
            repository: names::encode_repository(*repository),
            branch: branch.clone(),
            base: base.clone(),
            title: title.clone(),
            body: body.clone(),
            priority: u32::from_be_bytes(priority.to_be_bytes()),
            change: encode_change(change),
            ci: match ci {
                Some(read) => Some(
                    w::ChangeCi::new(
                        &w::CEILINGS,
                        w::ChangeCiParts { head: read.head, status: encode_status(read.status) },
                    )
                    .expect(FITS),
                ),
                None => None,
            },
            pull: *pull,
            pending: *pending,
            effect: encode_effect_result(*effect),
            delegate: match delegate {
                Some((task, request)) => Some(
                    w::DelegateRef::new(
                        &w::CEILINGS,
                        w::DelegateRefParts { task: *task, request: encode_delegate(*request) },
                    )
                    .expect(FITS),
                ),
                None => None,
            },
            delegate_status: encode_status(*delegate_status),
            verdicts: encode_gate_reports(verdicts),
            gate_remarks: encode_gate_remarks(gate_remarks),
            drift: match drift {
                Some(value) => Some(encode_hold(*value)),
                None => None,
            },
            base_repair: *base_repair,
            queue_repair: *queue_repair,
            queue_repairs: *queue_repairs,
            producing_charter: *producing_charter,
            review_charter: *review_charter,
            producing_llm: encode_llm(producing_llm.as_ref()),
            review_llm: encode_llm(review_llm.as_ref()),
            questions: encode_questions(questions),
            producer_failure: producer_failure.clone(),
            report: report.clone(),
            repairs: *repairs,
            follows: match follows {
                Some(value) => Some(
                    w::ChangeFollows::new(
                        &w::CEILINGS,
                        w::ChangeFollowsParts { follow: value.follow, commits: encode_commits(&value.commits) },
                    )
                    .expect(FITS),
                ),
                None => None,
            },
            follow_check: match follow_check {
                Some(check) => Some(encode_check(check)),
                None => None,
            },
            catch_up: *catch_up,
            close: close.clone(),
            merge_result: merge_result.clone(),
            history: match history {
                Some(value) => Some(super::history::encode_change_history(value)),
                None => None,
            },
            goal: *goal,
            ending: encode_ending(ending.as_ref()),
            repair_tasks: encode_numbers(repair_tasks),
            resolution_tasks: encode_numbers(resolution_tasks),
            effect_account: match effect_account {
                Some(account) => Some(
                    w::EffectAccount::new(
                        &w::CEILINGS,
                        w::EffectAccountParts { entry: account.entry, words: account.words.clone() },
                    )
                    .expect(FITS),
                ),
                None => None,
            },
        },
    )
    .expect(FITS)
}
#[expect(clippy::too_many_lines, reason = "the durable change account translates each row field in place")]
pub(super) fn decode_change_row(value: w::ChangeRow) -> Result<top::ChangeRow, super::Undecodable> {
    let w::ChangeRowParts {
        job,
        signed,
        task,
        repository,
        branch,
        base,
        title,
        body,
        priority,
        change,
        ci,
        pull,
        pending,
        effect,
        delegate,
        delegate_status,
        verdicts,
        gate_remarks,
        drift,
        base_repair,
        queue_repair,
        queue_repairs,
        producing_charter,
        review_charter,
        producing_llm,
        review_llm,
        questions,
        result,
        effect_account,
        producer_failure,
        report,
        repairs,
        follows,
        follow_check,
        catch_up,
        close,
        merge_result,
        history,
        goal,
        ending,
        repair_tasks,
        resolution_tasks,
    } = value.into_parts();
    Ok(top::ChangeRow {
        job: super::history::decode_job_record(job),
        signed: super::landing_log::decode_signed(signed),
        goal,
        ending: decode_ending(ending),
        repair_tasks: repair_tasks.into_boxed(),
        resolution_tasks: resolution_tasks.into_boxed(),
        result: decode_result(result),
        task,
        repository: names::decode_repository(repository),
        branch,
        base,
        title,
        body,
        priority: i32::from_be_bytes(priority.to_be_bytes()),
        change: decode_change(change),
        ci: match ci {
            Some(read) => {
                let w::ChangeCiParts { head, status } = read.into_parts();
                Some(d::Ci { head, status: decode_status(status) })
            }
            None => None,
        },
        pull,
        pending,
        effect: decode_effect_result(effect),
        delegate: match delegate {
            Some(value) => {
                let w::DelegateRefParts { task, request } = value.into_parts();
                Some((task, decode_delegate(request)))
            }
            None => None,
        },
        delegate_status: decode_status(delegate_status),
        verdicts: decode_gate_reports(verdicts),
        gate_remarks: decode_gate_remarks(gate_remarks),
        drift: match drift {
            Some(value) => Some(decode_hold(value)),
            None => None,
        },
        base_repair,
        queue_repair,
        queue_repairs,
        producing_charter,
        review_charter,
        producing_llm: decode_llm(producing_llm),
        review_llm: decode_llm(review_llm),
        questions: decode_questions(questions)?,
        producer_failure,
        report,
        repairs,
        follows: match follows {
            Some(value) => {
                let w::ChangeFollowsParts { follow, commits } = value.into_parts();
                Some(top::ChangeFollows { follow, commits: commits.into_boxed() })
            }
            None => None,
        },
        follow_check: match follow_check {
            Some(check) => Some(decode_check(check)),
            None => None,
        },
        catch_up,
        close,
        merge_result,
        history: match history {
            Some(value) => Some(super::history::decode_change_history(value)),
            None => None,
        },
        effect_account: match effect_account {
            Some(account) => {
                let w::EffectAccountParts { entry, words } = account.into_parts();
                Some(top::EffectAccount { entry, words })
            }
            None => None,
        },
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
        result
            .push(
                w::GateFailure::new(&w::CEILINGS, w::GateFailureParts { number: *number, count: *count }).expect(FITS),
            )
            .expect("same count");
    }
    result
}
fn decode_gate_failures(values: List<w::GateFailure>) -> Box<[d::GateFailure]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let w::GateFailureParts { number, count } = value.into_parts();
        result.push(d::GateFailure { number, count }).expect("same count");
    }
    result.into_boxed()
}

fn encode_result_text(value: &top::ResultText) -> w::ResultText {
    w::ResultText::new(&w::CEILINGS, w::ResultTextParts { bytes: value.bytes.clone(), cut: value.cut }).expect(FITS)
}
fn decode_result_text(value: w::ResultText) -> top::ResultText {
    let w::ResultTextParts { bytes, cut } = value.into_parts();
    top::ResultText { bytes, cut }
}
fn encode_result(value: &top::ChangeResult) -> w::ChangeResult {
    w::ChangeResult::new(
        &w::CEILINGS,
        w::ChangeResultParts {
            head: value.head,
            checks: encode_result_text(&value.checks),
            failed: encode_result_text(&value.failed),
            repairs: encode_result_text(&value.repairs),
            repair_count: value.repair_count,
            reviews: encode_result_text(&value.reviews),
        },
    )
    .expect(FITS)
}
fn decode_result(value: w::ChangeResult) -> top::ChangeResult {
    let w::ChangeResultParts { head, checks, failed, repairs, repair_count, reviews } = value.into_parts();
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
    w::ModelEffort::new(&w::CEILINGS, w::ModelEffortParts { model, effort }).expect(FITS)
}
fn decode_model_effort(value: w::ModelEffort) -> jig_domain_tasks::ModelEffort {
    let w::ModelEffortParts { model, effort } = value.into_parts();
    jig_domain_tasks::ModelEffort { model, effort }
}
fn encode_llm(value: Option<&jig_domain_tasks::LlmChoice>) -> Option<w::LlmChoice> {
    let jig_domain_tasks::LlmChoice { main, subagents, account, .. } = value?;
    let mut models = List::with_capacity(u32::try_from(subagents.len()).expect("admitted count"));
    for model in subagents {
        models.push(encode_model_effort(*model)).expect("same count");
    }
    Some(
        w::LlmChoice::new(
            &w::CEILINGS,
            w::LlmChoiceParts { main: encode_model_effort(*main), subagents: models, account: *account },
        )
        .expect(FITS),
    )
}
fn decode_llm(value: Option<w::LlmChoice>) -> Option<jig_domain_tasks::LlmChoice> {
    let w::LlmChoiceParts { main, subagents, account } = value?.into_parts();
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
            d::Stands::Held { answer } => {
                w::Stands::Held(w::Held::new(&w::CEILINGS, w::HeldParts { answer: answer.clone() }).expect(FITS))
            }
        };
        open.push(
            w::Open::new(
                &w::CEILINGS,
                w::OpenParts {
                    asker: question.asker,
                    number: question.number,
                    line: question.line.clone(),
                    words: question.words.clone(),
                    stands,
                },
            )
            .expect(FITS),
        )
        .expect("same count");
    }
    w::Questions::new(
        &w::CEILINGS,
        w::QuestionsParts {
            route: match value.route {
                d::Route::None => w::Route::None,
                d::Route::Up => w::Route::Up,
            },
            taken: value.taken,
            open,
        },
    )
    .expect(FITS)
}

fn decode_questions(value: w::Questions) -> Result<d::Questions, super::Undecodable> {
    let w::QuestionsParts { route, taken, open } = value.into_parts();
    let mut questions = List::with_capacity(open.len());
    for question in open.into_boxed() {
        let w::OpenParts { asker, number, line, words, stands } = question.into_parts();
        let stands = match stands {
            w::Stands::Taken => d::Stands::Taken,
            w::Stands::Retired => return Err(super::Undecodable::Malformed),
            w::Stands::Asked => d::Stands::Asked,
            w::Stands::Held(value) => d::Stands::Held { answer: value.into_parts().answer },
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
    })
}

fn encode_ending(ending: Option<&top::Ending>) -> Option<w::ChangeEnding> {
    Some(match ending? {
        top::Ending::Landed => w::ChangeEnding::Landed,
        top::Ending::NoChange => w::ChangeEnding::NoChange,
        top::Ending::Cancelled => w::ChangeEnding::Cancelled,
        top::Ending::Failed { line } => w::ChangeEnding::Failed(
            w::ChangeFailure::new(&w::CEILINGS, w::ChangeFailureParts { line: line.clone() }).expect(FITS),
        ),
    })
}
fn decode_ending(ending: Option<w::ChangeEnding>) -> Option<top::Ending> {
    Some(match ending? {
        w::ChangeEnding::Landed => top::Ending::Landed,
        w::ChangeEnding::NoChange => top::Ending::NoChange,
        w::ChangeEnding::Cancelled => top::Ending::Cancelled,
        w::ChangeEnding::Failed(line) => top::Ending::Failed { line: line.into_parts().line },
    })
}

pub(super) fn encode_judgement(value: &d::Judgement) -> w::Judgement {
    w::Judgement::new(
        &w::CEILINGS,
        w::JudgementParts {
            landing: value.landing,
            round: value.round,
            pull: value.pull,
            branch: value.branch.clone(),
            base: value.base.clone(),
            prefix: value.prefix,
        },
    )
    .expect(FITS)
}
fn encode_check(check: &top::FollowCheck) -> w::FollowCheckProgress {
    w::FollowCheckProgress::new(
        &w::CEILINGS,
        w::FollowCheckProgressParts {
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
        },
    )
    .expect(FITS)
}
pub(super) fn decode_judgement(value: w::Judgement) -> d::Judgement {
    let w::JudgementParts { landing, round, pull, branch, base, prefix } = value.into_parts();
    d::Judgement { landing, round, pull, branch, base, prefix }
}
fn decode_check(check: w::FollowCheckProgress) -> top::FollowCheck {
    let w::FollowCheckProgressParts { follow, source, head, definition, key, flight, page, faults, held } =
        check.into_parts();
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
