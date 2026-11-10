//! Independent party identities through the HTTP front (protocol/client/sign-in.md and protocol/client/README.md, 11).
use crate::{fixtures, stories};
use jig_client_kit::harness::{Seed, seed::RoleStory};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;

#[test]
fn two_parties_receive_their_own_me_over_http() {
    let mut world = fixtures::world(fixtures::SEED);
    world.seed(Seed::Roles { story: RoleStory::Goal });
    let clients = [1, 2].map(|party| {
        let client = world.client(party, jig_client_kit::stories::inbox());
        world.person(
            client,
            fixtures::sign_in_then(vec![Step::See {
                find: Find::named(Role::Button, b"Project 1"),
                within: Duration::from_secs(1),
            }]),
        );
        stories::finish(&mut world);
        client
    });
    let parties = clients.map(|client| {
        let root = world.root(client);
        root.jig().borrow(root.client()).frame().document.expect("me delivered over HTTP").party.number
    });
    assert_eq!(parties, [1, 2]);
}
