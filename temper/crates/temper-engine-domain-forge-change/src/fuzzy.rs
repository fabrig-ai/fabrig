//! The release property (temper/docs/design/domain/forge/change/repairs-releases.md, 6).
use crate::boundary::Commit;
use crate::tests::{BASE, LIMITS, at, change, facts, heard};
use crate::{
    AuthorityHold, AuthorityLack, AuthorityRecovery, Cause, Change, Ci, CloseAt, CloseRefused, Decision, Delegate,
    Effect, EffectResult, Facts, Freshness, Gate, GateKind, GateReport, Heard, Hold, Kind, Pull, Recheck,
    ReleaseTrigger, Repair, State, Status, Stepped, Way, release, step,
};
use skein_lib::{List, Rng};

const SEEDS: core::ops::Range<u64> = 0..256;
const STEPS: u32 = 32;
const GATE: u64 = 1;
const AUTHORITY: Cause = Cause::Authority(AuthorityHold {
    recovery: AuthorityRecovery { task: 7, spec: 0, hold: 1, tasks: 2 },
    lack: AuthorityLack::Tasks,
    had: 2,
    released: false,
});

#[derive(Clone, Copy, Debug)]
enum Phase {
    Taking,
    Producing,
    Opening,
    Reopening,
    Checking,
    Gating,
    Ready,
    Queued,
    First,
    Updating,
    WaitingProducer,
    Resolving,
    Repairing,
    Landing,
}

// A cause belongs to the predecessor which can write that hold, not necessarily
// to the step that will re-enter it. Stalls in particular need not be CI stalls.
const CASES: &[(Phase, Cause)] = &[
    (Phase::Taking, Cause::Stalled),
    (Phase::Taking, Cause::Failed),
    (Phase::Producing, Cause::Stalled),
    (Phase::Producing, Cause::Failed),
    (Phase::Producing, Cause::CloseRefused),
    (Phase::Producing, AUTHORITY),
    (Phase::Producing, Cause::Questions),
    (Phase::Opening, Cause::Stalled),
    (Phase::Opening, Cause::Failed),
    (Phase::Opening, Cause::BranchMoved),
    (Phase::Reopening, Cause::Stalled),
    (Phase::Reopening, Cause::Failed),
    (Phase::Checking, Cause::Stalled),
    (Phase::Checking, Cause::Runner),
    (Phase::Checking, Cause::Repairs),
    (Phase::Checking, AUTHORITY),
    (Phase::Gating, Cause::Rejected),
    (Phase::Gating, Cause::Repairs),
    (Phase::Ready, Cause::BranchMissing),
    (Phase::Queued, Cause::PullClosed),
    (Phase::First, Cause::Retargeted),
    (Phase::First, Cause::Resolutions),
    (Phase::First, Cause::Updates),
    (Phase::Updating, Cause::Stalled),
    (Phase::Updating, Cause::Failed),
    (Phase::WaitingProducer, Cause::BranchMoved),
    (Phase::WaitingProducer, Cause::Resolutions),
    (Phase::Resolving, Cause::Stalled),
    (Phase::Resolving, Cause::Failed),
    (Phase::Repairing, Cause::Stalled),
    (Phase::Repairing, Cause::Failed),
    (Phase::Repairing, Cause::RedStands),
    (Phase::Landing, Cause::Stalled),
    (Phase::Landing, Cause::Failed),
];

impl Phase {
    fn state(self, cause: Cause, head: Commit) -> State {
        let ready_since = at(1);
        match self {
            Self::Taking => State::Taking { requested: true },
            Self::Producing => State::Producing { requested: !matches!(cause, Cause::Authority(_)) },
            Self::Opening => State::Opening { requested: cause != Cause::BranchMoved },
            Self::Reopening => State::Reopening { requested: true, retarget: false },
            Self::Checking => State::Checking { head },
            Self::Gating => State::Gating { head, asked: Box::new([]) },
            Self::Ready => State::Ready { head, ready_since },
            Self::Queued => State::Queued { head, ready_since },
            Self::First => State::First { head, ready_since },
            Self::Updating => State::Updating { from: head, base: BASE, ready_since },
            Self::WaitingProducer => State::WaitingProducer { base: BASE },
            Self::Resolving => State::Resolving { requested: true, base: BASE },
            Self::Repairing => State::Repairing { requested: cause != Cause::RedStands, why: Repair::Ci },
            Self::Landing => State::Landing { head },
        }
    }
}

// The forge answers admitted writes and delegates. It doesn't invent a push or
// a merge merely because the policy waits for one. The unchanged person gate
// and close refusal are judgements, not facts this forge gets to revise.
struct Forge {
    facts: Facts,
    heard: Heard,
    next_head: u8,
    gate_kind: GateKind,
    pending: Decision,
}

impl Forge {
    fn new(phase: Phase, cause: Cause, head: Commit) -> Self {
        let gate_kind = if cause == Cause::Rejected { GateKind::Person } else { GateKind::Check };
        let mut forge = Self { facts: facts(), heard: heard(), next_head: 16, gate_kind, pending: Decision::None };
        forge.facts.branch = Some(head);
        forge.facts.pull = Pull::Open { head, base: BASE };
        forge.facts.contains_head = Status::Failed;
        match phase {
            Phase::Taking | Phase::Producing => {
                forge.facts.branch = None;
                forge.facts.pull = Pull::Missing;
            }
            Phase::Opening => forge.facts.pull = Pull::Missing,
            Phase::Reopening => forge.facts.pull = Pull::Closed,
            Phase::Checking
            | Phase::Gating
            | Phase::Ready
            | Phase::Queued
            | Phase::First
            | Phase::Updating
            | Phase::WaitingProducer
            | Phase::Resolving
            | Phase::Repairing
            | Phase::Landing => {}
        }
        forge.facts.taken = matches!(phase, Phase::Taking).then_some(head);
        forge.recover(phase, cause, head);
        forge.observe();
        forge
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "only selected held cases carry an outstanding request or a surviving refusal"
    )]
    fn recover(&mut self, phase: Phase, cause: Cause, head: Commit) {
        let pending = match (phase, cause) {
            (Phase::Producing, Cause::Questions | Cause::Stalled) => Decision::Delegate(Delegate::Produce),
            (Phase::Opening, Cause::Stalled) => Decision::Effect(Effect::Open),
            (Phase::Reopening, Cause::Stalled) => Decision::Effect(Effect::Reopen),
            (Phase::Updating, Cause::Stalled) => Decision::Effect(Effect::Update { head, base: BASE }),
            (Phase::Resolving, Cause::Stalled) => Decision::Delegate(Delegate::Resolve { base: BASE }),
            (Phase::Repairing, Cause::Stalled) => Decision::Delegate(Delegate::Repair(Repair::Ci)),
            (Phase::Landing, Cause::Stalled) => Decision::Effect(Effect::Merge { head, base: BASE }),
            _ => Decision::None,
        };
        match pending {
            Decision::Effect(_) => self.heard.effect = EffectResult::Pending,
            Decision::Delegate(delegate) => {
                self.facts.delegate = Some(delegate);
                self.heard.delegate_live = true;
                self.heard.delegate_running = true;
            }
            _ => {}
        }
        self.pending = pending;
        match (phase, cause) {
            (_, Cause::Authority(why)) => self.heard.lacking = Some(why),
            (_, Cause::BranchMoved) => self.push(self.next_commit()),
            (_, Cause::BranchMissing) => self.facts.branch = None,
            (_, Cause::PullClosed) => self.facts.pull = Pull::Closed,
            (_, Cause::Retargeted) => self.facts.pull = Pull::Open { head, base: [5; 32] },
            (Phase::Opening | Phase::Reopening | Phase::Landing, Cause::Failed) => {
                self.heard.effect = EffectResult::Failed;
            }
            (Phase::Producing, Cause::CloseRefused) => {
                let tips = CloseAt { branch: head, landing: BASE, asked: true, refused: None, outdated: None };
                self.facts.close = true;
                self.facts.close_tips = Some(tips);
                self.facts.close_at = Some(CloseAt { refused: Some(CloseRefused { counts: None }), ..tips });
            }
            (Phase::Repairing, Cause::RedStands) => {
                self.facts.repair_head = Some(head);
                self.facts.recheck = Recheck::Taken;
            }
            _ => {}
        }
    }

    fn next_commit(&self) -> Commit {
        [self.next_head; 32]
    }

    fn push(&mut self, head: Commit) {
        self.next_head = self.next_head.saturating_add(1);
        self.facts.branch = Some(head);
        if let Pull::Open { base, .. } = self.facts.pull {
            self.facts.pull = Pull::Open { head, base };
        }
    }

    fn observe(&mut self) {
        let Some(head) = self.facts.branch else { return };
        self.facts.ci = Ci { head, status: Status::Passed };
        let status = if self.gate_kind == GateKind::Person { Status::Failed } else { Status::Passed };
        self.facts.gates = Box::new([GateReport { number: GATE, head, status, revision: self.facts.revision }]);
    }

    fn answer(&mut self, decision: Decision) {
        match decision {
            Decision::ReadMerge { .. } => unreachable!("this forge has merge jobs disabled"),
            Decision::Effect(effect) => {
                let head = self.facts.branch;
                match effect {
                    Effect::CreateBranch { head } => self.push(head),
                    Effect::Open | Effect::Reopen | Effect::Retarget => {
                        self.facts.pull = Pull::Open { head: head.expect("pull has a branch"), base: BASE };
                    }
                    Effect::Prepare { .. } | Effect::Update { .. } => self.push(self.next_commit()),
                    Effect::Merge { .. } => self.facts.pull = Pull::Merged { commit: self.next_commit() },
                    Effect::Rerun { .. } | Effect::FollowCheck { .. } => {}
                }
                self.heard.effect = EffectResult::Made;
            }
            Decision::Delegate(delegate) => {
                match delegate {
                    Delegate::Gate { .. } => {}
                    Delegate::Produce
                    | Delegate::Close
                    | Delegate::Repair(_)
                    | Delegate::Resolve { .. }
                    | Delegate::Merge { .. } => self.push(self.next_commit()),
                }
                self.facts.delegate = Some(delegate);
                self.heard.delegate = Status::Passed;
                self.heard.delegate_running = false;
                self.heard.delegate_live = false;
            }
            Decision::CancelGates { .. }
            | Decision::CancelProducer
            | Decision::CancelRepair { .. }
            | Decision::CancelMovedDelegate { .. } => {
                self.facts.delegate = None;
                self.heard.delegate = Status::Unknown;
                self.heard.delegate_running = false;
                self.heard.delegate_live = false;
            }
            Decision::None
            | Decision::Wait { .. }
            | Decision::RepairEnded { .. }
            | Decision::EndHold
            | Decision::Ready
            | Decision::Finish { .. }
            | Decision::NoChange
            | Decision::Cancel
            | Decision::Hold(_)
            | Decision::ReplayJudgement(_)
            | Decision::Failed
            | Decision::QueueRepair => {}
        }
        self.observe();
    }
}

fn count(rng: &mut Rng, spent: bool, limit: u32) -> u32 {
    if spent { limit } else { u32::try_from(rng.between(1, u64::from(limit))).unwrap() }
}

fn seeded(seed: u64) -> (usize, Change, Forge, bool) {
    let mut rng = Rng::new(seed);
    let index = usize::try_from(rng.below(u64::try_from(CASES.len()).unwrap())).unwrap();
    let (phase, cause) = CASES[index];
    let head = [u8::try_from(rng.between(64, 127)).unwrap(); 32];
    let state = phase.state(cause, head);
    let mut row = change(State::Held { was: Box::new(state), hold: Hold::new(cause, at(31)) });
    row.since = at(1);
    row.last_head = (!matches!(phase, Phase::Taking | Phase::Producing)).then_some(head);
    if matches!(phase, Phase::Ready | Phase::Queued | Phase::First | Phase::Updating | Phase::Landing) {
        row.ready_since = Some(at(1));
    }
    row.way = if matches!(phase, Phase::Ready) { Way::Turn } else { Way::Queue };
    if !matches!(phase, Phase::Taking) {
        row.repairs = count(&mut rng, cause == Cause::Repairs, LIMITS.repairs);
        row.resolutions = count(&mut rng, cause == Cause::Resolutions, LIMITS.resolutions);
        row.updates = count(&mut rng, cause == Cause::Updates, LIMITS.updates);
    }
    let mut forge = Forge::new(phase, cause, head);
    let bare_authority = matches!(cause, Cause::Authority(_)) && rng.below(2) == 0;
    if let Cause::Authority(hold) = cause {
        if bare_authority {
            // The original delegate must still be needed. At checking, a green
            // read would naturally remove the need to fund a repair.
            if matches!(phase, Phase::Checking) {
                forge.facts.ci.status = Status::Failed;
            }
        } else {
            forge.facts.revision = hold.recovery.spec.saturating_add(1);
            forge.heard.lacking = None;
            forge.observe();
        }
    }
    row.gates = Box::new([Gate {
        number: GATE,
        kind: forge.gate_kind,
        blocking: true,
        freshness: Freshness::Exact,
        eager: false,
    }]);
    (index, row, forge, bare_authority)
}

fn counts(row: &Change) -> [u32; 3] {
    [row.repairs, row.resolutions, row.updates]
}

#[derive(Debug, PartialEq, Eq)]
struct Walk {
    index: usize,
    bare_authority: bool,
    initial: Change,
    trace: Box<[Stepped]>,
}

fn walk(seed: u64) -> Walk {
    let (index, initial, mut forge, bare_authority) = seeded(seed);
    let trigger = ReleaseTrigger::Task { owes_readback: false, close: forge.facts.close };
    let Some(mut next) = release(&initial, trigger, Some(&forge.facts), Some(&forge.heard), at(32)) else {
        panic!("the held change must accept one release: seed {seed}");
    };
    // The connector retires a definite failed merge after the release consumes
    // its readback. Pending merges keep their outcome for the policy to settle.
    if matches!(CASES[index], (Phase::Landing, Cause::Failed)) && !matches!(next.change.state, State::Held { .. }) {
        forge.heard.effect = EffectResult::None;
    }
    let mut trace = List::with_capacity(STEPS.saturating_add(1));
    for tick in 0..STEPS {
        let decision = next.decision;
        let settled = settled(&next.change);
        trace.push(next).unwrap();
        if settled {
            return Walk { index, bare_authority, initial, trace: trace.into_boxed() };
        }
        let outstanding = core::mem::replace(&mut forge.pending, Decision::None);
        forge.answer(outstanding);
        forge.answer(decision);
        let row = &trace.last().unwrap().change;
        next = step(row, &forge.facts, &forge.heard, at(33_u64.saturating_add(u64::from(tick))), &LIMITS);
    }
    trace.push(next).unwrap();
    Walk { index, bare_authority, initial, trace: trace.into_boxed() }
}

fn settled(change: &Change) -> bool {
    change.released.is_none()
        && matches!(
            change.state,
            State::Ready { .. }
                | State::Queued { .. }
                | State::First { .. }
                | State::Landed { .. }
                | State::Cleared { .. }
                | State::Uncleared { .. }
                | State::NoChange { .. }
                | State::Ended
                | State::Held { .. }
        )
}

#[test]
fn released_changes_converge_when_the_forge_agrees() {
    let mut seen = [false; CASES.len()];
    let mut bare_seen = [false; CASES.len()];
    let mut reported = [false; CASES.len()];
    let mut failures = List::with_capacity(u32::try_from(CASES.len()).unwrap());
    for seed in SEEDS {
        let played = walk(seed);
        assert_eq!(played, walk(seed), "replay seed {seed}");
        let index = played.index;
        let (phase, cause) = CASES[index];
        if played.bare_authority {
            bare_seen[index] = true;
        } else {
            seen[index] = true;
        }
        let mut expected = counts(&played.initial);
        #[expect(clippy::wildcard_enum_match_arm, reason = "only these three causes reset policy-owned counts")]
        match cause {
            Cause::Repairs => expected[0] = 0,
            Cause::Resolutions => expected[1] = 0,
            Cause::Updates => expected[2] = 0,
            _ => {}
        }
        let first = played.trace.first().unwrap();
        assert_eq!(counts(&first.change), expected, "allowances: seed {seed}, {phase:?}/{cause:?}");
        if played.bare_authority
            && let Cause::Authority(why) = cause
        {
            let reheld = Cause::Authority(AuthorityHold { released: true, ..why });
            assert!(
                matches!(&first.change.state, State::Held { hold, .. } if hold.cause == reheld)
                    && first.decision == Decision::Hold(reheld)
                    && first.change.released.is_none(),
                "a bare authority release must hold again at once: seed {seed}, {phase:?}, {first:?}"
            );
            continue;
        }
        let last = played.trace.last().unwrap();
        let converged = if let State::Held { hold, .. } = &last.change.state {
            cause.kind() == Kind::Judgement && hold.cause == cause && settled(&last.change)
        } else {
            settled(&last.change)
        };
        if !converged && !reported[index] {
            reported[index] = true;
            // Keep every transition but collapse the repeated observations of one wait.
            let mut trace = List::with_capacity(STEPS.saturating_add(1));
            for entry in &played.trace {
                let item = (entry.change.state.clone(), entry.decision);
                if trace.last() != Some(&item) {
                    trace.push(item).unwrap();
                }
            }
            failures.push((seed, phase, cause, played.trace.len(), trace.into_boxed())).unwrap();
        }
    }
    for (index, case) in CASES.iter().enumerate() {
        assert!(seen[index], "the pinned seeds must exercise {case:?}");
        if matches!(case.1, Cause::Authority(_)) {
            assert!(bare_seen[index], "the pinned seeds must also exercise a bare release for {case:?}");
        }
    }
    assert!(
        failures.is_empty(),
        "release did not converge (seed, phase, cause, trace length, transitions):\n{:?}",
        failures.as_slice()
    );
}
