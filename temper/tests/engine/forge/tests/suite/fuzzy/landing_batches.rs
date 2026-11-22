//! Seeded batch turns on the independent forge (domain/landing.md, 16).
use super::super::landing_batches::{heard, workflow};
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Rng;
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
}
struct Workload {
    rng: Rng,
    count: u64,
    squash: bool,
    red: bool,
    conflict: Option<u64>,
    moving: Option<u64>,
    red_member: u64,
}
impl Workload {
    fn new(seed: u64) -> Self {
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

#[expect(clippy::too_many_lines, reason = "batch effects and search invariants share one bounded replay loop")]
fn run_workload(world: World, seed: u64, horizon: u64, workload: Workload) -> (World, Replay) {
    let Workload { mut rng, count, squash, red, conflict, moving, red_member } = workload;
    let mut world = landings::turn_in(world);
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
        if red && index == red_member {
            world.push_wrong_tree(branch.as_bytes(), b"file", b"broken");
        }
        landings::ready_change(&mut world, 51 + index, branch.as_bytes());
    }
    Trains::default().judge(&mut world);
    let mut reported = false;
    let mut inject = moving;
    let mut pending = None;
    let mut finished = false;
    let mut probed = Vec::new();
    let mut search_round = 0;
    let mut max_probes = 0;
    // A timed-out call can move main in a later step. Each issued, proved
    // candidate may explain exactly one move; no other engine move is allowed.
    let mut issued = Vec::new();
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
        let next = &world.landing(LANDING).landing;
        if let Some(search) = &next.round.search {
            if prior.round.search.is_none() {
                probed.clear();
                search_round = next.preparation;
                let n = prior
                    .members
                    .iter()
                    .filter(|m| m.left.is_none() && m.landed.is_none() && m.candidate.is_some())
                    .count();
                max_probes = usize::BITS - n.saturating_sub(1).leading_zeros();
            }
            if let Some(member) = landing::probe(next) {
                let prefix = next.members[usize::from(member)].candidate.expect("probed prefix");
                if !probed.contains(&prefix) {
                    probed.push(prefix);
                }
            }
            assert_eq!(search_round, next.preparation);
            assert!(
                probed.len() <= usize::try_from(max_probes).expect("probe count"),
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
            issued.push((member, candidate, before, green));
        }
        let others = world.take_other_merges();
        let after = world.branch(b"main");
        if after != before {
            let matching: Vec<_> = issued
                .iter()
                .enumerate()
                .filter(|(_, (_, candidate, _, _))| number(*candidate) == after)
                .map(|(index, _)| index)
                .collect();
            let external: Vec<_> = others.iter().filter(|(_, from, to)| *from == before && *to == after).collect();
            assert_eq!(matching.len() + external.len(), 1, "seed {seed}: main moved outside the turn");
            if let Some(index) = matching.first() {
                let (_, candidate, base, green) = issued.remove(*index);
                assert!(green, "seed {seed}: proof before turn");
                assert!(contains(&world, number(candidate), translate::commit(base)), "seed {seed}: fast-forward only");
            } else {
                assert_eq!(external.len(), 1, "seed {seed}: only the world's recorded other user may move main");
            }
        }
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
    for member in &row.members {
        if let Some(prefix) = member.landed {
            assert!(member.left.is_none(), "seed {seed}");
            assert!(contains(&world, main, prefix), "seed {seed}: prefix in main");
            if member.combined == Some(landing::Combine::Merge) {
                assert!(contains(&world, main, member.head.expect("head")), "seed {seed}: merged head in main");
            }
        } else {
            assert!(member.left.is_some(), "seed {seed}: unlanded member has a cause");
        }
    }
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
    let replay = Replay { landing: row, main, calls: world.take_calls(), histories };
    (world, replay)
}
#[test]
fn seeded_batches_keep_the_fast_forward_prefix_and_cleanup_invariants() {
    for seed in 1..65 {
        assert_eq!(run(seed), run(seed), "seed {seed} replays");
    }
}
