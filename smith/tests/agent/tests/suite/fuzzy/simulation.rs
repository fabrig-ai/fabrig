//! Bounded seeded interleavings of the agent tree and scripted
//! host (smith/docs/design/domain/run.md, section 13; skein/docs/foundation/testing-strategy.md, sections 6 and 7).

use skein_lib::{Duration, Rng};
use smith_agent_world::{HostReply, Job, Settings, World};
use smith_domain::{
    Fact,
    run::{self, Answer, RunFailure},
};

#[test]
fn many_host_schedules_settle_against_the_boundary_referee() {
    let mut ended = [0; 3];
    for seed in 0..120 {
        let mut rng = Rng::new(seed);
        let calm = Settings::calm(seed);
        let job = match rng.below(6) {
            0 => Job::Coding,
            1 => Job::Review,
            2 => Job::Reporting,
            3 => Job::Delegating,
            4 => Job::Spending,
            _ => Job::Failing,
        };
        let push = match rng.below(4) {
            0 => HostReply::Stale,
            1 => HostReply::Failed(run::DeliveryFailure::new(0, run::DeliveryReason::Unreachable)),
            _ => HostReply::Delivered,
        };
        let cancel_at = if rng.chance(300) { Some(Duration::from_millis(rng.below(20_000))) } else { None };
        let budget = run::Budget { turns: u32::try_from(rng.between(4, 64)).expect("a few turns"), ..calm.budget };
        let settings = Settings { job, push, cancel_at, budget, races: 500, ..calm };
        let mut world = World::new(settings);
        world.run(20_000);
        match world.answer() {
            Answer::Parked { .. } | Answer::Accepted { .. } => ended[0] += 1,
            Answer::Failed { .. } => ended[1] += 1,
            Answer::Refused(_) => ended[2] += 1,
        }
        assert_eq!(world.judged().1, 1, "seed {seed}: the host received its answer");
        assert_eq!(world.lost(), 0, "seed {seed}: the shell drained facts within their capacity");
    }
    assert!(ended[0] > 0 && ended[1] > 0, "both successful and failed interleavings occurred: {ended:?}");
}

#[test]
fn mid_delivery_schedules_settle_with_all_terminals() {
    for seed in 0..40 {
        let mut rng = Rng::new(seed);
        let calm = Settings::calm(seed + 2000);
        let push = match seed % 5 {
            0 => HostReply::Delivered,
            1 => HostReply::Nothing,
            2 => HostReply::Refused,
            3 => HostReply::Failed(run::DeliveryFailure::new(0, run::DeliveryReason::Unreachable)),
            _ => HostReply::Stale,
        };
        let settings = Settings {
            job: if seed % 2 == 0 { Job::MidReport } else { Job::MarkerReport },
            push,
            cancel_at: if rng.chance(500) { Some(Duration::from_millis(rng.below(2000))) } else { None },
            races: 500,
            ..calm
        };
        let mut world = World::new(settings);
        world.run(20_000);
        assert_eq!(world.judged().1, 1, "mid delivery seed {seed}: exactly one independently judged answer");
        assert_eq!(world.lost(), 0, "mid delivery seed {seed}: bounded facts were drained");
    }
}

fn count(world: &World, predicate: impl Fn(&run::facts::FactKind) -> bool) -> usize {
    world
        .facts()
        .iter()
        .filter(|fact| match fact {
            Fact::Run { fact, opening: _ } => predicate(&fact.kind),
            Fact::Session { .. } => false,
        })
        .count()
}

// This sweep seeks cancellation interleavings beyond the three named focused behaviours.
#[test]
fn host_cancellation_at_many_moments_closes_the_whole_tree() {
    let mut cancelled = 0;
    let mut nested = 0;
    let mut done = 0;
    for seed in 0..40 {
        let settings = Settings {
            job: Job::Delegating,
            cancel_at: Some(Duration::from_millis(seed * 500)),
            races: 500,
            ..Settings::calm(600 + seed)
        };
        let mut world = World::new(settings);
        world.run(20_000);
        assert_eq!(world.judged().1, 1, "the referee met the host's one answer obligation");
        match world.answer() {
            Answer::Failed { failure: RunFailure::Cancelled, .. } => {
                cancelled += 1;
                nested += usize::from(
                    count(&world, |fact| {
                        matches!(fact, run::facts::FactKind::Opened { kind: run::facts::Kind::Child { .. }, .. })
                    }) > 0,
                );
            }
            Answer::Accepted { .. } => done += 1,
            answer @ (Answer::Parked { .. } | Answer::Refused(_) | Answer::Failed { .. }) => {
                panic!("seed {seed}: expected cancelled or already finished, got {answer:?}")
            }
        }
    }
    assert!(
        cancelled > 0 && nested > 0 && done > 0,
        "cancels crossed early, nested and already-finished runs: {cancelled}/{nested}/{done}"
    );
}
