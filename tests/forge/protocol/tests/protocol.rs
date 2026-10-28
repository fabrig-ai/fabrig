use skein_forge::api::{Answer, Read, State};
use skein_forge_client::{Event, Resource, Watch, What};
use skein_forge_machine_world::joint::REPO;
use skein_forge_protocol_world::{
    World,
    fixtures::{self, LIMITS},
};
use skein_lib::{Duration, Time, Token};
use std::collections::BTreeSet;

#[test]
fn a_creation_lost_on_the_wire_is_found_after_restart() {
    let mut world = fixtures::lost_creation(23);
    assert_eq!(world.creations(), 1);
    assert!(!world.made(1));
    world.restart();
    world.finish();
    assert_eq!(world.creations(), 1);
    assert!(world.made(1));
    assert_eq!(world.owner_counts(), (0, 0));
}

#[test]
fn listings_follow_another_partys_changes() {
    let mut world = World::new(5, 0);
    world.run_for(Duration::from_secs(1));
    let a = world.person_issue(b"A");
    let b = world.person_issue(b"B");
    let c = world.person_issue(b"C");
    world.person_edit(b, b"B2");
    world.person_close(c);

    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut page = 1;
    loop {
        let Answer::Items { items, more, .. } =
            world.read(Read::Items { since: Time::ZERO, page, kind: None }).expect("owner listing")
        else {
            panic!("listing returns items")
        };
        for item in items {
            assert!(seen.insert(item.number), "no number twice across pages");
            rows.push((item.number, item.title, item.state == State::Open));
        }
        if !more {
            break;
        }
        page += 1;
    }
    assert!(page > 1, "listing crosses the fake's page size of two");
    rows.sort_by_key(|row| row.0);
    assert_eq!(rows, world.snapshot());
    assert_eq!(
        rows,
        vec![
            (a, Box::from(b"A".as_slice()), true),
            (b, Box::from(b"B2".as_slice()), true),
            (c, Box::from(b"C".as_slice()), false)
        ]
    );
}

#[test]
fn signed_deliveries_reach_the_clients_owner() {
    let mut world = fixtures::hooked(7);
    world.run_for(Duration::from_secs(1));
    let number = world.person_issue(b"before");
    let resource = Resource { repository: REPO, what: What::Issue(number) };
    world.event(Event::Keep {
        owner: Token::new(1),
        watches: Box::new([Watch { resource: resource.clone(), participating: false }]),
    });
    world.run_for(Duration::from_secs(2));
    let Some(Answer::Item { item, .. }) = world.changed(&resource) else { panic!("kept issue is read") };
    assert_eq!(&*item.title, b"before");

    world.person_edit(number, b"after");
    let after = Duration::from_secs(2);
    assert!(after < LIMITS.poll, "the hint brings the change before polling");
    world.run_for(after);
    let Some(Answer::Item { item, .. }) = world.changed(&resource) else { panic!("hinted issue is read") };
    assert_eq!(&*item.title, b"after");
    assert_eq!(world.hints(), 2);
}

#[test]
fn directories_and_the_root_read_through_both_http_fakes() {
    use skein_fake_forge_domain::api::File;
    use skein_forge::api::{Entry, EntryKind};
    // Both entries together fit the existing owner read budget.
    const DIRECTORY_BYTES: u32 = 128;
    for mut world in [World::new(5, 0), World::github(5, 0)] {
        let head = world.tree(Box::new([
            File { path: Box::from(b"README".as_slice()), content: Box::from(b"hello".as_slice()) },
            File { path: Box::from(b"src/main.rs".as_slice()), content: Box::from(b"fn main() {}".as_slice()) },
        ]));
        for (path, entries) in [
            (
                b"".as_slice(),
                vec![
                    Entry { name: Box::from(b"README".as_slice()), kind: EntryKind::File },
                    Entry { name: Box::from(b"src".as_slice()), kind: EntryKind::Directory },
                ],
            ),
            (b"src", vec![Entry { name: Box::from(b"main.rs".as_slice()), kind: EntryKind::File }]),
        ] {
            assert_eq!(
                world.read(Read::File { head, path: Box::from(path), max_bytes: DIRECTORY_BYTES }),
                Ok(Answer::Directory {
                    head,
                    path: Box::from(path),
                    entries: entries.into_boxed_slice(),
                    truncated: false
                })
            );
        }
        world.finish();
        assert_eq!(world.owner_counts(), (0, 0));
    }
}
