use jig_client_world::harness::Until;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;

#[test]
fn random_people_and_faults_make_each_request_once() {
    for seed in 1..=8 {
        let mut world = fixtures::world(seed);
        world.randomness(12, 0, 0);
        let person = world.client(fixtures::PARTY, fixtures::chats_page());
        world.random_person(person);
        for (at, fault) in fixtures::faults(seed) {
            world.fault_at(at, fault);
        }
        let outcome = world.run(Until::Done);
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
        assert!(outcome.judged.0 > 0, "seed {seed}: the referee judged the run");
    }
}
