//! Random people on authentic core worlds, one sweep per fault pair
//! (jig/docs/design/domain/testing-workloads.md, 7).
use jig_client_world::joint::random::{self, ClientFault, EngineFault};
use skein_lib::Rng;
use skein_world::{
    domain::Verdict,
    sweep::{Test, cases, gate},
    this_test,
};
use std::convert::Infallible;

const FIRST_SEED: u64 = 4500;
const SEEDS: u64 = 64;
const PINNED: &[u64] = &[];

fn sweep(test: Test, engine: EngineFault, client: ClientFault) {
    let pair = (engine, client);
    let seeds =
        (FIRST_SEED..FIRST_SEED + SEEDS).chain(PINNED.iter().copied()).filter(|&seed| random::assigned(seed) == pair);
    let mut reached = 0;
    cases(test, seeds, |mut seed| {
        // Keep a matching seed as-is. Otherwise draw again from this case's own
        // stream until the draw gives the pair the test is named for.
        let mut draws = Rng::new(seed);
        while random::assigned(seed) != pair {
            seed = draws.next_u64();
        }
        let (outcome, reach) = random::world(seed);
        assert_eq!(outcome.verdict, Verdict::Passed, "{engine:?}/{client:?} seed {seed}: {outcome:?}");
        assert_eq!((reach.engine, reach.client), pair);
        assert!(outcome.judged.0 > 0 && outcome.faults > 0);
        assert!(reach.submitted > 0 && reach.durable_answers >= 2 && reach.acknowledged > 0);
        reached += 1;
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if gate() {
        assert!(reached > 0, "each actual drawn pair reached: {engine:?}/{client:?}");
    }
}

#[test]
fn random_people_with_held_commits_and_drop_watch() {
    sweep(this_test!("random_people_with_held_commits_and_drop_watch"), EngineFault::Held, ClientFault::DropWatch);
}

#[test]
fn random_people_with_held_commits_and_slow() {
    sweep(this_test!("random_people_with_held_commits_and_slow"), EngineFault::Held, ClientFault::Slow);
}

#[test]
fn random_people_with_held_commits_and_no_answer() {
    sweep(this_test!("random_people_with_held_commits_and_no_answer"), EngineFault::Held, ClientFault::NoAnswer);
}

#[test]
fn random_people_with_held_commits_and_reload() {
    sweep(this_test!("random_people_with_held_commits_and_reload"), EngineFault::Held, ClientFault::Reload);
}

#[test]
fn random_people_with_held_commits_and_double_press() {
    sweep(this_test!("random_people_with_held_commits_and_double_press"), EngineFault::Held, ClientFault::DoublePress);
}

#[test]
fn random_people_with_crash_commits_and_drop_watch() {
    sweep(this_test!("random_people_with_crash_commits_and_drop_watch"), EngineFault::Crash, ClientFault::DropWatch);
}

#[test]
fn random_people_with_crash_commits_and_slow() {
    sweep(this_test!("random_people_with_crash_commits_and_slow"), EngineFault::Crash, ClientFault::Slow);
}

#[test]
fn random_people_with_crash_commits_and_no_answer() {
    sweep(this_test!("random_people_with_crash_commits_and_no_answer"), EngineFault::Crash, ClientFault::NoAnswer);
}

#[test]
fn random_people_with_crash_commits_and_reload() {
    sweep(this_test!("random_people_with_crash_commits_and_reload"), EngineFault::Crash, ClientFault::Reload);
}

#[test]
fn random_people_with_crash_commits_and_double_press() {
    sweep(
        this_test!("random_people_with_crash_commits_and_double_press"),
        EngineFault::Crash,
        ClientFault::DoublePress,
    );
}

#[test]
fn random_people_with_failed_commits_and_drop_watch() {
    sweep(this_test!("random_people_with_failed_commits_and_drop_watch"), EngineFault::Failed, ClientFault::DropWatch);
}

#[test]
fn random_people_with_failed_commits_and_slow() {
    sweep(this_test!("random_people_with_failed_commits_and_slow"), EngineFault::Failed, ClientFault::Slow);
}

#[test]
fn random_people_with_failed_commits_and_no_answer() {
    sweep(this_test!("random_people_with_failed_commits_and_no_answer"), EngineFault::Failed, ClientFault::NoAnswer);
}

#[test]
fn random_people_with_failed_commits_and_reload() {
    sweep(this_test!("random_people_with_failed_commits_and_reload"), EngineFault::Failed, ClientFault::Reload);
}

#[test]
fn random_people_with_failed_commits_and_double_press() {
    sweep(
        this_test!("random_people_with_failed_commits_and_double_press"),
        EngineFault::Failed,
        ClientFault::DoublePress,
    );
}
