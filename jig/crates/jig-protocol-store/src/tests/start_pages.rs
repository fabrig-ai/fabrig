//! Start pages through the request protocol (protocol/store.md, 7.1, 7.2 and 9).
use super::fixtures::{self, ALICE, ENDED, LIVE};
use crate::restore::{Cause, DecodeFailure, Failure, Item};
use crate::{Cursor, Event, Family, Phase, Request, Restore, Row, SecretOwner, SecretValue, Store, Write, keys};
use jig_domain_core::{Census, Key, ReadResults, Record, RestoreRange};
use skein_lib::{List, Queue, Token, Wall};

const OWNER: Token = Token::new(5);
const ROWS: u32 = 8;
const BYTES: u64 = 65_536;

fn open(selected: bool) -> Store {
    let mut store = fixtures::over(fixtures::stores());
    if selected {
        store.start_restore();
    }
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: OWNER, root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let [Event::Opened { .. }] = events.as_ref() else { panic!("store opens") };
    store
}

fn live() -> Restore {
    Restore::Jig(Box::new(RestoreRange::LIVE))
}

fn census() -> Restore {
    Restore::Jig(Box::new([RestoreRange::Census(Census {
        questions: Box::new([]),
        results: Box::new([ReadResults { party: ALICE, after: 0 }]),
    })]))
}

fn request(store: &mut Store, range: Restore, after: Option<Cursor>, rows: u32) -> Event {
    let mut events =
        fixtures::run(store, Request::Restore { owner: OWNER, range, after, rows, bytes: BYTES }).into_iter();
    let event = events.next().expect("one page terminal");
    assert!(events.next().is_none());
    event
}

fn page(event: Event) -> (Box<[Item]>, Option<Cursor>) {
    let Event::Page { owner, items, next } = event else { panic!("typed page: {event:?}") };
    assert_eq!(owner, OWNER);
    (items, next)
}

fn records(items: Box<[Item]>) -> Box<[Record]> {
    let mut records = List::with_capacity(u32::try_from(items.len()).expect("page count"));
    for item in items {
        let Item::Row(Row::Jig(row)) = item else { panic!("jig row") };
        records.push(row).expect("page count");
    }
    records.into_boxed()
}

fn untyped(event: Event) -> (Box<[Record]>, Option<Cursor>) {
    let Event::Restored { owner, rows, next } = event else { panic!("untyped page: {event:?}") };
    assert_eq!(owner, OWNER);
    let mut records = List::with_capacity(u32::try_from(rows.len()).expect("page count"));
    for row in rows {
        let Row::Jig(row) = row else { panic!("jig row") };
        records.push(row).expect("page count");
    }
    (records.into_boxed(), next)
}

#[test]
fn load_core_pages_equal_the_unselected_walk_with_the_same_cursors() {
    let mut selected = open(true);
    let mut default = open(false);
    for store in [&mut selected, &mut default] {
        fixtures::committed(
            store,
            Box::new([
                fixtures::header(1),
                fixtures::save(fixtures::person(ALICE)),
                fixtures::save(fixtures::live_task(LIVE)),
                fixtures::save(fixtures::ended_task(ENDED)),
            ]),
        );
    }
    let mut after = None;
    for _ in 0..ROWS {
        let (items, next) = page(request(&mut selected, live(), after.clone(), 2));
        let (rows, default_next) = untyped(request(&mut default, live(), after, 2));
        assert_eq!(records(items), rows);
        assert_eq!(next, default_next);
        if next.is_none() {
            return;
        }
        after = next;
    }
    panic!("live walk must end");
}

#[test]
fn census_pairs_and_result_joins_equal_the_unselected_walk() {
    let position = Record::People(jig_domain_people::Stored::EndedRead { task: ENDED, party: ALICE, through: 1 });
    let mut selected = open(true);
    let mut default = open(false);
    for store in [&mut selected, &mut default] {
        fixtures::committed(
            store,
            Box::new([
                fixtures::header(1),
                fixtures::save(fixtures::ended_task(ENDED)),
                fixtures::save(fixtures::result_index(ALICE, 1, ENDED)),
                fixtures::save(position.clone()),
            ]),
        );
    }
    let mut after = None;
    for _ in 0..ROWS {
        let (items, next) = page(request(&mut selected, census(), after.clone(), 2));
        let (rows, default_next) = untyped(request(&mut default, census(), after, 2));
        assert_eq!(records(items), rows);
        assert_eq!(next, default_next);
        if next.is_none() {
            return;
        }
        after = next;
    }
    panic!("census walk must end");
}

#[test]
fn a_missing_census_owner_is_a_link_with_both_domain_keys() {
    let mut store = open(true);
    let child = fixtures::result_index(ALICE, 1, ENDED);
    fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(child.clone())]));
    let (items, next) = page(request(&mut store, census(), None, ROWS));
    let [Item::MissingLink { record, target }] = items.as_ref() else { panic!("missing owner link") };
    assert_eq!(*record, child.key());
    assert_eq!(*target, Key::Tasks(jig_domain_tasks::Key::Ended(ENDED)));
    assert_eq!(next, None);
    let Phase::Ready { .. } = store.phase else { panic!("missing link leaves the store ready") };
}

#[test]
fn an_undecodable_row_refuses_with_its_family_and_key_and_stops() {
    let key = keys::encode(&fixtures::person(ALICE).key(), fixtures::KEY_LIMIT).expect("person key");
    let mut stores = fixtures::stores();
    fixtures::plant(
        &mut stores,
        &[
            fixtures::format_put(fixtures::DEPLOYMENT),
            fixtures::layouts_put(),
            fixtures::header_put(fixtures::DEPLOYMENT, 1),
            fixtures::installation_put(fixtures::INSTALLATION),
            skein_kv::Op::Put { key: key.clone(), value: Box::new([255]) },
        ],
    );
    let mut store = fixtures::over(stores);
    store.start_restore();
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: OWNER, root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let [Event::Opened { .. }] = events.as_ref() else { panic!("store opens before reading people") };
    let Event::PageRefused { owner, failure } = request(&mut store, live(), None, ROWS) else {
        panic!("typed refusal")
    };
    assert_eq!(owner, OWNER);
    let site = crate::restore::RowSite {
        family: Some(Family::People),
        key_bytes: u64::try_from(key.len()).expect("fixture key length"),
        key,
        truncated: false,
    };
    assert_eq!(failure, Failure { cause: Cause::Decode(DecodeFailure::Value), row: Some(site) });
    assert_eq!(store.phase, Phase::Stopped);
}

fn digest() -> SecretValue {
    SecretValue::Digest { expiry: 0, digest: [41; 32] }
}

fn raw_secret(store: &mut Store, owner: SecretOwner, number: u64) -> Option<Box<[u8]>> {
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.secrets,
        &crate::memory_env(store.limits.secrets),
        skein_kv::Request::Get {
            owner: OWNER,
            key: crate::secrets::key(owner, number, fixtures::KEY_LIMIT).expect("secret key"),
        },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty());
    let Some(skein_kv::Event::Got { value, .. }) = above.pop() else { panic!("secret map read") };
    assert!(above.is_empty());
    value
}

fn lay_secrets(store: &mut Store, numbers: &[u64]) {
    let mut ops = List::with_capacity(u32::try_from(numbers.len()).expect("fixture count"));
    for &number in numbers {
        ops.push(skein_kv::Op::Put {
            key: crate::secrets::key(SecretOwner::SignIn, number, fixtures::KEY_LIMIT).expect("secret key"),
            value: crate::secrets::codec::encode(digest(), u32::MAX).expect("digest encoding"),
        })
        .expect("fixture count");
    }
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.secrets,
        &crate::memory_env(store.limits.secrets),
        skein_kv::Request::Commit { owner: OWNER, ops: ops.into_boxed() },
        &mut above,
        &mut below,
    );
    let Some(skein_kv::Event::Committed { .. }) = above.pop() else { panic!("fixture secrets planted") };
    assert!(above.is_empty() && below.is_empty());
}

fn people(store: &mut Store) {
    let (items, next) = page(request(store, Restore::Jig(Box::new([RestoreRange::People])), None, ROWS));
    assert!(!items.is_empty());
    assert_eq!(next, None);
}

fn fire(store: &mut Store) {
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::fire(store, &fixtures::env(), &mut above, &mut below);
    assert!(above.is_empty());
    assert!(below.store.is_empty() && below.secrets.is_empty() && below.transcripts.is_empty());
}

#[test]
fn load_core_arms_one_sweep_and_a_second_people_walk_never_restarts_it() {
    let mut store = open(true);
    lay_secrets(&mut store, &[4, 5, 6]);
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(Record::People(jig_domain_people::Stored::SignIn {
                number: 6,
                person: ALICE,
                expires: Wall::EPOCH,
            })),
            fixtures::save(fixtures::person(ALICE)),
        ]),
    );
    store.limits.secrets.page.rows = 1;
    let (_, next) = page(request(&mut store, live(), None, ROWS));
    assert_eq!(next, None);
    assert!(!store.mark_durable());
    people(&mut store);
    fire(&mut store);
    let encoded = crate::secrets::codec::encode(digest(), u32::MAX).expect("digest encoding");
    for number in [4, 5, 6] {
        assert_eq!(raw_secret(&mut store, SecretOwner::SignIn, number), Some(encoded.clone()));
    }
    fixtures::committed(&mut store, Box::new([fixtures::header(2), Write::Wires]));
    assert!(store.mark_durable());
    fire(&mut store);
    assert_eq!(raw_secret(&mut store, SecretOwner::SignIn, 4), None);
    let last = store.stores.secrets.last();
    people(&mut store);
    assert_eq!(store.stores.secrets.last(), last, "a second walk does not run an active sweep");
    assert_eq!(raw_secret(&mut store, SecretOwner::SignIn, 5), Some(encoded.clone()));
    fire(&mut store);
    fire(&mut store);
    assert_eq!(raw_secret(&mut store, SecretOwner::SignIn, 5), None);
    assert_eq!(raw_secret(&mut store, SecretOwner::SignIn, 6), Some(encoded.clone()));
    lay_secrets(&mut store, &[7]);
    people(&mut store);
    fire(&mut store);
    assert_eq!(raw_secret(&mut store, SecretOwner::SignIn, 7), Some(encoded));
}

#[test]
#[should_panic(expected = "empty nonterminal restore page must advance its cursor")]
fn selected_pages_share_the_empty_nonterminal_progress_assertion() {
    let mut store = open(true);
    fixtures::committed(&mut store, Box::new([fixtures::header(1)]));
    request(&mut store, live(), None, 0);
}

#[test]
fn a_connector_page_keeps_its_untyped_bytes() {
    let mut store = open(true);
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(1), Write::Put { key: Box::from(b"a1".as_slice()), value: Box::new([7]) }]),
    );
    let (items, next) =
        page(request(&mut store, Restore::Application(Box::new([skein_kv::Range::prefix(b"a")])), None, ROWS));
    let [Item::Row(Row::Application { key, value })] = items.as_ref() else { panic!("connector row") };
    assert_eq!(key.as_ref(), b"a1");
    assert_eq!(value.as_ref(), &[7]);
    assert_eq!(next, None);
}

#[test]
fn the_sweep_waits_for_the_wires_commits_disk_terminal() {
    let mut disk = fixtures::disk::Disk::new(&[]);
    {
        let mut store = disk.store_with_chunk(4096);
        disk.open(&mut store);
        disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1), Write::Wires])));
        let events =
            disk.request(&mut store, Request::Secrets { owner: OWNER, request: fixtures::sign_in_secret(4, 41) });
        let [Event::Kept { .. }] = events.as_ref() else { panic!("orphan digest durable") };
    }
    disk.crash(2);
    let mut store = disk.store_with_chunk(4096);
    store.start_restore();
    let events = disk.open(&mut store);
    let [Event::Opened { .. }] = events.as_ref() else { panic!("selected reopen") };
    let (_, next) = page(request(&mut store, live(), None, ROWS));
    assert_eq!(next, None);
    let before = raw_secret(&mut store, SecretOwner::SignIn, 4).expect("orphan survives open and load");
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(2, Box::new([fixtures::header(2), Write::Wires])),
        &mut above,
        &mut below,
    );
    assert_eq!(store.pending_mark(), Some(2));
    assert!(!store.mark_durable());
    assert!(above.is_empty());
    crate::fire(&mut store, &fixtures::env(), &mut above, &mut below);
    assert!(below.secrets.is_empty(), "no secrets erase before the wires terminal");
    assert_eq!(raw_secret(&mut store, SecretOwner::SignIn, 4), Some(before));
    disk.drive(&mut below, |io, below| crate::up(&mut store, &fixtures::env(), io, &mut above, below));
    let Some(Event::Committed { number: 2, .. }) = above.pop() else { panic!("wires durable") };
    assert!(above.is_empty());
    assert!(store.mark_durable());
    crate::fire(&mut store, &fixtures::env(), &mut above, &mut below);
    disk.drive(&mut below, |io, below| crate::up(&mut store, &fixtures::env(), io, &mut above, below));
    assert_eq!(raw_secret(&mut store, SecretOwner::SignIn, 4), None);
    assert!(above.is_empty());
}
