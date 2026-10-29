//! Seeded random people in both Ops starting states, judged by the shared referee.
use jig_client_view::address::Address as JigAddress;
use jig_client_world::Until;
use jig_ops_client_domain::{Address, observability as obs};
use jig_ops_world::client::stories;
use skein_world::domain::Verdict;
fn sweep(incident: bool, ceiling: u64) {
    let mut faults = 0;
    let mut judged = 0;
    for seed in 5050..5058 {
        eprintln!("Ops {} seed {seed}", if incident { "incident" } else { "environment chat" });
        let mut world = if incident { stories::incident_world(seed) } else { stories::environment_world(seed) };
        world.randomness(12, 150, 8);
        world.measure_heap_within(ceiling);
        let address = if incident {
            Address::Observability(obs::Address::Incidents { project: 1, open: true })
        } else {
            Address::Jig(JigAddress::Conversations { project: None })
        };
        for party in [7, 8] {
            let client = world.client(party, address.clone());
            world.random_person(client);
            let tab = world.tab(client);
            world.random_person(tab);
        }
        let result = world.run(Until::Done);
        assert_eq!(result.verdict, Verdict::Passed, "incident={incident} seed={seed}");
        faults += result.faults;
        judged += result.judged.0;
        assert!(result.heap_peak <= result.heap_bound);
    }
    assert!(faults > 0, "fault coverage must be positive");
    assert!(judged > 0, "referee coverage must be positive");
}
// Peak 1,923,687 bytes measured 10-29; testing-strategy.md 6.
const INCIDENT_SWEEP_HEAP_CEILING: u64 = 2_500_000;

#[test]
fn random_people_in_ops_incident_starts() {
    sweep(true, INCIDENT_SWEEP_HEAP_CEILING);
}
// Peak 1,823,596 bytes measured 10-29; testing-strategy.md 6.
const ENVIRONMENT_SWEEP_HEAP_CEILING: u64 = 2_300_000;

#[test]
fn random_people_in_ops_environment_chat_starts() {
    sweep(false, ENVIRONMENT_SWEEP_HEAP_CEILING);
}
