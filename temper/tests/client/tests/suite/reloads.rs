//! A load releases the preceding client's watches (client/browser.md, 2.4).
use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::fixtures;

#[test]
fn repeated_loads_release_the_previous_clients_watches() {
    let mut world = fixtures::world(fixtures::SEED);
    let client = world.client(fixtures::PARTY, fixtures::chats_page());
    let loads = temper_engine_domain::VIEWS.watchers_per_party + 2;
    for _ in 0..loads {
        world.person(
            client,
            vec![Step::See { find: Find::exactly_named(Role::Status, b"Live"), within: fixtures::WITHIN }],
        );
        assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
        world.reload(client);
    }
    world
        .person(client, vec![Step::See { find: Find::exactly_named(Role::Status, b"Live"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
