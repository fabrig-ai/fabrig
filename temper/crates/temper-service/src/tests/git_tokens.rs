use super::{fixtures, lifecycle_tests, routing_service};
use alloc::boxed::Box;
use jig_protocol_engine_workers::GrantValue;
use skein_forge_connection::{self as connection, Change};
use skein_lib::{Duration, List, Reader, Secret, Time, Wall};
use temper_engine_domain as root;
use temper_protocol_worker_wire::v1;

fn github() -> crate::Service {
    let mut service = routing_service(8, Duration::from_secs(3600));
    lifecycle_tests::app(&mut service);
    lifecycle_tests::accepted(&mut service, &[11, 22, 33, 44]);
    service
}
fn route(service: &mut crate::Service, event: connection::Event) {
    let engine = service.engine.as_mut().unwrap();
    engine.forge.above.push(event);
    crate::engine::forge_pass(engine);
}
fn token(installation: u64, account: u64, login: &[u8], bytes: &[u8], expires: u64) -> connection::Event {
    connection::Event::GitToken {
        forge: 1,
        installation,
        account: connection::Account { id: account, login: login.into() },
        token: Secret::new(bytes.into()),
        expires: Wall::from_nanos(expires),
    }
}
fn installation(id: u64, change: Change) -> connection::Event {
    connection::Event::Installation { forge: 1, installation: id, account: None, change }
}
fn credentials(service: &mut crate::Service) -> Box<[(u64, Duration)]> {
    let engine = service.engine.as_mut().unwrap();
    let mut events = List::with_capacity(engine.events.len());
    for _ in 0..engine.events.len() {
        let event = engine.events.pop().unwrap();
        if let root::Event::Credential { number, generation, valid } = event {
            assert_eq!(number, 1);
            assert!(engine.values.identities.get(number, generation).is_some(), "value precedes event");
            events.push((generation, valid)).unwrap();
        }
    }
    events.into_boxed()
}
fn value(service: &crate::Service, generation: u64) -> Option<v1::Identity> {
    let engine = service.engine.as_ref().unwrap();
    let values = engine.values.grants(&[fixtures::application_grant(1, generation)], Time::ZERO);
    match &values[0] {
        Some(GrantValue::Application { secret }) => {
            Some(v1::Identity::decode(&engine.workers.encoding.limits, &mut Reader::new(secret.expose())).unwrap())
        }
        None => None,
        _ => panic!("application grant"),
    }
}
type Entries = Box<[(u64, Box<[u8]>, Box<[u8]>)]>;
fn entries(service: &crate::Service, generation: u64) -> Entries {
    let value = value(service, generation).unwrap();
    let v1::Kind::Github(entries) = value.kind() else { panic!("GitHub") };
    let mut decoded = List::with_capacity(entries.entries().len());
    for entry in entries.entries() {
        decoded.push((entry.account(), entry.account_login().into(), entry.token().expose().into())).unwrap();
    }
    decoded.into_boxed()
}
#[test]
fn tokens_before_the_first_connected_are_announced_together_at_it() {
    let mut service = github();
    route(&mut service, token(22, 222, b"second", b"token-two", 200));
    route(&mut service, token(11, 111, b"first", b"token-one", 100));
    assert!(credentials(&mut service).is_empty());
    assert!(value(&service, 1).is_none());
    assert_eq!(crate::forge::tokens::account(&service.engine.as_ref().unwrap().values.tokens, 1, b"first"), None);
    route(&mut service, connection::Event::Connected { forge: 1 });
    assert_eq!(credentials(&mut service).as_ref(), [(1, Duration::from_nanos(100))]);
    assert_eq!(
        entries(&service, 1).as_ref(),
        [
            (111, Box::from(b"first".as_slice()), Box::from(b"token-one".as_slice())),
            (222, Box::from(b"second".as_slice()), Box::from(b"token-two".as_slice()))
        ]
    );
    let identity = value(&service, 1).unwrap();
    assert_eq!(identity.login(), b"x-access-token");
    assert_eq!(identity.name(), b"app-fixture[bot]");
    assert_eq!(identity.email(), b"9+app-fixture[bot]@users.noreply.github.com");
}
#[test]
fn configured_login_and_author_override_the_app_defaults() {
    let mut service = github();
    let mut settings = fixtures::engine();
    settings.forges[0].kind = crate::settings::Kind::GitHub;
    service.engine.as_mut().unwrap().values.tokens = crate::forge::tokens::Tokens::new(&settings);
    route(&mut service, connection::Event::Connected { forge: 1 });
    let identity = value(&service, 1).unwrap();
    assert_eq!(identity.login(), b"temper");
    assert_eq!(identity.name(), b"temper");
    assert_eq!(identity.email(), b"temper@forge.example");
}
#[test]
fn an_empty_list_is_announced_at_connected_valid_without_bound() {
    let mut service = github();
    route(&mut service, connection::Event::Connected { forge: 1 });
    assert_eq!(credentials(&mut service).as_ref(), [(1, Duration::from_nanos(u64::MAX))]);
    assert!(entries(&service, 1).is_empty());
}
#[test]
fn each_change_after_connected_makes_the_next_generation() {
    let mut service = github();
    for id in [11, 22, 33] {
        route(&mut service, token(id, id, b"owner", b"original", 100));
    }
    route(&mut service, connection::Event::Connected { forge: 1 });
    assert_eq!(credentials(&mut service).len(), 1);
    let rows = [
        (token(11, 111, b"renewed", b"renewal", 200), [111_u64, 22, 33].as_slice()),
        (token(44, 44, b"new", b"added", 200), [111, 22, 33, 44].as_slice()),
        (installation(11, Change::Deleted), [22, 33, 44].as_slice()),
        (installation(22, Change::Suspended), [33, 44].as_slice()),
    ];
    for (index, (event, expected)) in rows.into_iter().enumerate() {
        route(&mut service, event);
        let generation = u64::try_from(index).unwrap() + 2;
        assert_eq!(credentials(&mut service).as_ref(), [(generation, Duration::from_nanos(100))]);
        let actual = entries(&service, generation);
        assert_eq!(actual.len(), expected.len());
        for ((account, _, _), expected) in actual.iter().zip(expected) {
            assert_eq!(account, expected);
        }
        if generation == 2 {
            assert_eq!(actual[0].2.as_ref(), b"renewal");
        }
    }
    // The same kept-set path used after Accept, Reconcile and deletion.
    lifecycle_tests::accepted(&mut service, &[33]);
    super::lifecycle::installations(service.engine.as_mut().unwrap(), 1);
    assert_eq!(credentials(&mut service).as_ref(), [(6, Duration::from_nanos(100))]);
    assert_eq!(entries(&service, 6)[0].0, 33);
    assert_eq!(entries(&service, 6).len(), 1);
}
#[test]
fn a_change_without_an_entry_makes_no_generation() {
    let mut service = github();
    route(&mut service, connection::Event::Connected { forge: 1 });
    credentials(&mut service);
    for change in [Change::Deleted, Change::Suspended, Change::Unsuspended, Change::Renamed] {
        route(&mut service, installation(11, change));
        assert!(credentials(&mut service).is_empty());
    }
}
#[test]
fn a_duplicate_token_makes_no_generation() {
    let mut service = github();
    route(&mut service, token(11, 111, b"owner", b"same", 100));
    route(&mut service, connection::Event::Connected { forge: 1 });
    assert_eq!(credentials(&mut service).len(), 1);
    route(&mut service, token(11, 111, b"owner", b"same", 100));
    assert!(credentials(&mut service).is_empty());
    assert!(value(&service, 2).is_none());
}
#[test]
fn a_stopped_forge_keeps_its_entries_and_a_second_connected_announces_nothing() {
    let mut service = github();
    route(&mut service, token(11, 111, b"owner", b"kept", 100));
    route(&mut service, connection::Event::Connected { forge: 1 });
    credentials(&mut service);
    route(&mut service, connection::Event::Stopped { forge: 1, why: connection::Stopped::KeyRefused });
    route(&mut service, connection::Event::Connected { forge: 1 });
    assert!(credentials(&mut service).is_empty());
    assert_eq!(entries(&service, 1)[0].2.as_ref(), b"kept");
}
#[test]
fn the_values_validity_is_its_earliest_expiry() {
    let mut service = github();
    route(&mut service, token(11, 111, b"early", b"early-token", 100));
    route(&mut service, token(22, 222, b"late", b"late-token", 200));
    service.engine.as_mut().unwrap().env.wall = Wall::from_nanos(99);
    route(&mut service, connection::Event::Connected { forge: 1 });
    assert_eq!(credentials(&mut service).as_ref(), [(1, Duration::from_nanos(1))]);
    for (now, generation) in [(100, 2), (101, 3)] {
        service.engine.as_mut().unwrap().env.wall = Wall::from_nanos(now);
        route(&mut service, token(22, 222, b"late", b"renewed", now + 200));
        assert_eq!(credentials(&mut service).as_ref(), [(generation, Duration::ZERO)]);
    }
}
#[test]
fn the_value_server_answers_the_two_newest_generations() {
    let mut service = github();
    route(&mut service, connection::Event::Connected { forge: 1 });
    route(&mut service, token(11, 111, b"owner", b"second", 100));
    route(&mut service, token(11, 111, b"owner", b"third", 200));
    assert!(value(&service, 1).is_none());
    assert_eq!(entries(&service, 2)[0].2.as_ref(), b"second");
    assert_eq!(entries(&service, 3)[0].2.as_ref(), b"third");
}
#[test]
fn a_github_repository_carries_its_owners_entrys_account() {
    let mut service = github();
    route(&mut service, token(11, 111, b"OwNeR", b"account-token", 100));
    route(&mut service, connection::Event::Connected { forge: 1 });
    let engine = service.engine.as_ref().unwrap();
    for (owner, expected) in [(b"owner".as_slice(), Some(111)), (b"unknown", None)] {
        let assignment = fixtures::assignment_with(fixtures::repository(7, owner, b"repo"));
        let items = crate::translate::items(
            &assignment,
            &engine.workers.forges,
            &engine.values.tokens,
            &engine.workers.encoding.limits,
        )
        .unwrap()
        .unwrap();
        assert_eq!(*items.repositories().get(0).unwrap().account(), expected);
    }
    let assignment = fixtures::assignment_with(fixtures::repository(7, b"owner", b"repo"));
    let settings = fixtures::engine();
    let items =
        crate::translate::items(&assignment, &settings.forges, &engine.values.tokens, &engine.workers.encoding.limits)
            .unwrap()
            .unwrap();
    assert_eq!(*items.repositories().get(0).unwrap().account(), None);
}
#[test]
#[expect(clippy::disallowed_macros, reason = "formatting Debug proves root events and notes omit git tokens")]
fn no_token_reaches_a_root_event_or_a_note() {
    let mut service = github();
    let bytes = b"private-git-token-marker";
    route(&mut service, token(11, 111, b"owner", bytes, 100));
    route(&mut service, connection::Event::Connected { forge: 1 });
    route(&mut service, token(11, 111, b"owner", bytes, 200));
    route(&mut service, installation(11, Change::Suspended));
    route(&mut service, connection::Event::Stopped { forge: 1, why: connection::Stopped::KeyRefused });
    let engine = service.engine.as_ref().unwrap();
    assert!(!engine.notes.is_empty(), "the stop produces a note");
    let rendered = format!("{:?}{:?}{:?}", engine.events, engine.notes, engine.values);
    for window in rendered.as_bytes().windows(bytes.len()) {
        assert_ne!(window, bytes);
    }
}
