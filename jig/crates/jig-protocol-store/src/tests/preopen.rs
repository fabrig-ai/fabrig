//! Once-before-open selection and first causes (protocol/store.md, 10.1–10.2).
use super::{fixtures, patching::SETS};
use crate::{Event, Io, OpenRefusal, Phase, Request, Restore, Store, Write, keys, restore};
use core::panic::AssertUnwindSafe;
use skein_lib::{List, Queue, Token};
use std::panic::catch_unwind;

fn open_ask() -> Request {
    Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION }
}

fn cold() -> Store {
    fixtures::over(fixtures::stores())
}

fn late(store: &mut Store) {
    assert!(catch_unwind(AssertUnwindSafe(|| store.start_restore())).is_err());
}

#[test]
fn selection_is_explicit_once_and_configuration_and_accessors_leave_it_available() {
    let mut store = cold();
    store.patch_sets(crate::PatchSets::NONE);
    assert!(store.wires(&[]));
    assert_eq!(store.pending_mark(), None);
    assert!(!store.mark_durable());
    assert_eq!(store.start_open_failure(), None);
    assert!(!store.start_restore);
    store.start_restore();
    assert!(store.start_restore);
    late(&mut store);
    let mut disk = fixtures::disk::Disk::new(&[]);
    let mut store = disk.store_with_chunk(4096);
    store.start_restore();
    assert!(store.start_restore);
    // Directory construction and seeding are configuration too.
    assert_eq!(store.phase, Phase::Closed);
    let events = disk.open(&mut store);
    let [Event::Opened { .. }] = events.as_ref() else { panic!("opened: {events:?}") };
    late(&mut store);
}

#[test]
fn every_operational_door_consumes_before_rejection_without_selecting() {
    for phase in [Phase::Closed, Phase::Stopped] {
        for door in 0_u32..5 {
            let mut store = cold();
            store.phase = phase;
            let mut above = Queue::uncapped();
            let mut below = fixtures::below();
            match door {
                0 => {
                    fixtures::run(
                        &mut store,
                        Request::Restore {
                            owner: Token::new(1),
                            range: Restore::Jig(Box::new([])),
                            after: None,
                            rows: 1,
                            bytes: 1,
                        },
                    );
                }
                1 => crate::fire(&mut store, &fixtures::env(), &mut above, &mut below),
                2 => crate::up(
                    &mut store,
                    &fixtures::env(),
                    Io::Transcripts(crate::Arrived {
                        event: skein_io::file::Event::Closed { owner: Token::new(7) },
                        fd: None,
                    }),
                    &mut above,
                    &mut below,
                ),
                3 => {
                    let _result = restore::page(&mut store, Restore::Jig(Box::new([])), None, 1, 1);
                }
                4 => {
                    let _result = store.retained_page(None, 1, 1);
                }
                _ => unreachable!("five operational doors"),
            }
            assert!(store.consumed, "door {door}, phase {phase:?}");
            assert!(!store.start_restore);
            late(&mut store);
        }
    }
    let mut store = cold();
    // An Open rejected while already stopped also consumes before returning.
    store.phase = Phase::Stopped;
    fixtures::run(
        &mut store,
        Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    assert!(store.consumed);
    late(&mut store);
}

#[test]
fn selection_requires_closed_idle_and_no_directory_root() {
    for condition in 0_u32..3 {
        let mut store = cold();
        match condition {
            0 => store.phase = Phase::Ready { next: 1, format_due: true },
            1 => store.directory.opening = crate::directory::Opening::Locking,
            2 => store.directory.root = Some(fixtures::ROOT),
            _ => unreachable!("three preconditions"),
        }
        late(&mut store);
    }
}

fn installation(bytes: Option<Box<[u8]>>, selected: bool) -> Store {
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
    if selected {
        store.start_restore();
    }
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
    let mut missing = installation(None, true);
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
    late(&mut missing);
    let corrupt = installation(Some(Box::new([255])), true);
    assert_eq!(
        corrupt.start_open_failure(),
        Some(&restore::OpenFailure::Data(restore::Failure {
            cause: restore::Cause::Decode(restore::DecodeFailure::Value),
            row: Some(restore::site(&corrupt, &key)),
        }))
    );
    assert_eq!(installation(None, false).start_open_failure(), None);
    assert_eq!(installation(Some(Box::new([255])), false).start_open_failure(), None);
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
    store.start_restore();
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let why = OpenRefusal::Numbers { store: 1, header: 2 };
    let [Event::OpenRefused { why: found, .. }] = events.as_ref() else { panic!("numbers refusal") };
    assert_eq!(*found, why);
    assert_eq!(store.start_open_failure(), Some(&restore::OpenFailure::Refused(why)));
    let mut store = cold();
    store.start_restore();
    for _ in 0_u32..2 {
        fixtures::run(
            &mut store,
            Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
        );
    }
    assert_eq!(store.start_open_failure(), Some(&restore::OpenFailure::Refused(OpenRefusal::Failed)));
    late(&mut store);
}

#[test]
fn actual_changed_counts_are_kept_only_the_default_judges_equality() {
    for selected in [false, true] {
        let mut store = cold();
        if selected {
            store.start_restore();
        }
        fixtures::run(
            &mut store,
            Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
        );
        store.patch_sets(SETS);
        let key: Box<[u8]> = Box::from(b"l/0".as_slice());
        fixtures::committed(
            &mut store,
            Box::new([fixtures::header(1), Write::Put { key: key.clone(), value: Box::new([1]) }]),
        );
        let before = crate::patching::check::before(&mut store).expect("before counts");
        fixtures::committed(&mut store, Box::new([fixtures::header(2), Write::Remove { key }]));
        let counts = crate::patching::check::counts(&mut store, before);
        if selected {
            let counts = counts.expect("counts retained without judgment");
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
        } else {
            assert_eq!(
                counts,
                Err(OpenRefusal::Check { why: crate::Check::Count { family: "fixture-label", before: 1, after: 0 } })
            );
        }
    }
}

#[test]
fn a_real_get_refusal_keeps_its_category_and_bounded_site() {
    let mut store = cold();
    store.start_restore();
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
    store.start_restore();
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
fn an_in_progress_open_never_reopens_selection_or_clears_its_first_failure() {
    let disk = fixtures::disk::Disk::new(&[]);
    let mut store = disk.store_with_chunk(4096);
    store.start_restore();
    assert!(fixtures::run(&mut store, open_ask()).is_empty());
    late(&mut store);
    let events = fixtures::run(&mut store, open_ask());
    let [Event::OpenRefused { why: OpenRefusal::Failed, .. }] = events.as_ref() else { panic!("in-progress refusal") };
    let expected = restore::OpenFailure::Refused(OpenRefusal::Failed);
    assert_eq!(store.start_open_failure(), Some(&expected));
    assert!(fixtures::run(&mut store, open_ask()).is_empty());
    assert_eq!(store.start_open_failure(), Some(&expected));
    late(&mut store);
}

#[test]
fn a_runtime_get_failure_does_not_become_an_open_failure() {
    let mut store = cold();
    store.start_restore();
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
