//! A focused replay of the recovery deadline found by the fuzzy gate.
use jig_client_world::harness::Until;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;

#[test]
fn seed_five_recovers_online_after_the_four_faults() {
    let mut world = fixtures::world(5);
    world.randomness(12, 0, 0);
    let person = world.client(fixtures::PARTY, fixtures::chats_page());
    world.random_person(person);
    for (at, fault) in fixtures::faults(5) {
        world.fault_at(at, fault);
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.judged.0 > 0);
}
