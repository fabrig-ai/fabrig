//! Bounded seeded interleavings of the agent tree and scripted
//! host (smith/docs/design/domain/run.md, section 12; skein/docs/foundation/testing-strategy.md, sections 6 and 7).

use core::convert::Infallible;

use skein_lib::{Duration, Rng};
use skein_world::sweep::{Case, cases, gate};
use smith_agent_world::{HostReply, Job, Settings, World};
use smith_domain::{
    Fact,
    run::{self, Answer, RunFailure},
};
use smith_session_world::shapes::Sample;

#[test]
fn many_host_schedules_settle_against_the_boundary_referee() -> Result<(), Case<Infallible>> {
    let collect_reach = gate();
    let mut ended = [0; 3];
    let mut shapes = Sample::new("many_host_schedules_settle_against_the_boundary_referee");
    cases(skein_world::this_test!("many_host_schedules_settle_against_the_boundary_referee"), 0..120, |seed| {
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
        if collect_reach {
            match world.answer() {
                Answer::Parked { .. } | Answer::Accepted { .. } => ended[0] += 1,
                Answer::Failed { .. } => ended[1] += 1,
                Answer::Refused(_) => ended[2] += 1,
            }
        }
        assert_eq!(world.judged().1, 1, "seed {seed}: the host received its answer");
        assert_eq!(world.lost(), 0, "seed {seed}: the shell drained facts within their capacity");
        shapes.ran(&world.reached()).expect("valid completion shapes");
        Ok(())
    })?;
    shapes.write();
    if collect_reach {
        assert!(ended[0] > 0 && ended[1] > 0, "both successful and failed interleavings occurred: {ended:?}");
    }
    Ok(())
}

#[test]
fn mid_delivery_schedules_settle_with_all_terminals() -> Result<(), Case<Infallible>> {
    let mut shapes = Sample::new("mid_delivery_schedules_settle_with_all_terminals");
    cases(skein_world::this_test!("mid_delivery_schedules_settle_with_all_terminals"), 0..40, |seed| {
        shapes.ran(&mid_delivery_case(seed).reached()).expect("valid completion shapes");
        Ok(())
    })?;
    shapes.write();
    Ok(())
}

pub(super) fn mid_delivery_case(seed: u64) -> World {
    let mut rng = Rng::new(seed);
    let calm = Settings::calm(seed.wrapping_add(2000));
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
    world
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
fn host_cancellation_at_many_moments_closes_the_whole_tree() -> Result<(), Case<Infallible>> {
    let collect_reach = gate();
    let mut cancelled = 0;
    let mut nested = 0;
    let mut done = 0;
    let mut shapes = Sample::new("host_cancellation_at_many_moments_closes_the_whole_tree");
    cases(skein_world::this_test!("host_cancellation_at_many_moments_closes_the_whole_tree"), 0..40, |seed| {
        let (outcome, world) = host_cancellation_case(seed);
        shapes.ran(&world.reached()).expect("valid completion shapes");
        if collect_reach {
            match outcome {
                Cancellation::Cancelled { child_opened } => {
                    cancelled += 1;
                    nested += usize::from(child_opened);
                }
                Cancellation::Finished => done += 1,
            }
        }
        Ok(())
    })?;
    shapes.write();
    if collect_reach {
        assert!(
            cancelled > 0 && nested > 0 && done > 0,
            "cancels crossed early, nested and already-finished runs: {cancelled}/{nested}/{done}"
        );
    }
    Ok(())
}

pub(super) enum Cancellation {
    Cancelled { child_opened: bool },
    Finished,
}

pub(super) fn host_cancellation_case(seed: u64) -> (Cancellation, World) {
    let settings = Settings {
        job: Job::Delegating,
        cancel_at: Some(Duration::from_millis((seed % 40) * 500)),
        races: 500,
        ..Settings::calm(seed.wrapping_add(600))
    };
    let mut world = World::new(settings);
    world.run(20_000);
    assert_eq!(world.judged().1, 1, "the referee met the host's one answer obligation");
    let outcome = match world.answer() {
        Answer::Failed { failure: RunFailure::Cancelled, .. } => Cancellation::Cancelled {
            child_opened: count(&world, |fact| {
                matches!(fact, run::facts::FactKind::Opened { kind: run::facts::Kind::Child { .. }, .. })
            }) > 0,
        },
        Answer::Accepted { .. } => Cancellation::Finished,
        answer @ (Answer::Parked { .. } | Answer::Refused(_) | Answer::Failed { .. }) => {
            panic!("seed {seed}: expected cancelled or already finished, got {answer:?}")
        }
    };
    (outcome, world)
}

#[test]
fn held_renewals_and_newer_grants_never_spend_a_completion_retry() -> Result<(), Case<Infallible>> {
    use super::super::accounts::{account_world, counts, grant, seconds};
    let mut shapes = Sample::new("held_renewals_and_newer_grants_never_spend_a_completion_retry");
    cases(skein_world::this_test!("held_renewals_and_newer_grants_never_spend_a_completion_retry"), 0..64, |seed| {
        let mut rng = Rng::new(seed);
        let arrives = rng.between(100, 500);
        let resumes = if rng.chance(500) { arrives - 1 } else { arrives + 1 };
        let mut world = account_world(
            Some(0),
            Some(grant(3, true, Duration::from_secs(60))),
            0,
            smith_domain::llm::Failure::Unauthorized,
        );
        world.start_at(seconds(90));
        for at in [5, 15, 25, 35] {
            world.renewals_at(seconds(at), rng.chance(500));
        }
        world.renewals_at(seconds(50), true);
        world.grant_at(seconds(rng.between(91, arrives - 2)), grant(3, true, Duration::ZERO));
        world.renewals_at(seconds(resumes), false);
        world.grant_at(seconds(arrives), grant(4, true, Duration::from_secs(3600)));
        world.run(2000);
        assert_eq!(counts(&world), (1, 0, 1, 0), "seed {seed}");
        assert_eq!(world.completions()[0].started, seconds(arrives), "seed {seed}");
        assert_eq!(world.completions()[0].grant.expect("account").generation, 4, "seed {seed}");
        shapes.ran(&world.reached()).expect("valid completion shapes");
        Ok(())
    })?;
    shapes.write();
    Ok(())
}
