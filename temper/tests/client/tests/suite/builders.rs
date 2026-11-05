//! Builders reads the joined engine's deployment (client/README.md, 3.5, Builders).
use super::model::{follow, open, see};
use skein_ui::Role;
use temper_client_domain::pages::builders::Read;
use temper_client_world::fixtures as f;

#[test]
fn an_owner_reads_the_engines_worker_on_builders() {
    let mut world = f::accounts_world(f::SEED);
    world.front().engine.hello(skein_lib::Token::new(7));
    let client = open(&mut world, b"/projects/1/settings/builders");
    follow(
        &mut world,
        client,
        vec![
            see(Role::Heading { level: 2 }, "Builders · 1".as_bytes()),
            see(Role::Text, b"worker"),
            see(Role::Text, "1 slot · 0 in use".as_bytes()),
            see(Role::Text, b"Connected"),
        ],
    );
    let root = world.root(client).person();
    let Read::Hosts(doc) = root.pages().builders().unwrap().read(root.client()) else {
        panic!("engine deployment read")
    };
    let worker = doc.hosts.get(0).expect("one worker");
    assert_eq!(doc.hosts.len(), 1);
    assert_eq!(worker.channel, 7);
    assert_eq!(worker.name.as_deref(), Some(b"worker".as_slice()));
    assert!(worker.connected);
}
#[test]
fn a_person_with_no_role_is_told_who_sees_them() {
    // This fixture gives its second person a project membership and no deployment role.
    let mut world = f::configuring_world(f::SEED, true);
    let caller = world.front().caller;
    let client = world.client(caller, temper_client_domain::Address::Builders { project: 1 });
    follow(
        &mut world,
        client,
        vec![
            see(Role::Status, b"Live"),
            see(Role::Paragraph, "Builders are seen by the deployment’s owners.".as_bytes()),
        ],
    );
    let root = world.root(client).person();
    assert_eq!(root.pages().me(root.client()).unwrap().deployment_role, None);
    assert!(matches!(root.pages().builders().unwrap().read(root.client()), Read::NoRole));
    assert!(!world.tree(client).nodes().iter().any(|n| n.role == Role::Heading { level: 2 }));
}
