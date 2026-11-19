//! Restore fetch diagnostics name the history place, not a rowless value.
use super::*;
use crate::restore::{Cause, Failure, HistoryFailure};

const TASK: u64 = 4097;

fn setup() -> Store {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(fixtures::ended_task(TASK)),
            fixtures::save(fixtures::activity_line(TASK, 1)),
            fixtures::save(fixtures::result_index(ALICE, 1, TASK)),
        ]),
    );
    store
}
fn request() -> Request {
    Request::Restore {
        owner: Token::new(7),
        range: Restore::Jig(Box::new([core::RestoreRange::Census(core::Census {
            results: Box::new([core::ReadResults { party: ALICE, after: 0 }]),
            questions: Box::new([]),
        })])),
        after: None,
        rows: 4,
        bytes: 65_536,
    }
}
fn place() -> Box<[u8]> {
    keys::encode_own(&keys::Own::History { task: TASK }, fixtures::KEY_LIMIT).unwrap()
}
fn refused(store: &Store, events: &[Event], check: HistoryFailure) {
    let [Event::PageRefused { owner, failure }] = events else { panic!("restore fetch refusal: {events:?}") };
    assert_eq!(*owner, Token::new(7));
    assert_eq!(
        *failure,
        Failure::at(Cause::History { task: TASK, failure: check }, crate::restore::site(store, &place()))
    );
    assert_eq!(store.phase, Phase::Stopped);
}
#[test]
fn a_restore_fetch_with_an_extent_but_no_place_names_the_task() {
    let mut store = setup();
    assert!(fixtures::get_kv(&mut store, &logs::extent_key(logs::history(TASK).unwrap())).is_some());
    raw_commit(&mut store, Box::new([skein_kv::Op::Erase { key: place() }]));
    let events = fixtures::run(&mut store, request());
    refused(&store, &events, HistoryFailure::PlaceMissing);
}
#[test]
fn a_restore_fetch_with_a_malformed_extent_names_the_task() {
    let mut stores = fixtures::stores();
    let index = fixtures::result_index(ALICE, 1, TASK);
    fixtures::plant(
        &mut stores,
        &[
            fixtures::format_put(fixtures::DEPLOYMENT),
            fixtures::header_put(fixtures::DEPLOYMENT, 1),
            fixtures::installation_put(fixtures::INSTALLATION),
            skein_kv::Op::Put {
                key: keys::encode(&index.key(), fixtures::KEY_LIMIT).unwrap(),
                value: codec::encode(&index, &fixtures::codec_limits(), &[]).unwrap(),
            },
            skein_kv::Op::Put { key: logs::extent_key(logs::history(TASK).unwrap()), value: Box::new([0]) },
        ],
    );
    let mut store = fixtures::over(stores);
    let opened = fixtures::run(
        &mut store,
        Request::Open {
            purpose: crate::Purpose::Serve,
            owner: Token::new(1),
            root: fixtures::ROOT,
            installation: fixtures::INSTALLATION,
        },
    );
    let [Event::Opened { .. }, Event::Census { .. }] = opened.as_ref() else { panic!("open: {opened:?}") };
    let events = fixtures::run(&mut store, request());
    refused(&store, &events, HistoryFailure::ExtentMalformed);
}

#[test]
fn a_restore_frame_past_the_fetch_page_names_the_task() {
    let mut store = setup();
    store.limits.store.page.bytes = 80;
    let events = fixtures::run(&mut store, request());
    refused(&store, &events, HistoryFailure::FramePastPage);
}
#[test]
fn a_restore_log_the_generation_cannot_give_names_the_task() {
    use fixtures::disk::Disk;
    let mut disk = Disk::new(&[]);
    let mut store = disk.store();
    let opened = disk.open(&mut store);
    let [Event::Opened { .. }, Event::Census { .. }] = opened.as_ref() else { panic!("open: {opened:?}") };
    let events = disk.request(
        &mut store,
        fixtures::commit(
            1,
            Box::new([
                fixtures::header(1),
                fixtures::save(fixtures::ended_task(TASK)),
                fixtures::save(fixtures::result_index(ALICE, 1, TASK)),
            ]),
        ),
    );
    let [Event::Committed { .. }] = events.as_ref() else { panic!("commit: {events:?}") };
    let mut items = disk.items();
    let before = items.len();
    items.retain(|item| !item.path.ends_with(b"logs/0000000000000002/0000000000002002"));
    assert_eq!(items.len().checked_add(1).unwrap(), before, "remove only the history file");
    let mut disk = Disk::new(&items);
    let mut store = disk.store();
    let opened = disk.open(&mut store);
    let [Event::Opened { .. }, Event::Census { .. }] = opened.as_ref() else { panic!("open: {opened:?}") };
    let events = disk.request(&mut store, request());
    refused(&store, &events, HistoryFailure::Unread);
}
