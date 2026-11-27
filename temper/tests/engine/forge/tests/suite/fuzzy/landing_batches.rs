//! Seeded batch turns on the independent forge (domain/landing.md, 16;
//! temper/docs/design/testing/workloads.md, 8.5).
use super::super::landing_batches::{heard, workflow};
use core::convert::Infallible;
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Rng;
use skein_world::sweep::{Test, cases, gate};
use std::io::{self, Write};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World,
    landings::{self, LANDING, Trains},
    translate,
};
fn number(commit: api::Commit) -> u64 {
    skein_fake_forge_domain::commit_count(&commit).expect("fake commit")
}
fn contains(world: &World, tip: u64, commit: api::Commit) -> bool {
    let raw::Answer::Commits { commits } = world.inspect(&raw::Read::Commits { before: tip, after: number(commit) })
    else {
        panic!("ancestry")
    };
    commits.is_empty()
}
#[derive(Debug, PartialEq)]
pub(crate) struct Replay {
    landing: landing::Landing,
    main: u64,
    calls: Vec<api::Op>,
    histories: Vec<Vec<top::history::Event>>,
    pub(super) skipped: Skipped,
}

/// Ineligible fixture draws, not failed engine work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Skipped {
    commits: u64,
    tasks: u64,
    gates: u64,
}

impl Skipped {
    pub(super) fn fits_commits(&mut self, world: &mut World, commits: u32) -> bool {
        if world.forge_mut().room().commits < commits {
            self.commits += 1;
            return false;
        }
        true
    }

    pub(super) fn fits_tasks(&mut self, world: &World, tasks: &[u64]) -> bool {
        if !world.can_hold_tasks(tasks) {
            self.tasks += 1;
            return false;
        }
        true
    }

    pub(super) fn no_gate(&mut self) {
        self.gates += 1;
    }

    pub(super) fn add(&mut self, other: Self) {
        self.commits += other.commits;
        self.tasks += other.tasks;
        self.gates += other.gates;
    }

    pub(super) fn report(self, test: Test) {
        if gate() {
            self.assert_none();
        } else {
            for (reason, count) in [("commit-room", self.commits), ("task-table", self.tasks), ("no-gate", self.gates)]
            {
                if count > 0 {
                    let _reported = writeln!(io::stderr().lock(), "fuzzy reach: {test} ineligible/{reason} {count}");
                }
            }
        }
    }

    fn assert_none(self) {
        assert_eq!(self, Self::default(), "all scripted draws were eligible");
    }
}

pub(crate) struct Workload {
    pub rng: Rng,
    pub count: u64,
    pub squash: bool,
    pub red: bool,
    pub conflict: Option<u64>,
    pub moving: Option<u64>,
    pub red_member: u64,
}
impl Workload {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let count = 2 + rng.below(3);
        let squash = rng.below(2) == 0;
        let red = rng.below(2) == 0;
        let conflict = (rng.below(2) == 0).then(|| 1 + rng.below(count - 1));
        let moving = (rng.below(2) == 0).then(|| rng.below(count));
        let red_member = rng.below(count);
        Self { rng, count, squash, red, conflict, moving, red_member }
    }
}
pub(crate) fn workload_workflow(seed: u64) -> raw::Workflow {
    workflow(Workload::new(seed).red)
}
fn run(seed: u64) -> Replay {
    let world = World::batch_turn(seed, Some(workload_workflow(seed)));
    run_in(world, seed, 160).1
}
/// Run the same workload and checks, retaining the world for a sweep's ledger.
pub(crate) fn run_in(world: World, seed: u64, horizon: u64) -> (World, Replay) {
    run_workload(world, seed, horizon, Workload::new(seed))
}

/// Keep this workload's draws and checks, with a red member for repair sweeps.
pub(crate) fn run_repairing_in(world: World, seed: u64, horizon: u64) -> (World, Replay) {
    run_workload(world, seed, horizon, Workload { red: true, ..Workload::new(seed) })
}

fn run_workload(world: World, seed: u64, horizon: u64, workload: Workload) -> (World, Replay) {
    let Workload { mut rng, count, squash, red, conflict, moving, red_member } = workload;
    let mut skipped = Skipped::default();
    let mut world = setup_batch(world, count, red, conflict, red_member, &mut skipped);
    Trains::default().judge(&mut world);
    let mut reported = false;
    let mut inject = moving;
    let mut pending = None;
    let mut finished = false;
    let mut checks = BatchChecks::default();
    let restart = rng.below(4);
    for step in 0..horizon {
        if step == restart {
            let before: Vec<_> = (51..51 + count).map(|change| world.history(change)).collect();
            world.restart();
            let after: Vec<_> = (51..51 + count).map(|change| world.history(change)).collect();
            assert_eq!(before, after, "seed {seed}: restart writes no event twice");
        }
        let prior = world.landing(LANDING).landing.clone();
        let before = world.branch(b"main");
        let mut news = pending.take().unwrap_or_else(|| heard(landing::Preparation::Idle, b""));
        news.released = world.release_restored_fault(LANDING);
        let decision = landings::step_landing_heard(&mut world, LANDING, 10_000 + step, news);
        checks.step(&mut world, seed, LANDING, &prior, before, decision);
        if matches!(decision, landing::Decision::Prepare { .. }) {
            let selected: Vec<_> = (0..usize::try_from(count).expect("members"))
                .filter(|index| conflict != Some(u64::try_from(*index).expect("index")))
                .collect();
            world.prepare_subset(LANDING, squash, &selected);
            if !reported && let Some(member) = conflict {
                let result = format!(
                    r#"{{"conflicts":[{{"member":{},"with":"member","files":["member0"],"cut":false}}]}}"#,
                    member + 1
                );
                let mut news = heard(landing::Preparation::Idle, result.as_bytes());
                news.outcomes = top::preparation::outcomes(result.as_bytes(), u32::try_from(count).expect("members"))
                    .expect("worker conflict");
                pending = Some(news);
                reported = true;
            }
        }
        if world.landing(LANDING).landing.phase == landing::Phase::Proving
            && let Some(index) = inject.take()
            && skipped.fits_commits(&mut world, 1)
        {
            world.push_file(format!("temper/c{}", 51 + index).as_bytes(), b"moved", b"new delivery");
        }
        if decision == landing::Decision::Finish {
            finished = true;
            break;
        }
    }
    assert!(finished, "seed {seed}: every batch terminates: {:?}", world.landing(LANDING));
    let row = world.landing(LANDING).landing.clone();
    let main = world.branch(b"main");
    check_members(&world, seed, &row);
    landings::close_landing(&mut world, LANDING);
    for member in &row.members {
        assert!(!world.has_branch(&member.branch), "seed {seed}: prefix released");
    }
    let mut histories = Vec::new();
    for change in 51..51 + count {
        let events = world.history(change);
        assert!(!events.is_empty(), "seed {seed}: history is exercised");
        super::history::check(seed, change, &events);
        histories.push(events);
    }
    let replay = Replay { landing: row, main, calls: world.take_calls(), histories, skipped };
    (world, replay)
}
#[test]
fn seeded_batches_keep_the_fast_forward_prefix_and_cleanup_invariants() {
    let test = skein_world::this_test!("seeded_batches_keep_the_fast_forward_prefix_and_cleanup_invariants");
    let mut skipped = Skipped::default();
    cases(test, 1..65, |seed| {
        let replay = run(seed);
        assert_eq!(replay, run(seed), "seed {seed} replays");
        skipped.add(replay.skipped);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    skipped.report(test);
}

/// The same workload preparation for fault and party sweeps.
pub(super) fn setup(world: World, count: u64, red: bool, conflict: Option<u64>, red_member: u64) -> World {
    let mut skipped = Skipped::default();
    let world = setup_batch(world, count, red, conflict, red_member, &mut skipped);
    skipped.assert_none();
    world
}

fn setup_batch(
    mut world: World,
    count: u64,
    red: bool,
    conflict: Option<u64>,
    red_member: u64,
    skipped: &mut Skipped,
) -> World {
    world = landings::turn_in(world);
    world.batch(u16::try_from(count).expect("small batch"));
    world.push_file(b"main", b"file", b"fixed");
    for index in 0..count {
        let branch = format!("temper/c{}", 51 + index);
        world.external(
            raw::Op::Git(raw::Git::Create { branch: branch.as_bytes().into(), commit: world.branch(b"main") }),
            2,
        );
        let path = format!("member{}", if conflict == Some(index) { 0 } else { index });
        world.push_file(branch.as_bytes(), path.as_bytes(), &index.to_be_bytes());
        if red && index == red_member && skipped.fits_commits(&mut world, 1) {
            world.push_wrong_tree(branch.as_bytes(), b"file", b"broken");
        }
        landings::ready_change(&mut world, 51 + index, branch.as_bytes());
    }
    world
}

#[derive(Default)]
pub(super) struct BatchChecks {
    probed: Vec<api::Commit>,
    search_round: u16,
    max_probes: u32,
    // Each proved candidate may explain one main move, including a late timed-out call.
    issued: Vec<(u16, api::Commit, u64, bool)>,
}
impl BatchChecks {
    pub fn step(
        &mut self,
        world: &mut World,
        seed: u64,
        task: u64,
        prior: &landing::Landing,
        before: u64,
        decision: landing::Decision,
    ) {
        let next = &world.landing(task).landing;
        if let Some(search) = &next.round.search {
            if prior.round.search.is_none() {
                self.probed.clear();
                self.search_round = next.preparation;
                let n = prior
                    .members
                    .iter()
                    .filter(|m| m.left.is_none() && m.landed.is_none() && m.candidate.is_some())
                    .count();
                self.max_probes = usize::BITS - n.saturating_sub(1).leading_zeros();
            }
            if let Some(member) = landing::probe(next) {
                let prefix = next.members[usize::from(member)].candidate.expect("probed prefix");
                if !self.probed.contains(&prefix) {
                    self.probed.push(prefix);
                }
            }
            assert_eq!(self.search_round, next.preparation);
            assert!(
                self.probed.len() <= usize::try_from(self.max_probes).expect("probe count"),
                "seed {seed}: logarithmic probes"
            );
            for (index, (old, new)) in prior.members.iter().zip(&next.members).enumerate() {
                if old.landed.is_none() && new.landed.is_some() {
                    assert!(index < usize::from(search.red), "seed {seed}: no red suffix lands");
                }
                if matches!(new.left, Some(landing::Leave::Blamed { .. })) && old.left.is_none() {
                    let first = prior
                        .members
                        .iter()
                        .skip(usize::from(search.green))
                        .find(|m| m.left.is_none() && m.landed.is_none() && m.candidate.is_some())
                        .expect("unit above green");
                    assert_eq!(new.unit, first.unit, "seed {seed}: unit above green blamed");
                }
            }
        }
        if let landing::Decision::Merge { member, candidate } = decision {
            let green = prior.members[usize::from(member)]
                .gates
                .iter()
                .all(|gate| matches!(gate.proof, landing::Proof::Passed { .. }));
            self.issued.push((member, candidate, before, green));
        }
        let others = world.take_other_merges();
        let after = world.branch(b"main");
        if after != before {
            let matching: Vec<_> = self
                .issued
                .iter()
                .enumerate()
                .filter(|(_, (_, candidate, _, _))| number(*candidate) == after)
                .map(|(index, _)| index)
                .collect();
            let external: Vec<_> = others.iter().filter(|(_, from, to)| *from == before && *to == after).collect();
            assert_eq!(matching.len() + external.len(), 1, "seed {seed}: main moved outside the turn");
            if let Some(index) = matching.first() {
                let (_, candidate, base, green) = self.issued.remove(*index);
                assert!(green, "seed {seed}: proof before turn");
                assert!(contains(world, number(candidate), translate::commit(base)), "seed {seed}: fast-forward only");
            } else {
                assert_eq!(external.len(), 1, "seed {seed}: only the world's recorded other user may move main");
            }
        }
    }
}

pub(super) fn check_members(world: &World, seed: u64, row: &landing::Landing) {
    let main = world.branch(b"main");
    for member in &row.members {
        if let Some(prefix) = member.landed {
            assert!(member.left.is_none(), "seed {seed}");
            assert!(contains(world, main, prefix), "seed {seed}: prefix in main");
            if member.combined == Some(landing::Combine::Merge) {
                assert!(contains(world, main, member.head.expect("head")), "seed {seed}: merged head in main");
            }
        } else {
            assert!(member.left.is_some(), "seed {seed}: unlanded member has a cause");
        }
    }
}
