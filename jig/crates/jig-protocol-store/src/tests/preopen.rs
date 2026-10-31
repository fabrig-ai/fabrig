//! Open failures and first causes (protocol/store.md, 10.1–10.2).
use super::{fixtures, patching::SETS};
use crate::{Event, OpenRefusal, Request, Restore, Store, Write, keys, restore};
use skein_lib::{List, Queue, Token};

fn open_ask() -> Request {
    Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION }
}

fn cold() -> Store {
    fixtures::over(fixtures::stores())
}

fn installation(bytes: Option<Box<[u8]>>) -> Store {
    let mut stores = fixtures::stores();
    let mut rows = List::with_capacity(4);
    for row in [
        fixtures::format_put(fixtures::DEPLOYMENT),
        fixtures::layouts_put(),
        fixtures::header_put(fixtures::DEPLOYMENT, 1),
    ] {
        rows.push(row).expect("component rows");
    }
    if let Some(value) = bytes {
        rows.push(skein_kv::Op::Put { key: keys::installation(fixtures::KEY_LIMIT).expect("installation key"), value })
            .expect("installation room");
    }
    fixtures::plant(&mut stores, rows.as_slice());
    let mut store = fixtures::over(stores);
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let [Event::Corrupt { .. }] = events.as_ref() else { panic!("corrupt: {events:?}") };
    store
}

#[test]
fn missing_and_corrupt_installation_are_distinct_and_first_detail_survives_inputs() {
    let key = keys::installation(fixtures::KEY_LIMIT).expect("installation key");
    let mut missing = installation(None);
    let expected = restore::OpenFailure::Missing(restore::site(&missing, &key));
    assert_eq!(missing.start_open_failure(), Some(&expected));
    // Rejected Open/Restore, background work and synchronous readers never reset it.
    fixtures::run(&mut missing, Request::Open { owner: Token::new(2), root: fixtures::ROOT, installation: [7; 16] });
    fixtures::run(
        &mut missing,
        Request::Restore { owner: Token::new(2), range: Restore::Jig(Box::new([])), after: None, rows: 1, bytes: 1 },
    );
    let _result = restore::page(&mut missing, Restore::Jig(Box::new([])), None, 1, 1);
    let _result = missing.retained_page(None, 1, 1);
    crate::fire(&mut missing, &fixtures::env(), &mut Queue::uncapped(), &mut fixtures::below());
    assert_eq!(missing.start_open_failure(), Some(&expected));
    let corrupt = installation(Some(Box::new([255])));
    assert_eq!(
        corrupt.start_open_failure(),
        Some(&restore::OpenFailure::Data(restore::Failure {
            cause: restore::Cause::Decode(restore::DecodeFailure::Value),
            row: Some(restore::site(&corrupt, &key)),
        }))
    );
}

#[test]
fn ordinary_refusals_and_coarse_failures_remain_truthful_and_sticky() {
    let mut stores = fixtures::stores();
    fixtures::plant(
        &mut stores,
        &[
            fixtures::format_put(fixtures::DEPLOYMENT),
            fixtures::layouts_put(),
            fixtures::header_put(fixtures::DEPLOYMENT, 2),
        ],
    );
    let mut store = fixtures::over(stores);
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let why = OpenRefusal::Numbers { store: 1, header: 2 };
    let [Event::OpenRefused { why: found, .. }] = events.as_ref() else { panic!("numbers refusal") };
    assert_eq!(*found, why);
    assert_eq!(store.start_open_failure(), Some(&restore::OpenFailure::Refused(why)));
    let mut store = cold();
    for _ in 0_u32..2 {
        fixtures::run(
            &mut store,
            Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
        );
    }
    assert_eq!(store.start_open_failure(), Some(&restore::OpenFailure::Refused(OpenRefusal::Failed)));
}

#[test]
fn changed_counts_are_kept_in_the_manifest() {
    let mut store = cold();
    fixtures::run(&mut store, open_ask());
    store.patch_sets(SETS);
    let key: Box<[u8]> = Box::from(b"l/0".as_slice());
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(1), Write::Put { key: key.clone(), value: Box::new([1]) }]),
    );
    let before = crate::patching::check::before(&mut store).expect("before counts");
    fixtures::committed(&mut store, Box::new([fixtures::header(2), Write::Remove { key }]));
    let counts = crate::patching::check::counts(&mut store, before).expect("counts retained without judgment");
    let manifest = crate::codec::component::Manifest {
        generation: 2,
        from: Some(1),
        before: crate::codec::component::Levels { skein: None, smith: None, jig: None, application: None },
        after: crate::codec::component::Levels { skein: None, smith: None, jig: None, application: None },
        counts,
        store: 2,
        secrets: None,
        written: 0,
    };
    let manifest = crate::codec::component::Manifest::decode(&manifest.encode()).expect("manifest readback");
    let counts = manifest.counts;
    let label = counts.get(0).expect("label manifest count").clone().into_parts();
    assert_eq!(label.family.as_ref(), b"fixture-label");
    assert_eq!((label.before, label.after), (1, 0));
    assert_eq!(counts.len(), 4);
}

#[test]
fn a_real_get_refusal_keeps_its_category_and_bounded_site() {
    let mut store = cold();
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Close { owner: Token::new(1) },
        &mut above,
        &mut below,
    );
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let [Event::OpenRefused { why: OpenRefusal::Failed, .. }] = events.as_ref() else { panic!("old terminal mapping") };
    let site = restore::site(&store, &keys::encode_own(&keys::Own::Format, fixtures::KEY_LIMIT).expect("format key"));
    assert_eq!(
        store.start_open_failure(),
        Some(&restore::OpenFailure::Data(restore::Failure {
            cause: restore::Cause::Join {
                target: site.clone(),
                failure: restore::JoinFailure::Refused(skein_kv::Refusal::Unavailable)
            },
            row: Some(site),
        }))
    );
}

#[test]
fn a_real_patch_count_walk_refusal_keeps_its_category() {
    let mut store = cold();
    store.patch_sets(SETS);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Close { owner: Token::new(1) },
        &mut above,
        &mut below,
    );
    assert!(crate::patching::check::before(&mut store).is_none());
    assert_eq!(
        store.start_open_failure(),
        Some(&restore::OpenFailure::Data(restore::Failure {
            cause: restore::Cause::Walk(restore::WalkFailure::Refused(skein_kv::Refusal::Unavailable)),
            row: None,
        }))
    );
}

#[test]
fn an_in_progress_open_keeps_its_first_failure() {
    let disk = fixtures::disk::Disk::new(&[]);
    let mut store = disk.store_with_chunk(4096);
    assert!(fixtures::run(&mut store, open_ask()).is_empty());
    let events = fixtures::run(&mut store, open_ask());
    let [Event::OpenRefused { why: OpenRefusal::Failed, .. }] = events.as_ref() else { panic!("in-progress refusal") };
    let expected = restore::OpenFailure::Refused(OpenRefusal::Failed);
    assert_eq!(store.start_open_failure(), Some(&expected));
    assert!(fixtures::run(&mut store, open_ask()).is_empty());
    assert_eq!(store.start_open_failure(), Some(&expected));
}

#[test]
fn a_runtime_get_failure_does_not_become_an_open_failure() {
    let mut store = cold();
    fixtures::run(&mut store, open_ask());
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Close { owner: Token::new(1) },
        &mut above,
        &mut below,
    );
    assert_eq!(crate::open::get(&mut store, Token::new(1), Box::new([2])), Err(OpenRefusal::Failed));
    assert_eq!(store.start_open_failure(), None);
}
