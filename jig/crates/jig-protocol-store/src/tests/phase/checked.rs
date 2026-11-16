//! One verdict belongs to one open, through its copy and reopen (protocol/store/opening.md, 3).
use super::*;
use crate::{Owner, PatchSets};

fn registered() -> PatchSets {
    let set = value_types::set(Owner::Application, value_types::family(Owner::Application), &[]);
    PatchSets { sets: Box::leak(Box::new([set])) }
}

fn once_and_dropped(store: &Store) {
    assert_eq!(crate::patching::plan::counting::get(), 1, "the whole sets check runs once in this open");
    assert!(store.directory.checked.is_none(), "a settled open drops its verdict");
}

#[test]
fn first_start_checks_registered_sets_once() {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(registered());
    assert_opened(&disk.open(&mut store), 0);
    once_and_dropped(&store);
}

fn committed(sets: PatchSets) -> Vec<Item> {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(sets);
    assert_opened(&disk.open(&mut store), 0);
    let events = disk.request(&mut store, fixtures::commit(1, Box::new([fixtures::header(1)])));
    assert!({ if let [Event::Committed { number: 1, .. }] = events.as_ref() { true } else { false } }, "{events:?}");
    disk.items()
}

#[test]
fn patch_copy_and_reopen_share_one_set_check() {
    let mut disk = Disk::new(&committed(BASE));
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(SETS);
    let events = disk.open(&mut store);
    assert!(
        events.iter().any(|event| if let Event::Opened { pending: true, last: 2, .. } = event { true } else { false }),
        "{events:?}"
    );
    let key = crate::keys::encode_own(&crate::keys::Own::Format, store.limits.store.key).expect("format key");
    let bytes = crate::open::get(&mut store, Token::new(1), key).expect("format read").expect("format row");
    let format = crate::codec::component::Format::decode(&bytes).expect("format");
    assert_eq!(format.levels, crate::levels(&SETS), "the copy reopened at the binary's levels");
    assert!(store.directory.patch.is_none(), "the copy's check consumed the patch phase");
    once_and_dropped(&store);
}

#[test]
fn seeded_serving_open_checks_sets_once() {
    let sets = registered();
    let mut disk = Disk::new(&committed(sets));
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(sets);
    let events = disk.open(&mut store);
    assert_opened(&events, 1);
    assert!(
        { if let Some(Event::Opened { pending: false, .. }) = events.first() { true } else { false } },
        "{events:?}"
    );
    assert!(store.registry.batch(&store.limits).expect("registry batch").is_empty());
    once_and_dropped(&store);
}

#[test]
fn rehearsal_fresh_and_the_open_share_one_set_check() {
    let app = SETS.sets[0];
    let marked = Box::leak(app.patches[0].replacen("\n\n", "\nmark fixture-fresh\n\n", 1).into_boxed_str());
    let sets = PatchSets {
        sets: Box::leak(Box::new([
            crate::PatchSet {
                patches: Box::leak(Box::new([marked, app.patches[1]])),
                fresh: Some("fixture-fresh"),
                ..app
            },
            SETS.sets[1],
        ])),
    };
    let mut disk = Disk::new(&committed(BASE));
    let before = disk.contents();
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(sets);
    let scratch = disk.scratch_root();
    let directory = disk.root();
    let events = disk.request(
        &mut store,
        Request::Rehearse { owner: Token::new(1), directory, scratch, installation: fixtures::INSTALLATION },
    );
    assert!(
        events.iter().any(|event| if let Event::Rehearsing { fresh: true, .. } = event { true } else { false }),
        "{events:?}"
    );
    assert!(events.iter().any(|event| if let Event::Opened { .. } = event { true } else { false }), "{events:?}");
    once_and_dropped(&store);
    assert_eq!(disk.contents(), before, "the source is unchanged");
}

#[test]
fn changed_sets_get_a_new_verdict_after_serving_and_refusal() {
    const LONG: &str = "fixture-family-name-with-sixty-five-bytes-past-the-manifest-bound";
    let good = registered();
    assert_eq!(LONG.len(), 65);
    let original = good.sets[0];
    let bad = PatchSets {
        sets: Box::leak(Box::new([crate::PatchSet {
            families: Box::leak(Box::new([crate::KeptFamily { name: LONG, ..original.families[0] }])),
            ..original
        }])),
    };
    let mut store = fixtures::over(fixtures::stores());
    for (sets, accepted) in [(good, true), (bad, false), (good, true)] {
        // The memory fixture has no shell teardown. Keep the Store and its
        // directory, returning only its phase to the next start's entrance.
        store.phase = Phase::Closed;
        store.patch_sets(sets);
        let events = fixtures::run(
            &mut store,
            Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
        );
        if accepted {
            assert_opened(&events, 0);
        } else {
            let [Event::OpenRefused { why, .. }] = events.as_ref() else { panic!("{events:?}") };
            assert_eq!(*why, OpenRefusal::Set { owner: Owner::Application, patch: 0 });
        }
        once_and_dropped(&store);
    }
}
