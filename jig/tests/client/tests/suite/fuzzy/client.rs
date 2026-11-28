//! Replayable random people over every story start and seeded random documents.
use jig_client_world::random::{self, Start};
use skein_lib::Rng;
use skein_world::{
    domain::Verdict,
    sweep::{Test, cases, gate},
    this_test,
};
use std::convert::Infallible;
// Every first dedicated seed reached 8 faults in the unchanged 16-seed inventory.
// Two keep drawn slack. Chat retains 16: the 32 worlds never opened its original address.
const FIRST_START_SEED: u64 = 100;
const SHORT_START_SEEDS: u64 = 2;
const CHAT_START_SEEDS: u64 = 16;
fn sweep(test: Test, start: Start) {
    let mut faults = 0;
    let count = if start == Start::Chat { CHAT_START_SEEDS } else { SHORT_START_SEEDS };
    cases(test, FIRST_START_SEED..FIRST_START_SEED + count, |seed| {
        let outcome = random::story_without_heap(seed, start);
        assert_eq!(outcome.verdict, Verdict::Passed, "{start:?}");
        faults += outcome.faults;
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if gate() {
        assert!(faults > 0);
    }
}
#[test]
fn random_people_in_new_chat_starts() {
    sweep(this_test!("random_people_in_new_chat_starts"), Start::NewChat);
}
#[test]
fn random_people_in_chat_starts() {
    sweep(this_test!("random_people_in_chat_starts"), Start::Chat);
}
#[test]
fn random_people_in_held_chat_starts() {
    sweep(this_test!("random_people_in_held_chat_starts"), Start::Held);
}
#[test]
fn random_people_in_proposal_starts() {
    sweep(this_test!("random_people_in_proposal_starts"), Start::Proposal);
}
#[test]
fn random_people_in_goal_starts() {
    sweep(this_test!("random_people_in_goal_starts"), Start::Goal);
}
#[test]
fn random_people_in_incident_starts() {
    sweep(this_test!("random_people_in_incident_starts"), Start::Incident);
}
// The gate partitions 200..232 by each seed's first draw. A campaign keeps
// the test's named fixture, even when a fresh seed would draw another start.
fn truth_sweep(test: Test, start: Start) {
    let seeds = (200..232).filter(|&seed| Start::draw(&mut Rng::new(seed)) == start);
    let mut counts = (0, 0, 0);
    cases(test, seeds, |seed| {
        let (outcome, reach) = random::world_with_coverage(seed, start);
        assert_eq!(reach.start, start, "the world keeps its named fixture");
        counts.0 += 1;
        counts.1 += outcome.faults;
        if reach.original_address {
            counts.2 += outcome.faults;
        }
        assert_eq!(outcome.verdict, Verdict::Passed, "{start:?}");
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if gate() {
        assert!(counts.0 > 0 && counts.1 > 0, "{start:?} truth reach {counts:?}");
        if start != Start::Chat {
            assert!(counts.2 > 0, "{start:?} retains original-address fault reach: {counts:?}");
        }
    }
    eprintln!("coverage truth {start:?}={counts:?}");
}
#[test]
fn random_truth_worlds_in_new_chat_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(
        this_test!("random_truth_worlds_in_new_chat_sweep_documents_addresses_and_persons_with_faults"),
        Start::NewChat,
    );
}
#[test]
fn random_truth_worlds_in_chat_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(
        this_test!("random_truth_worlds_in_chat_sweep_documents_addresses_and_persons_with_faults"),
        Start::Chat,
    );
}
#[test]
fn random_truth_worlds_in_held_chat_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(
        this_test!("random_truth_worlds_in_held_chat_sweep_documents_addresses_and_persons_with_faults"),
        Start::Held,
    );
}
#[test]
fn random_truth_worlds_in_proposal_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(
        this_test!("random_truth_worlds_in_proposal_sweep_documents_addresses_and_persons_with_faults"),
        Start::Proposal,
    );
}
#[test]
fn random_truth_worlds_in_goal_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(
        this_test!("random_truth_worlds_in_goal_sweep_documents_addresses_and_persons_with_faults"),
        Start::Goal,
    );
}
#[test]
fn random_truth_worlds_in_incident_sweep_documents_addresses_and_persons_with_faults() {
    truth_sweep(
        this_test!("random_truth_worlds_in_incident_sweep_documents_addresses_and_persons_with_faults"),
        Start::Incident,
    );
}
