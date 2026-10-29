use jig_domain_core as core;
use jig_protocol_store::{Event, SecretOwner, Secrets, Write};
use jig_store_world::{
    World,
    referee::Referee,
    root::{self, LIVE},
};
use skein_lib::Secret;
use std::collections::BTreeMap;

fn committed(seed: u64) -> World {
    let mut world = World::new(seed);
    assert_eq!(world.commit(root::first()), Some(1));
    world
}

#[test]
fn restored_pages_equal_the_models_rows() {
    let mut world = committed(18);
    let expected: BTreeMap<Box<[u8]>, Box<[u8]>> = BTreeMap::from([
        (Box::from(b"a".as_slice()), Box::from(b"first".as_slice())),
        (Box::from(b"b".as_slice()), Box::from(b"second".as_slice())),
        (Box::from(b"c".as_slice()), Box::from(b"third".as_slice())),
        (Box::from(b"d".as_slice()), Box::from(b"fourth".as_slice())),
        (Box::from(b"e".as_slice()), Box::from(b"fifth".as_slice())),
    ]);
    let mut writes = vec![Write::Save(root::header(2))];
    writes.extend(expected.iter().map(|(key, value)| Write::Put { key: key.clone(), value: value.clone() }));
    assert_eq!(world.commit(writes.into_boxed_slice()), Some(2));
    world.crash(18);
    let restored = world.reopen();
    Referee::check(&world.model, &restored).expect("restored whole model prefix");
    for (label, load) in [("restart pages", false), ("child load pages", true)] {
        assert_eq!(world.application(load), expected, "{label}");
    }
    assert_eq!(world.get(core::Key::Tasks(jig_domain_tasks::Key::Live(LIVE))), root::live_task(LIVE));
    assert_eq!(world.read_task(LIVE), [root::live_task(LIVE)]);
}

#[test]
fn a_secret_kept_survives_a_crash() {
    let mut world = committed(18);
    let events = world.secret(Secrets::Keep {
        owner: SecretOwner::Application,
        number: 1,
        value: jig_protocol_store::SecretValue::Application(Secret::new(Box::from(b"private credential".as_slice()))),
    });
    assert!(matches!(events.as_slice(), [Event::Kept { .. }]));
    world.crash(18);
    world.reopen();
    world.mark().expect("restart mark before serving secrets");
    let events = world.secret(Secrets::Get { owner: SecretOwner::Application, number: 1 });
    let [Event::Secret { value: Some(jig_protocol_store::SecretValue::Application(secret)), .. }] = events.as_slice()
    else {
        panic!("secret terminal: {events:?}")
    };
    assert_eq!(secret.expose(), b"private credential");
}

#[test]
fn maximum_writer_rows_restore_across_a_crash_above_the_ordinary_read_allowance() {
    use jig_protocol_store::{Request, Restore, Row, sizes};
    use skein_lib::Token;
    let mut limits = root::limits();
    limits.core.load_bytes = 1;
    let mut world = World::unopened_observed(19, limits, None);
    assert_eq!(world.open(), Some(0));
    assert_eq!(world.commit(root::first()), Some(1));
    let first = vec![b'a'; usize::try_from(root::APPLICATION.key).expect("application key bound")].into_boxed_slice();
    let second = vec![b'b'; first.len()].into_boxed_slice();
    let value = vec![7; usize::try_from(root::APPLICATION.value).expect("application value bound")].into_boxed_slice();
    assert_eq!(
        world.commit(Box::new([
            Write::Save(root::header(2)),
            Write::Put { key: first.clone(), value: value.clone() },
            Write::Put { key: second.clone(), value: value.clone() },
        ])),
        Some(2)
    );
    world.crash(19);
    let restored = world.reopen();
    Referee::check(&world.model, &restored).expect("whole committed prefix after crash");
    let mut after = None;
    for (index, expected) in [first, second].into_iter().enumerate() {
        let events = world.request(Request::Restore {
            owner: Token::new(1),
            range: Restore::Application(Box::new([skein_kv::Range::prefix(b"")])),
            after,
            rows: 1,
            bytes: sizes::restore_bytes(&limits).expect("restart allowance fits"),
        });
        let [Event::Restored { rows, next, .. }] = events.as_slice() else { panic!("restore page: {events:?}") };
        let [Row::Application { key, value: found }] = rows.as_ref() else { panic!("one application row") };
        assert_eq!(*key, expected);
        assert_eq!(*found, value);
        assert_eq!(next.is_some(), index == 0);
        after = next.clone();
    }
}
