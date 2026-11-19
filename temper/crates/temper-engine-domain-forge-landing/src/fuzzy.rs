//! Seeded snapshots of stacks and their changing rounds (domain/landing.md, 4–8).
use crate::tests::step::{stack, stack_facts};
use crate::{Combine, Decision, Landing, Outcome, Phase, Preparation, Proof, Status};
use skein_lib::{Duration, List, Rng, Wall};

#[test]
fn seeded_stacks_keep_their_prefix_and_turn_invariants() {
    let mut whole_cut = false;
    let mut selected_cut = false;
<<<<<<< HEAD
    let mut stale_after_cut = false;
=======
    let mut unkept = false;
    let mut held = false;
>>>>>>> refs/hand5/9644
    for seed in 0..256 {
        prefix_search(seed);
        let played = run(seed);
        let [whole, selected, absent] = played.2;
        whole_cut |= whole;
        selected_cut |= selected;
<<<<<<< HEAD
        stale_after_cut |= played.3;
        assert_eq!(played, run(seed), "replay seed {seed}");
    }
    assert!(whole_cut && selected_cut, "the sweep reaches both kinds of cut");
    assert!(stale_after_cut, "the sweep moves main after a cut, before the next open is answered");
=======
        unkept |= absent;
        held |= played.0.phase == Phase::Held { was: Box::new(Phase::Preparing), why: crate::Hold::Prepare };
        assert_eq!(played, run(seed), "replay seed {seed}");
    }
    assert!(whole_cut && selected_cut, "the sweep reaches both kinds of cut");
    assert!(unkept && held, "the sweep reaches a prepare with no kept answer and its hold");
>>>>>>> refs/hand5/9644
}

// Each effect takes 1–3 steps, separately from its result becoming readable.
// The settling bound is 32 + 20 per member + 24 per drawn disturbance:
// member steps, round cleanup, and another proof after each disturbance.
<<<<<<< HEAD
fn run(seed: u64) -> (Landing, Box<[Decision]>, [bool; 2], bool) {
=======
fn run(seed: u64) -> (Landing, Box<[Decision]>, [bool; 3]) {
>>>>>>> refs/hand5/9644
    let mut rng = Rng::new(seed);
    let count = usize::try_from(rng.below(6).checked_add(1).unwrap()).unwrap();
    let units = usize::try_from(rng.below(4).checked_add(1).unwrap()).unwrap().min(count);
    let mut landing = stack(count);
    let mut world = Top::new(rng, count, units);
    let mut disturbances = usize::from(world.stale.is_some())
        .checked_add(usize::from(world.cancel))
        .unwrap()
        .checked_add(usize::from(world.red.is_some()))
        .unwrap()
        .checked_add(usize::try_from(world.unkept).unwrap())
        .unwrap();
    for (index, member) in landing.members.iter_mut().enumerate() {
        member.unit = u16::try_from(index.checked_mul(units).unwrap().checked_div(count).unwrap()).unwrap();
        let fresh = &mut world.facts.members[index];
        fresh.judgement = Status::Pending;
        if index > 0 && world.rng.below(4) == 0 {
            fresh.dependencies = Box::new([u16::try_from(index.checked_sub(1).unwrap()).unwrap()]);
        }
        if world.rng.below(4) == 0 {
            fresh.ready = false;
            world.later(0, 0, u16::try_from(index).unwrap(), Effect::Ready);
            disturbances = disturbances.checked_add(1).unwrap();
        }
        disturbances = disturbances.checked_add(usize::from(world.conflicts[index].is_some())).unwrap();
        disturbances = disturbances.checked_add(usize::from(world.noise[index].is_some())).unwrap();
    }
    let bound = u32::try_from(
        32_usize
            .checked_add(20_usize.checked_mul(count).unwrap())
            .unwrap()
            .checked_add(24_usize.checked_mul(disturbances).unwrap())
            .unwrap(),
    )
    .unwrap();
    let mut trace = List::with_capacity(bound);
    let mut probes = 0_u32;
    let mut probe_bound = 0_u32;
    let mut waiting = false;
    let mut due = None;
    let mut asked = Asked { prepare: None, asks: 0, deadline: false };
    for tick in 0..bound {
        let answered = world.answer(&mut landing, tick);
        let disturbed = world.disturb(&landing, tick);
        let now = Wall::from_nanos(u64::from(tick).checked_mul(1_000_000_000).unwrap());
        if waiting && !answered && !disturbed {
            match due {
                Some(at) if now >= at => {}
                Some(_) | None => continue,
            }
        }
        let next = crate::step(&landing, &world.facts, &crate::Project::DEFAULT, &LIMITS, now);
        invariants(seed, &landing, &world.facts, &next, &mut probes, &mut probe_bound);
        asked.decided(seed, &next);
        if let Decision::Wait { until, on } = next.decision {
            world.wait(seed, tick, &next.landing, until, on);
            due = until;
            waiting = true;
        } else {
            waiting = false;
        }
        // The cleanup boundary, not a leave notification, retires the changed suffix.
        if landing.round.ending.is_some() && next.landing.round.ending.is_none() {
            world.cleanup(seed, &landing, &next.landing);
        }
        trace.push(next.decision).unwrap();
        world.start(&next.landing, next.decision, tick);
        match next.decision {
            Decision::Finish => {
                for member in &next.landing.members {
                    assert!(member.landed.is_some() || member.left.is_some(), "terminal seed {seed}");
                }
                if world.pending.is_empty() {
<<<<<<< HEAD
                    return (next.landing, trace.into_boxed(), world.cuts, world.stale_after_cut);
                }
            }
            Decision::Hold(_) | Decision::Cancel => {
                return (next.landing, trace.into_boxed(), world.cuts, world.stale_after_cut);
            }
=======
                    return (next.landing, trace.into_boxed(), world.reached());
                }
            }
            Decision::Hold(_) | Decision::Cancel => return (next.landing, trace.into_boxed(), world.reached()),
>>>>>>> refs/hand5/9644
            Decision::None
            | Decision::Wait { .. }
            | Decision::Prepare { .. }
            | Decision::Key { .. }
            | Decision::Dispatch { .. }
            | Decision::Retry { .. }
            | Decision::Post { .. }
            | Decision::Repair { .. }
            | Decision::Proved
            | Decision::Merge { .. }
            | Decision::Stale
            | Decision::Open
            | Decision::Record { .. }
            | Decision::Landed { .. }
            | Decision::Close
            | Decision::Delete { .. }
            | Decision::Resolve { .. }
            | Decision::Leave { .. }
            | Decision::Judge { .. } => {}
        }
        landing = next.landing;
    }
    panic!("did not settle seed {seed} in {bound} steps: {trace:?}, {:?}", world.pending);
}

// The same prepare, its suffix included, is asked again only after a wait with a
// deadline, and never a third time in a row without a release (landing-preparing.md, 5).
struct Asked {
    prepare: Option<(Decision, Box<[u16]>)>,
    asks: u32,
    deadline: bool,
}
impl Asked {
    fn decided(&mut self, seed: u64, next: &crate::Stepped) {
        match next.decision {
            Decision::Prepare { .. } => {
                let mut suffix = List::with_capacity(6);
                for (index, member) in next.landing.members.iter().enumerate() {
                    if member.left.is_none() && member.landed.is_none() && member.candidate.is_none() {
                        suffix.push(u16::try_from(index).unwrap()).unwrap();
                    }
                }
                let prepare = Some((next.decision, suffix.into_boxed()));
                if prepare == self.prepare {
                    assert!(self.deadline, "the same prepare without a deadline wait seed {seed}");
                    self.asks = self.asks.checked_add(1).unwrap();
                    assert!(self.asks <= 2, "a third same prepare seed {seed}");
                } else {
                    self.prepare = prepare;
                    self.asks = 1;
                }
                self.deadline = false;
            }
            Decision::Wait { until: Some(_), .. } => self.deadline = true,
            Decision::None
            | Decision::Wait { until: None, .. }
            | Decision::Key { .. }
            | Decision::Dispatch { .. }
            | Decision::Retry { .. }
            | Decision::Post { .. }
            | Decision::Repair { .. }
            | Decision::Proved
            | Decision::Merge { .. }
            | Decision::Stale
            | Decision::Finish
            | Decision::Cancel
            | Decision::Hold(_)
            | Decision::Open
            | Decision::Record { .. }
            | Decision::Landed { .. }
            | Decision::Close
            | Decision::Delete { .. }
            | Decision::Resolve { .. }
            | Decision::Leave { .. }
            | Decision::Judge { .. } => {}
        }
    }
}

const LIMITS: crate::Limits =
    crate::Limits { members: 6, gates: 4, poll: Duration::from_secs(5), tests_bytes: 16, write_attempts: 2 };

fn invariants(
    seed: u64,
    landing: &Landing,
    f: &crate::Facts,
    next: &crate::Stepped,
    probes: &mut u32,
    bound: &mut u32,
) {
    if next.landing.round.search != landing.round.search {
        if landing.round.search.is_none() {
            *probes = 0;
            let mut units = skein_lib::Set::with_capacity(6);
            for member in &landing.members {
                if member.left.is_none() && member.landed.is_none() && member.candidate.is_some() {
                    units.insert(member.unit).unwrap();
                }
            }
            *bound = u32::BITS.checked_sub(units.len().saturating_sub(1).leading_zeros()).unwrap();
        }
        if crate::probe(&next.landing).is_some() {
            *probes = probes.checked_add(1).unwrap();
        }
        assert!(*probes <= *bound, "logarithmic search seed {seed}");
    }
    if let Some(search) = &landing.round.search {
        for (index, (old, new)) in landing.members.iter().zip(&next.landing.members).enumerate() {
            if old.landed.is_none() && new.landed.is_some() {
                assert!(index < usize::from(search.red), "red suffix seed {seed}");
            }
            if let Some(crate::Leave::Blamed { .. }) = new.left
                && old.left.is_none()
            {
                let mut first = None;
                for member in landing.members.iter().skip(usize::from(search.green)) {
                    if member.left.is_none() && member.landed.is_none() && member.candidate.is_some() {
                        first = Some(member.unit);
                        break;
                    }
                }
                assert_eq!(Some(new.unit), first, "blame above green seed {seed}");
            }
        }
    }
    if let Decision::Merge { member, candidate } = next.decision {
        let mut last = None;
        for (index, m) in landing.members.iter().enumerate() {
            if m.left.is_none() && m.landed.is_none() && m.candidate.is_some() {
                last = Some(index);
                assert_eq!(f.members[index].judgement, Status::Passed, "judged seed {seed}");
            }
        }
        let last = last.unwrap();
        let top = &landing.members[last];
        assert_eq!(usize::from(member), last, "top seed {seed}");
        assert_eq!(Some(candidate), top.candidate, "prefix seed {seed}");
        for gate in &top.gates {
            match gate.proof {
                Proof::Passed { .. } => {}
                Proof::Unkeyed | Proof::Keyed { .. } | Proof::Dispatched { .. } | Proof::Failed { .. } => {
                    panic!("gates seed {seed}")
                }
            }
        }
    }
}

#[derive(Debug)]
struct Pending {
    due: u32,
    round: u16,
    member: u16,
    effect: Effect,
}

#[derive(Clone, Copy, Debug)]
enum Effect {
    Prepare,
    Prefixes,
    Open,
    Close,
    Delete,
    Post { candidate: crate::Commit },
    Run { candidate: crate::Commit, passed: bool, selected: bool },
    Receipt { candidate: crate::Commit, passed: bool, selected: bool },
    Judge { candidate: crate::Commit },
    Ready,
    Merge { candidate: crate::Commit, base: crate::Commit },
    Readback { candidate: crate::Commit },
    Record,
}

#[derive(Clone, Copy)]
enum StaleAt {
    Proved,
    Merging,
    Cut,
}

struct Top {
    rng: Rng,
    facts: crate::Facts,
    pending: List<Pending>,
    conflicts: Box<[Option<Outcome>]>,
    noise: Box<[Option<(u32, bool)>]>,
    red: Option<u16>,
    by_test: bool,
    stale: Option<StaleAt>,
    cancel: bool,
    cuts: [bool; 2],
<<<<<<< HEAD
    stale_after_cut: bool,
=======
    /// Prepares still to end with no answer their round keeps, read as absent.
    unkept: u32,
    absent: bool,
>>>>>>> refs/hand5/9644
}

impl Top {
    fn new(mut rng: Rng, count: usize, units: usize) -> Self {
        let mut conflicts = List::with_capacity(u32::try_from(count).unwrap());
        let mut noise = List::with_capacity(u32::try_from(count).unwrap());
        for _ in 0..count {
            conflicts
                .push(match rng.below(12) {
                    0 => Some(Outcome::Conflict),
                    1 => Some(Outcome::EarlierConflict),
                    _ => None,
                })
                .unwrap();
            noise
                .push(if rng.below(6) == 0 {
                    Some((u32::try_from(rng.below(12).checked_add(2).unwrap()).unwrap(), rng.below(2) == 0))
                } else {
                    None
                })
                .unwrap();
        }
        let red = if rng.below(2) == 0 {
            Some(u16::try_from(rng.below(u64::try_from(units).unwrap())).unwrap())
        } else {
            None
        };
        let by_test = rng.below(2) == 0;
        let stale = match rng.below(4) {
            0 => Some(StaleAt::Proved),
            1 => Some(StaleAt::Merging),
            2 => Some(StaleAt::Cut),
            _ => None,
        };
        let cancel = rng.below(32) == 0;
        let unkept = match rng.below(8) {
            0 => 1,
            1 => 2,
            _ => 0,
        };
        let mut facts = stack_facts(count);
        facts.turn_free = true;
        facts.preparation = Preparation::Absent;
        Self {
            rng,
            facts,
            pending: List::with_capacity(32),
            conflicts: conflicts.into_boxed(),
            noise: noise.into_boxed(),
            red,
            by_test,
            stale,
            cancel,
            cuts: [false; 2],
<<<<<<< HEAD
            stale_after_cut: false,
=======
            unkept,
            absent: false,
>>>>>>> refs/hand5/9644
        }
    }

    fn reached(&self) -> [bool; 3] {
        let [whole, selected] = self.cuts;
        [whole, selected, self.absent]
    }

    fn merge_pending(&self) -> bool {
        for fresh in &self.facts.members {
            if fresh.merge == Status::Pending {
                return true;
            }
        }
        false
    }

    fn later(&mut self, tick: u32, round: u16, member: u16, effect: Effect) {
        let due = tick.checked_add(u32::try_from(self.rng.below(3).checked_add(1).unwrap()).unwrap()).unwrap();
        self.pending.push(Pending { due, round, member, effect }).unwrap();
    }

    fn disturb(&mut self, landing: &Landing, tick: u32) -> bool {
        let mut disturbed = false;
        if self.cancel && tick >= 8 {
            for fresh in &mut self.facts.members {
                fresh.cancelled = true;
            }
            self.cancel = false;
            disturbed = true;
        }
        for (index, noise) in self.noise.iter_mut().enumerate() {
            if let Some((at, cancel)) = *noise
                && tick >= at
                && !crate::domain::matches_turn(&landing.phase)
            {
                if cancel {
                    self.facts.members[index].cancelled = true;
                } else {
                    self.facts.members[index].head = [90; 32];
                }
                *noise = None;
                disturbed = true;
            }
        }
        if let Some(at) = self.stale {
            let due = match at {
                StaleAt::Proved => proved(landing),
                StaleAt::Merging => self.merge_pending(),
                StaleAt::Cut => {
                    landing.round.ending == Some(crate::RoundEnd::Cut)
                        && match &landing.round.search {
                            Some(search) => search.green != 0,
                            None => false,
                        }
                }
            };
            if due {
                if let StaleAt::Cut = at {
                    self.stale_after_cut = true;
                    assert_ne!(self.facts.opening, Status::Pending, "the cut precedes its next open");
                }
                for fresh in &mut self.facts.members {
                    fresh.main = [91; 32];
                }
                self.stale = None;
                disturbed = true;
            }
        }
        disturbed
    }

    fn answer(&mut self, landing: &mut Landing, tick: u32) -> bool {
        let mut answered = false;
        let pending = core::mem::replace(&mut self.pending, List::with_capacity(32));
        for p in pending.into_boxed() {
            if p.due > tick {
                self.pending.push(p).unwrap();
                continue;
            }
            answered = true;
            let index = usize::from(p.member);
            match p.effect {
                Effect::Prepare if self.unkept > 0 => {
                    // It ended with no answer the round keeps (landing-preparing.md, 2).
                    self.unkept = self.unkept.checked_sub(1).unwrap();
                    self.facts.preparation = Preparation::Absent;
                    self.absent = true;
                }
                Effect::Prepare => {
                    self.facts.preparation = Preparation::Idle;
                    self.later(tick, p.round, p.member, Effect::Prefixes);
                }
                Effect::Prefixes => {
                    for (index, member) in landing.members.iter().enumerate().skip(index) {
                        if member.left.is_none() && member.landed.is_none() && member.candidate.is_none() {
                            if let Some(outcome) = self.conflicts[index].take() {
                                self.facts.members[index].outcome = Some(outcome);
                            } else {
                                // A changed round has another tree; kept prefixes keep their receipts.
                                let mut candidate = crate::tests::step::prefix(index);
                                candidate[1] = u8::try_from(p.round).unwrap();
                                self.facts.members[index].candidate = Some((
                                    candidate,
                                    if self.rng.below(2) == 0 { Combine::Merge } else { Combine::Squash },
                                ));
                            }
                        }
                    }
                }
                Effect::Open => {
                    self.facts.opened = Some(u64::from(p.round).checked_add(99).unwrap());
                    self.facts.opening = Status::Passed;
                }
                Effect::Close => self.facts.closed = Status::Passed,
                Effect::Delete => self.facts.members[index].deleted = Status::Passed,
                Effect::Post { candidate } => {
                    if fresh_candidate(&self.facts, index) == Some(candidate) {
                        self.facts.members[index].gates[0].posting = Status::Passed;
                    }
                }
                Effect::Run { candidate, passed, selected } => {
                    if fresh_candidate(&self.facts, index) == Some(candidate) {
                        self.facts.members[index].gates[0].flight = crate::Flight::Mine {
                            run: Some(if passed { crate::Verdict::Passed } else { crate::Verdict::Failed }),
                        };
                    }
                    self.later(tick, p.round, p.member, Effect::Receipt { candidate, passed, selected });
                }
                Effect::Receipt { candidate, passed, selected } => {
                    if fresh_candidate(&self.facts, index) == Some(candidate) {
                        let fresh = &mut self.facts.members[index];
                        fresh.gates[0].flight = crate::Flight::None;
                        if selected && passed {
                            // A selected pass is a probe outcome, never a whole-key receipt.
                            if landing.round.search.is_some() {
                                keep_probe(landing, p.member, true);
                            }
                        } else {
                            fresh.gates[0].receipt = if passed { Status::Passed } else { Status::Failed };
                            fresh.failed[0] = if !passed && self.by_test {
                                Some(crate::tests::search_tests::ids(&[b"b"]))
                            } else {
                                None
                            };
                        }
                    }
                }
                Effect::Judge { candidate } => {
                    if fresh_candidate(&self.facts, index) == Some(candidate) {
                        self.facts.members[index].judgement = Status::Passed;
                    }
                }
                Effect::Ready => self.facts.members[index].ready = true,
                Effect::Merge { candidate, base } => {
                    if self.facts.members[index].main == base {
                        self.facts.members[index].merge = Status::Passed;
                        self.later(tick, p.round, p.member, Effect::Readback { candidate });
                    } else {
                        self.facts.members[index].merge = Status::Failed;
                    }
                }
                Effect::Readback { candidate } => {
                    for (fresh, member) in self.facts.members.iter_mut().zip(&landing.members) {
                        fresh.main = candidate;
                        if member.combined == Some(Combine::Merge) {
                            fresh.merged = member.candidate;
                        }
                    }
                }
                Effect::Record => self.facts.members[index].record = Status::Passed,
            }
        }
        answered
    }

    fn cleanup(&mut self, seed: u64, before: &Landing, after: &Landing) {
        let first = if before.round.ending == Some(crate::RoundEnd::Stale) {
            0
        } else {
            usize::from(before.round.changed.unwrap())
        };
        for (index, (old, new)) in before.members.iter().zip(&after.members).enumerate() {
            if index < first {
                // By-test cuts reset even kept proofs; a kept prefix itself does not change.
                assert_eq!(old.candidate, new.candidate, "kept prefix seed {seed}");
                if !by_test(before) {
                    assert_eq!(old, new, "kept below leave seed {seed}");
                }
            } else {
                assert_eq!(new.candidate, None, "cleared suffix seed {seed}");
                let fresh = &mut self.facts.members[index];
                fresh.candidate = None;
                fresh.deleted = Status::Unknown;
                fresh.outcome = None;
                fresh.judgement = Status::Pending;
                fresh.merge = Status::Unknown;
                fresh.gates[0].key = None;
                fresh.gates[0].receipt = Status::Unknown;
                fresh.gates[0].flight = crate::Flight::None;
                fresh.gates[0].posting = Status::Unknown;
                fresh.failed[0] = None;
            }
        }
        self.facts.closed = Status::Unknown;
        self.facts.opened = None;
        self.facts.opening = Status::Unknown;
        self.facts.preparation = Preparation::Absent;
    }

    fn start(&mut self, landing: &Landing, decision: Decision, tick: u32) {
        let round = landing.preparation;
        if landing.round.ending == Some(crate::RoundEnd::Cut) {
            self.cuts[usize::from(by_test(landing))] = true;
        }
        match decision {
            Decision::Prepare { member, .. } => {
                self.facts.preparation = Preparation::Running;
                self.later(tick, round, member, Effect::Prepare);
            }
            Decision::Open => {
                self.facts.opening = Status::Pending;
                self.later(tick, round, 0, Effect::Open);
            }
            Decision::Close => {
                self.facts.closed = Status::Pending;
                self.later(tick, round, 0, Effect::Close);
            }
            Decision::Delete { member } => {
                self.facts.members[usize::from(member)].deleted = Status::Pending;
                self.later(tick, round, member, Effect::Delete);
            }
            Decision::Judge { member } => {
                let candidate = landing.members[usize::from(member)].candidate.unwrap();
                self.later(tick, round, member, Effect::Judge { candidate });
            }
            Decision::Key { member, candidate, .. } => {
                self.facts.members[usize::from(member)].gates[0].key = Some(candidate);
            }
            Decision::Post { member, .. } => {
                self.facts.members[usize::from(member)].gates[0].posting = Status::Pending;
                let candidate = landing.members[usize::from(member)].candidate.unwrap();
                self.later(tick, round, member, Effect::Post { candidate });
            }
            Decision::Dispatch { member, .. } | Decision::Retry { member, .. } => {
                let candidate = landing.members[usize::from(member)].candidate.unwrap();
                let mut passed = true;
                for m in landing.members.iter().take(usize::from(member).checked_add(1).unwrap()) {
                    if m.left.is_none() && m.landed.is_none() && Some(m.unit) == self.red {
                        passed = false;
                    }
                }
                let selected = by_test(landing);
                self.facts.members[usize::from(member)].gates[0].flight =
                    crate::Flight::Mine { run: Some(crate::Verdict::Running) };
                self.later(tick, round, member, Effect::Run { candidate, passed, selected });
            }
            Decision::Merge { member, candidate } => {
                let base = landing.members[usize::from(member)].base.unwrap();
                self.facts.members[usize::from(member)].merge = Status::Pending;
                self.later(tick, round, member, Effect::Merge { candidate, base });
            }
            Decision::Record { member, .. } => {
                self.facts.members[usize::from(member)].record = Status::Pending;
                self.later(tick, round, member, Effect::Record);
            }
            Decision::Leave { member, .. } | Decision::Repair { member, .. } | Decision::Resolve { member, .. } => {
                self.facts.members[usize::from(member)].ready = false;
                self.later(tick, round, member, Effect::Ready);
            }
            Decision::Landed { member, .. } => self.facts.members[usize::from(member)].ready = false,
            Decision::None
            | Decision::Wait { .. }
            | Decision::Proved
            | Decision::Stale
            | Decision::Finish
            | Decision::Cancel
            | Decision::Hold(_) => {}
        }
    }

    fn wait(&self, seed: u64, tick: u32, landing: &Landing, until: Option<Wall>, on: crate::Awaited) {
        use crate::Awaited;
        assert_ne!(on, Awaited::Read, "the policy never waits on the top's read seed {seed}");
        let deadline_required = match on {
            Awaited::Prefixes | Awaited::Run { .. } | Awaited::Receipt { .. } | Awaited::Ancestry { .. } => true,
            Awaited::Release
            | Awaited::Ready { .. }
            | Awaited::Turn
            | Awaited::Prepare
            | Awaited::Open
            | Awaited::Close
            | Awaited::Delete { .. }
            | Awaited::Post { .. }
            | Awaited::Judgement { .. }
            | Awaited::Merge
            | Awaited::Record { .. }
            | Awaited::Read => false,
        };
        if deadline_required {
            assert!(until.is_some(), "silent fact wait seed {seed} tick {tick}: {on:?}");
        }
        if let Some(until) = until {
            assert!(
                until.as_nanos() <= u64::from(tick).checked_add(5).unwrap().checked_mul(1_000_000_000).unwrap(),
                "poll bound seed {seed}"
            );
            return;
        }
        let mut wake = (on == Awaited::Release && held(landing)) || (on == Awaited::Turn && !self.facts.turn_free);
        for pending in &self.pending {
            let member = pending.member;
            let awaiting = match pending.effect {
                Effect::Prepare => Some(Awaited::Prepare),
                Effect::Open => Some(Awaited::Open),
                Effect::Close => Some(Awaited::Close),
                Effect::Merge { .. } => Some(Awaited::Merge),
                Effect::Delete => Some(Awaited::Delete { member }),
                Effect::Record => Some(Awaited::Record { member }),
                Effect::Ready => Some(Awaited::Ready { member }),
                Effect::Judge { .. } => Some(Awaited::Judgement { member }),
                Effect::Post { .. } => Some(Awaited::Post { member, gate: 0 }),
                Effect::Prefixes | Effect::Run { .. } | Effect::Receipt { .. } | Effect::Readback { .. } => None,
            };
            wake |= awaiting == Some(on);
        }
        assert!(wake, "no wake seed {seed} tick {tick}: {on:?}, {:?}", self.pending);
    }
}

#[expect(clippy::manual_map, reason = "the strict subset uses no closures")]
fn fresh_candidate(facts: &crate::Facts, index: usize) -> Option<crate::Commit> {
    match facts.members[index].candidate {
        Some((candidate, _)) => Some(candidate),
        None => None,
    }
}
fn by_test(landing: &Landing) -> bool {
    match &landing.round.search {
        Some(search) => match search.tests {
            crate::SearchTests::ByTest { .. } => true,
            crate::SearchTests::Whole | crate::SearchTests::WholeAgain => false,
        },
        None => false,
    }
}
fn proved(landing: &Landing) -> bool {
    match landing.phase {
        Phase::Proved { .. } => true,
        Phase::Waiting
        | Phase::Preparing
        | Phase::Proving
        | Phase::Turn { .. }
        | Phase::Landed
        | Phase::Held { .. }
        | Phase::Cancelled => false,
    }
}
fn held(landing: &Landing) -> bool {
    match landing.phase {
        Phase::Held { .. } => true,
        Phase::Waiting
        | Phase::Preparing
        | Phase::Proving
        | Phase::Proved { .. }
        | Phase::Turn { .. }
        | Phase::Landed
        | Phase::Cancelled => false,
    }
}

fn prefix_search(seed: u64) {
    let mut rng = Rng::new(seed);
    let n = rng.below(4).checked_add(2).unwrap();
    let k = rng.below(n).checked_add(1).unwrap();
    let mut receipts = List::with_capacity(5);
    for unit in 1..=n {
        receipts.push(if unit < k { Status::Passed } else { Status::Failed }).unwrap();
    }
    let (landing, _, probes) = crate::tests::step::search_receipts(receipts.as_slice());
    let bound = u64::BITS.checked_sub(n.checked_sub(1).unwrap().leading_zeros()).unwrap();
    assert!(probes.len() <= usize::try_from(bound).unwrap(), "logarithmic prefix search seed {seed}");
    assert_eq!(
        landing.round.search,
        Some(crate::Search {
            tests: crate::SearchTests::Whole,
            green: u16::try_from(k.checked_sub(1).unwrap()).unwrap(),
            red: u16::try_from(k).unwrap()
        })
    );
}

#[test]
fn seeded_searches_by_test_narrow_end_unlanded_and_bound_both_searches() {
    for seed in 0..256 {
        prefix_search_tests(seed);
    }
}

fn keep_probe(landing: &mut Landing, member: u16, passed: bool) {
    let crate::SearchTests::ByTest { probed, .. } = &mut landing.round.search.as_mut().unwrap().tests else {
        panic!("selected run")
    };
    let mut outcomes = List::with_capacity(u32::try_from(probed.len()).unwrap().checked_add(1).unwrap());
    for outcome in probed.iter() {
        outcomes.push(*outcome).unwrap();
    }
    outcomes.push(crate::Probed { member: member.checked_add(1).unwrap(), gate: 0, run: 84, passed }).unwrap();
    *probed = outcomes.into_boxed();
}

#[expect(clippy::too_many_lines, reason = "seeded selected and fallback probes share one bounded replay loop")]
fn prefix_search_tests(seed: u64) {
    use crate::tests::search_tests::{decide, ids, top_failed};
    let mut rng = Rng::new(seed);
    let count = usize::try_from(rng.below(4).checked_add(2).unwrap()).unwrap();
    let (landing, mut facts) = top_failed(count);
    let mut landing = decide(&landing, &facts).landing;
    let bound =
        usize::try_from(usize::BITS.checked_sub(count.checked_sub(1).unwrap().leading_zeros()).unwrap()).unwrap();
    let mut selected_probes = List::with_capacity(5);
    let mut whole_probes = List::with_capacity(5);
    for _tick in 0_u32..100 {
        let by_test = match &landing.round.search.as_ref().unwrap().tests {
            crate::SearchTests::ByTest { .. } => true,
            crate::SearchTests::Whole | crate::SearchTests::WholeAgain => false,
        };
        if let Some(probe) = crate::probe(&landing) {
            let probes = if by_test { &mut selected_probes } else { &mut whole_probes };
            if !probes.as_slice().contains(&probe) {
                probes.push(probe).unwrap();
                if by_test {
                    match rng.below(5) {
                        0 => facts.members[usize::from(probe)].gates[0].receipt = Status::Passed,
                        1 => {
                            facts.members[usize::from(probe)].gates[0].receipt = Status::Failed;
                            facts.members[usize::from(probe)].failed[0] = Some(ids(&[b"b", b"c"]));
                        }
                        2 => keep_probe(&mut landing, probe, true),
                        3 => keep_probe(&mut landing, probe, false),
                        4 => {}
                        _ => unreachable!("five outcomes"),
                    }
                }
            }
            assert!(selected_probes.len() <= u32::try_from(bound).unwrap(), "selected probe budget seed {seed}");
            assert!(whole_probes.len() <= u32::try_from(bound).unwrap(), "whole probe budget seed {seed}");
        }
        let next = decide(&landing, &facts);
        if by_test {
            assert_ne!(next.decision, Decision::Proved, "selected prefix seed {seed}");
            assert!(!crate::domain::matches_turn(&next.landing.phase), "selected turn seed {seed}");
            for member in &next.landing.members {
                assert!(member.landed.is_none(), "selected prefix landed seed {seed}");
            }
            let crate::SearchTests::ByTest { gates: before, .. } = &landing.round.search.as_ref().unwrap().tests else {
                panic!("by test")
            };
            match &next.landing.round.search.as_ref().unwrap().tests {
                crate::SearchTests::ByTest { gates: after, .. } => {
                    for (before, after) in before.iter().zip(after) {
                        match (before, after) {
                            (Some(before), Some(after)) => {
                                for id in after {
                                    assert!(before.contains(id), "selection widened seed {seed}");
                                }
                            }
                            (None, None) => {}
                            (None, Some(_)) | (Some(_), None) => panic!("selection changed gates seed {seed}"),
                        }
                    }
                }
                crate::SearchTests::WholeAgain => {
                    let search = next.landing.round.search.as_ref().unwrap();
                    assert_eq!((search.green, usize::from(search.red)), (0, count));
                }
                crate::SearchTests::Whole => panic!("fallback has its own identity seed {seed}"),
            }
        }
        match next.decision {
            Decision::Dispatch { member, gate, .. } => {
                let fresh = &mut facts.members[usize::from(member)];
                if by_test {
                    if rng.below(3) == 0 {
                        fresh.gates[usize::from(gate)].receipt = Status::Failed;
                        fresh.failed[usize::from(gate)] = Some(ids(&[b"b", b"c"]));
                    } else {
                        landing = next.landing;
                        keep_probe(&mut landing, member, rng.below(2) == 0);
                        continue;
                    }
                } else {
                    fresh.gates[usize::from(gate)].receipt =
                        if rng.below(2) == 0 { Status::Passed } else { Status::Failed };
                }
            }
            Decision::Leave { .. } => {
                assert_eq!(next.landing.round.ending, Some(crate::RoundEnd::Cut));
                if by_test {
                    assert!(next.landing.searched.is_some());
                }
                return;
            }
            Decision::Post { member, gate, .. } => {
                facts.members[usize::from(member)].gates[usize::from(gate)].posting = Status::Passed;
            }
            Decision::None | Decision::Wait { .. } => {}
            Decision::Prepare { .. }
            | Decision::Key { .. }
            | Decision::Retry { .. }
            | Decision::Repair { .. }
            | Decision::Proved
            | Decision::Merge { .. }
            | Decision::Stale
            | Decision::Finish
            | Decision::Cancel
            | Decision::Hold(_)
            | Decision::Open
            | Decision::Record { .. }
            | Decision::Landed { .. }
            | Decision::Close
            | Decision::Delete { .. }
            | Decision::Resolve { .. }
            | Decision::Judge { .. } => panic!("unexpected {:?} seed {seed}", next.decision),
        }
        landing = next.landing;
    }
    panic!("search did not settle seed {seed}");
}
