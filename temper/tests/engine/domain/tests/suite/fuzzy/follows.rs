//! Seeded, sharded whole-engine follows (forge/follows.md, 11).
//! Each shard supplies its own cases (`temper/docs/design/testing/workloads-sweeps.md`, 1.5).
use crate::forge_routes::follow_changes::{proof, role};
use jig_domain_tasks as tasks;
use skein_forge::api::Commit;
use skein_lib::{Rng, StateHasher};
use skein_world::sweep::{Test, cases};
use std::collections::BTreeSet;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use temper_engine_domain::{Key, Record};
use temper_engine_domain_forge::{self as forge, FollowState, follows::FollowAct};
use temper_engine_domain_world::{
    forge_routes::World,
    replay::{same_digest, same_text},
};
#[path = "follows/referee.rs"]
pub(super) mod referee;
use referee::Referee;

const SEEDS: u64 = 16;
const STEPS: u32 = 24;
const PRELUDE: [Step; 11] = [
    Step::Landing(Files::Inside),
    Step::Landing(Files::Unknown),
    Step::Landing(Files::Outside),
    Step::Hold(0),
    Step::Merge(0),
    Step::Owner(true),
    Step::Owner(false),
    Step::Start(0),
    Step::Check(true),
    Step::Restart,
    Step::Check(false),
];
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Files {
    Inside,
    Outside,
    Unknown,
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Step {
    Landing(Files),
    Start(u64),
    Hold(u64),
    Merge(u64),
    Skip(u64),
    Check(bool),
    Restart,
    Owner(bool),
}
fn stream(seed: u64) -> Rng {
    Rng::new(seed.wrapping_add(17_100))
}

fn drawn(random: &mut Rng) -> Step {
    match random.below(12) {
        0 | 1 => Step::Landing(Files::Inside),
        2 => Step::Landing(Files::Outside),
        3 => Step::Landing(Files::Unknown),
        4 => Step::Start(random.next_u64()),
        5 => Step::Hold(random.next_u64()),
        6 => Step::Merge(random.next_u64()),
        7 => Step::Skip(random.next_u64()),
        8 => Step::Check(random.below(2) == 0),
        9 => Step::Restart,
        10 => Step::Owner(true),
        _ => Step::Owner(false),
    }
}
struct Driver {
    world: World,
    referee: Referee,
    red: BTreeSet<u64>,
    key: u8,
    role: u64,
    task_owner: bool,
    owner_edits: u8,
}
impl Driver {
    fn new() -> Self {
        let mut world = proof::fixture();
        let role = role::standing(&mut world, 150, 300);
        Self {
            world,
            referee: Referee::default(),
            red: BTreeSet::new(),
            key: 200,
            role,
            task_owner: false,
            owner_edits: 0,
        }
    }
    fn act(&mut self, act: FollowAct) {
        self.key = self.key.checked_add(1).expect("bounded script keys");
        proof::owner_session(&mut self.world);
        proof::owner(&mut self.world, self.key, act);
    }
    fn entry(&self, at: u64) -> Option<forge::FollowEntry> {
        let queue = &proof::follow(&self.world).queue;
        if queue.is_empty() {
            return None;
        }
        let at = usize::try_from(at % u64::try_from(queue.len()).expect("bounded queue")).expect("bounded index");
        queue.get(at).cloned()
    }
    fn waiting(entry: &forge::FollowEntry) -> bool {
        !matches!(entry.state, FollowState::Producing | FollowState::Held(_) | FollowState::Ended(_))
    }
    fn step(&mut self, step: Step) {
        match step {
            Step::Landing(files) => {
                // Two queued entries fit the fixture's 60-unit follow budget without starving a producer.
                // Each observed move remains one landing, rather than a full-queue accumulated range.
                if proof::follow(&self.world).queue.len() >= 2 {
                    return;
                }
                self.world.fail_comparisons = files == Files::Unknown;
                let path = if files == Files::Inside { b"src/a.c".as_slice() } else { b"src-old/a.c".as_slice() };
                let message = format!("source move {}", self.referee.moves.len());
                let after = proof::landing(&mut self.world, path, message.as_bytes());
                proof::heard(&mut self.world, after);
                self.world.fail_comparisons = false;
                self.referee.moves.push((after, files != Files::Outside));
            }
            Step::Start(at) => {
                if let Some(entry) = self.entry(at)
                    && Self::waiting(&entry)
                {
                    self.act(FollowAct::Start { after: entry.after });
                }
            }
            Step::Hold(at) => {
                if let Some(entry) = self.entry(at)
                    && Self::waiting(&entry)
                {
                    self.act(FollowAct::Hold { after: entry.after, reason: b"wait for source fix".as_slice().into() });
                }
            }
            Step::Skip(at) => {
                if let Some(entry) = self.entry(at)
                    && Self::waiting(&entry)
                {
                    // The referee needs every skip's evidence, but the follow keeps only its newest skips.
                    let skips = proof::follow(&self.world).skips.len();
                    if skips < skein_lib::usize_of(self.world.env.limits.forge.follow_skips) {
                        self.act(FollowAct::Skip { after: entry.after, reason: b"already ported".as_slice().into() });
                    } else {
                        eprintln!("skip ineligible: the follow's skip history is full");
                    }
                }
            }
            Step::Merge(at) => {
                if let Some(first) = self.entry(at) {
                    let queue = &proof::follow(&self.world).queue;
                    if let Some(index) = queue.iter().position(|entry| entry.after == first.after)
                        && let Some(next) = queue.get(index + 1)
                        && next.change != first.change
                        && Self::waiting(&first)
                        && Self::waiting(next)
                    {
                        let group: Vec<_> = queue
                            .iter()
                            .filter(|entry| entry.change == first.change || entry.change == next.change)
                            .collect();
                        if group.len()
                            <= usize::try_from(self.world.env.limits.forge.follow_merge).expect("merge bound")
                        {
                            let afters = group.iter().map(|entry| entry.after).collect();
                            self.act(FollowAct::Merge { afters });
                        }
                    }
                }
            }
            Step::Check(red) => self.check(red),
            Step::Owner(task_owner) => {
                // Repeated remakes count against the role's lifetime tree bound, not a fresh allowance.
                if task_owner == self.task_owner || self.owner_edits == 4 {
                    return;
                }
                self.key += 1;
                role::idle(&mut self.world, self.role);
                proof::owner_session(&mut self.world);
                let owner = if task_owner {
                    forge::settings::FollowOwner::Task(self.role)
                } else {
                    forge::settings::FollowOwner::Person(1)
                };
                role::owner(&mut self.world, owner, self.key);
                role::idle(&mut self.world, self.role);
                self.task_owner = task_owner;
                self.owner_edits += 1;
            }
            Step::Restart => {
                self.world.drain_cut();
                self.world.restart_same();
                proof::session(&mut self.world);
                self.world.drain_cut();
            }
        }
        self.world.drain_cut();
        self.referee.check(&mut self.world);
    }
    fn check(&mut self, red: bool) {
        let Some(entry) = proof::follow(&self.world).queue.first().cloned() else {
            return;
        };
        if entry.state != FollowState::Producing {
            return;
        }
        let change = entry.change.expect("made head");
        let producer = proof::delegate(&mut self.world, change, 0);
        let red = red && self.red.insert(change);
        proof::port(&mut self.world, change, &producer, red);
        if red {
            let repair = proof::delegate(&mut self.world, change, producer.task);
            assert!(repair.sections.iter().any(|section| matches!(&section.body,
                temper_engine_domain::BriefBody::Text(words) if String::from_utf8_lossy(words).contains("tests/handoff"))), "red names its test");
        } else {
            proof::landed(&mut self.world, change);
        }
    }
    fn finish(&mut self) {
        for _ in 0..16 {
            let Some(entry) = proof::follow(&self.world).queue.first().cloned() else {
                self.referee.check(&mut self.world);
                assert_eq!(self.referee.resolved(), self.referee.kept());
                return;
            };
            if Self::waiting(&entry) {
                self.step(Step::Start(0));
            }
            self.step(Step::Check(false));
        }
        panic!("seed's tail did not settle: {:?}", proof::follow(&self.world));
    }
}
fn same(first: &Driver, replay: &Driver, seed: u64, step: Step) {
    same_digest(
        &(&first.world.root, &first.world.store.rows, &first.world.events),
        &(&replay.world.root, &replay.world.store.rows, &replay.world.events),
        &format_args!("follow seed {seed}: {step:?}"),
    );
    assert_eq!(first.world.writes, replay.world.writes, "forge trace seed {seed}: {step:?}");
    assert_eq!(first.world.assigned, replay.world.assigned, "worker trace seed {seed}: {step:?}");
}
fn run_replayed(seed: u64) {
    let mut random = stream(seed);
    let mut first = Driver::new();
    let mut replay = Driver::new();
    // A live red crosses a restart on every seed; random steps then interleave all stimuli.
    for step in PRELUDE {
        first.step(step);
        replay.step(step);
        same(&first, &replay, seed, step);
    }
    for number in 0..STEPS {
        // The fake worker answers a live run on its next turn, within the shipped run deadline.
        let step =
            if proof::follow(&first.world).queue.first().is_some_and(|entry| entry.state == FollowState::Producing) {
                Step::Check(random.below(2) == 0)
            } else {
                drawn(&mut random)
            };
        eprintln!("follow seed {seed} step {number}: {step:?}");
        first.step(step);
        replay.step(step);
        same(&first, &replay, seed, step);
    }
    first.finish();
    replay.finish();
    same(&first, &replay, seed, Step::Check(false));
    same_text(&first.world.store.rows, &replay.world.store.rows, &format_args!("settled follow seed {seed}"));
    same_text(&first.world.fake, &replay.world.fake, &format_args!("settled forge seed {seed}"));
    first.world.close();
    replay.world.close();
}
/// A short follows workload, with the independent restart stream enabled.
pub(super) fn run_with_restarts(seeds: &[u64]) -> Vec<skein_world::faults::FaultLedger> {
    use temper_engine_domain_world::forge_routes::restarts::Injector;
    // A live red crosses the drawn restart. Two further stimuli keep this a
    // follows sweep without repeating the full follows suite's prelude/tail.
    const RESTART_STEPS: u32 = 2;
    const RESTART_HORIZON: u64 = 100;
    let mut driver = Driver::new();
    let mut ledgers = Vec::new();
    for &seed in seeds {
        eprintln!("engine restart seed {seed}");
        let mut random = stream(seed);
        driver.world.drain_cut();
        driver.world.inject_restarts(
            Injector::new(seed, &temper_faults::restarts::ENGINE, RESTART_HORIZON).with_recovery_clock(),
        );
        for step in [Step::Landing(Files::Inside), Step::Start(0), Step::Check(true)] {
            driver.step(step);
        }
        for _ in 0..RESTART_STEPS {
            let step = if proof::follow(&driver.world)
                .queue
                .first()
                .is_some_and(|entry| entry.state == FollowState::Producing)
            {
                Step::Check(random.below(2) == 0)
            } else {
                drawn(&mut random)
            };
            driver.step(step);
        }
        driver.finish();
        let ledger = driver.world.finish_restarts();
        driver.finish();
        ledgers.push(ledger);
    }
    driver.world.close();
    ledgers
}

#[test]
fn fuzzy_follows_draws_keep_gate_steps_and_accept_full_width_seeds() {
    let mut digest = StateHasher::new();
    for seed in 0..SEEDS {
        let mut random = stream(seed);
        for _ in 0..STEPS {
            drawn(&mut random).hash(&mut digest);
        }
    }
    assert_eq!(digest.finish(), 0xee7f_d68c_2f8e_656b, "step draws in gate seed order");

    let mut random = stream(u64::MAX);
    for _ in 0..STEPS {
        let _ = drawn(&mut random);
    }
}

fn shard(test: Test, seeds: Range<u64>) {
    cases(test, seeds, |seed| {
        run_replayed(seed);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
#[test]
fn fuzzy_follows_shard_0() {
    shard(skein_world::this_test!("fuzzy_follows_shard_0"), 0..4);
}
#[test]
fn fuzzy_follows_shard_1() {
    shard(skein_world::this_test!("fuzzy_follows_shard_1"), 4..8);
}
#[test]
fn fuzzy_follows_shard_2() {
    shard(skein_world::this_test!("fuzzy_follows_shard_2"), 8..12);
}
#[test]
fn fuzzy_follows_shard_3() {
    shard(skein_world::this_test!("fuzzy_follows_shard_3"), 12..SEEDS);
}
