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
fn run(seed: u64) -> (landing::Landing, u64, Vec<api::Op>) {
    let mut rng = Rng::new(seed);
    let count = 2 + rng.below(3);
    let squash = rng.below(2) == 0;
    let red = rng.below(2) == 0;
    let conflict = (rng.below(2) == 0).then(|| 1 + rng.below(count - 1));
    let moving = (rng.below(2) == 0).then(|| rng.below(count));
    let red_member = rng.below(count);
    let mut world = landings::turn_in(World::batch_turn(seed, Some(workflow(red))));
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
            world.push_file(branch.as_bytes(), b"file", b"broken");
        }
        landings::ready_change(&mut world, 51 + index, branch.as_bytes());
    }
    Trains::default().judge(&mut world);
    let mut reported = false;
    let mut inject = moving;
    let mut pending = None;
    let mut finished = false;
    for step in 0..160 {
        let before = world.branch(b"main");
        let decision = landings::step_landing_heard(
            &mut world,
            LANDING,
            10_000 + step,
            pending.take().unwrap_or_else(|| heard(landing::Preparation::Idle, b"")),
        );
        if world.branch(b"main") != before {
            let landing::Decision::Merge { member, candidate } = decision else {
                panic!("seed {seed}: main moved outside the turn")
            };
            assert_eq!(translate::commit(world.branch(b"main")), candidate, "seed {seed}: only the proved top lands");
            assert!(contains(&world, number(candidate), translate::commit(before)), "seed {seed}: fast-forward only");
            let top = &world.landing(LANDING).landing.members[usize::from(member)];
            assert!(
                top.gates.iter().all(|gate| matches!(gate.proof, landing::Proof::Passed { .. })),
                "seed {seed}: proof before turn"
            );
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
    (row, main, world.take_calls())
}
#[test]
fn seeded_batches_keep_the_fast_forward_prefix_and_cleanup_invariants() {
    for seed in 1..65 {
        assert_eq!(run(seed), run(seed), "seed {seed} replays");
    }
}
