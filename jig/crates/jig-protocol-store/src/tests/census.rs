//! The census's clock, names and copied counts (protocol/store/README.md, 6.3).
use super::fixtures;
use crate::{Event, Request, Store, Write};
use jig_domain_core_vocabulary::documents::StoreCensus;
use skein_lib::{Queue, Time, Token};

fn ask(store: &mut Store, seconds: u64, request: Request) -> Box<[Event]> {
    let mut env = fixtures::env();
    env.now = Time::from_nanos(seconds.checked_mul(1_000_000_000).unwrap());
    let mut above = Queue::uncapped();
    crate::down(store, &env, request, &mut above, &mut fixtures::below());
    fixtures::events(&mut above)
}

fn opened() -> Store {
    let mut store = fixtures::over(fixtures::stores());
    let events = ask(
        &mut store,
        0,
        Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let [Event::Opened { last: 0, .. }, Event::Census { census }] = events.as_ref() else {
        panic!("opened then recovery's census: {events:?}");
    };
    assert_eq!(census.commit, 0);
    assert_eq!(census.since, 1);
    assert!(census.keyspaces.is_empty());
    store
}

#[test]
fn reports_follow_open_and_due_commit_answers_not_idle_time() {
    let mut store = opened();
    for (number, seconds, due) in [(1, 10, false), (2, 50, false), (3, 61, true), (4, 120, false), (5, 121, true)] {
        let events = ask(&mut store, seconds, fixtures::commit(number, Box::new([fixtures::header(number)])));
        if due {
            let [Event::Committed { number: found, .. }, Event::Census { census }] = events.as_ref() else {
                panic!("due report follows commit answer: {events:?}");
            };
            assert_eq!(*found, number);
            assert_eq!(census.commit, number);
        } else {
            let [Event::Committed { number: found, .. }] = events.as_ref() else {
                panic!("no report before the interval: {events:?}");
            };
            assert_eq!(*found, number);
        }
    }
    let mut env = fixtures::env();
    env.now = Time::from_nanos(300_000_000_000);
    let mut above = Queue::uncapped();
    crate::fire(&mut store, &env, &mut above, &mut fixtures::below());
    assert!(above.is_empty(), "elapsed time without a commit answer emits no census");
}

fn report(events: &[Event]) -> &StoreCensus {
    let [Event::Committed { .. }, Event::Census { census }] = events else { panic!("commit and census: {events:?}") };
    census
}

#[test]
fn reports_copy_named_keyspaces_and_leave_secrets_uncounted() {
    let mut store = opened();
    store.keyspaces(crate::Keyspaces { owner: "example", families: &[(12, "example.changes")] });
    let events = ask(
        &mut store,
        61,
        fixtures::commit(
            1,
            Box::new([
                fixtures::header(1),
                fixtures::save(fixtures::hold_milestone(21, 1)),
                Write::Put { key: Box::new([12, 9]), value: Box::new([0, 7, 42]) },
                Write::Put { key: Box::new([99]), value: Box::new([1]) },
            ]),
        ),
    );
    let census = report(&events);
    assert_eq!(census.since, store.stores.store.census().since());
    let mut previous: &[u8] = &[];
    for (row, counted) in census.keyspaces.iter().zip(store.stores.store.census().keyspaces()) {
        assert!(previous < &*row.prefix, "prefix order");
        previous = &row.prefix;
        assert_eq!(&*row.prefix, counted.prefix());
        assert_eq!((row.rows, row.keys, row.values), (counted.rows, counted.keys, counted.values));
        assert_eq!((row.puts, row.erases, row.written), (counted.puts, counted.erases, counted.written));
        assert_eq!(row.heads.len(), u32::try_from(counted.heads().len()).unwrap());
        for (head, counted) in row.heads.iter().zip(counted.heads()) {
            assert_eq!((head.head, head.rows), (counted.head, counted.rows));
        }
    }
    assert_eq!(census.keyspaces.len(), u32::try_from(store.stores.store.census().keyspaces().len()).unwrap());
    let milestone = space(census, &[1, 1, 10]);
    assert_eq!(&*milestone.name, "tasks.milestones");
    assert_eq!(milestone.rows, 1);
    assert_eq!(milestone.puts, 1);
    let application = space(census, &[2, 12]);
    assert_eq!(&*application.name, "example.changes");
    assert_eq!((application.rows, application.keys, application.values, application.written), (1, 3, 3, 6));
    assert_eq!(&*space(census, &[2, 99]).name, "");
    let marked = ask(&mut store, 62, fixtures::commit(2, Box::new([fixtures::header(2), Write::Wires])));
    let [Event::Committed { number: 2, .. }] = marked.as_ref() else { panic!("durable wires: {marked:?}") };
    let events = ask(
        &mut store,
        300,
        Request::Secrets {
            owner: Token::new(9),
            request: crate::Secrets::Keep {
                owner: crate::SecretOwner::SignIn,
                number: 1,
                value: crate::SecretValue::Digest { expiry: u64::MAX, digest: [3; 32] },
            },
        },
    );
    let [Event::Kept { .. }] = events.as_ref() else { panic!("secret answer without census: {events:?}") };
    assert_eq!(store.stores.secrets.last(), 1, "the secret was written");
    assert_eq!(store.stores.secrets.census().keyspaces(), []);
}

#[test]
fn the_rule_uses_owner_prefix_lengths_and_unknown_prefixes_stay_unnamed() {
    let state = crate::census::State::new();
    for (byte, &length) in state.rule.prefix.iter().enumerate() {
        assert_eq!(
            length,
            match byte {
                0 | 2 => 2,
                1 => 3,
                _ => 1,
            }
        );
    }
    for prefix in [&[][..], &[1], &[1, 1], &[255], &[0, 99], &[1, 99, 99]] {
        assert_eq!(crate::census::name(prefix, crate::Keyspaces::NONE), "");
    }
}

fn space<'a>(census: &'a StoreCensus, prefix: &[u8]) -> &'a jig_domain_core_vocabulary::documents::CensusKeyspace {
    for row in &census.keyspaces {
        if row.prefix.as_ref() == prefix {
            return row;
        }
    }
    panic!("missing keyspace {prefix:?}");
}
