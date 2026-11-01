//! Adversarial promises at open, through the existing store world only.
use jig_protocol_store::{Older, OpenRefusal, Owner, PatchSet, PatchSets, Write};
use jig_store_world::{
    World,
    patches::{self, BASE, Records, SETS},
    root,
};
use skein_fake_machine::Item;

const SEED: u64 = 0x5141;
const UNPATCHED: u16 = 1;
const PATCHED: u16 = 2;
// The fixture has two application patches, two jig patches (the second patches fixture-frame alone, which the world's
// sets leave out), and two-put pages where a test lowers the journal's writes.
const APPLICATION_LEVEL: u32 = 2;
const JIG_LEVEL: u32 = 2;
const PAGE_ROWS: u64 = 2;
const APP_ONLY: PatchSets = PatchSets { sets: &[SETS.sets[0]] };
const JIG_ONLY: PatchSets = PatchSets { sets: &[BASE.sets[1]] };
const JIG_APPLIED: PatchSets = PatchSets { sets: &[BASE.sets[0], SETS.sets[1]] };
const MID: PatchSets = PatchSets {
    sets: &[
        PatchSet {
            families: &[SETS.sets[0].families[0], BASE.sets[0].families[1]],
            patches: &[jig_test_patches::patches::PATCHES[0]],
            ..SETS.sets[0]
        },
        BASE.sets[1],
    ],
};
const WRONG_OWNER: PatchSets = PatchSets {
    sets: &[
        PatchSet {
            patches: &[
                "patch 1\nowner stranger 1 after fixture-jig 0\n\nfamily fixture-label 1 to 2\nadd Label.color option u8 none\n",
            ],
            ..SETS.sets[0]
        },
        SETS.sets[1],
    ],
};
const PIN_TOO_FAR: PatchSets = PatchSets {
    sets: &[
        PatchSet {
            patches: &[
                "patch 1\nowner fixture 1 after fixture-jig 3\n\nfamily fixture-label 1 to 2\nadd Label.color option u8 none\n",
            ],
            ..SETS.sets[0]
        },
        SETS.sets[1],
    ],
};

fn pending(records: &Records) -> World {
    let mut world = records.committed(SEED, BASE);
    world.patch_sets(SETS);
    world
}

fn second_generation() -> World {
    let mut world = Records::story().committed(SEED, BASE);
    world.patch_sets(MID);
    world.open_switched().expect("intermediate upgrade opens");
    world.crash(SEED);
    world.patch_sets(SETS);
    world
}

fn stray(world: &mut World) {
    world.lay(&[Item::directory(b"generations/9"), Item::file(b"generations/9/sentinel", b"never discard on refusal")]);
}

#[test]
fn pinned_order_reports_exact_walk_counts() {
    // store.md 10.2 step 4 and 10.3: each page is one commit within skein-kv's bounds (5.4), in the plan's pinned order.
    let mut limits = patches::limits();
    limits.commits.writes = u32::try_from(PAGE_ROWS).expect("two-put page");
    let mut world = World::unopened_observed(SEED, limits, None);
    world.patch_sets(BASE);
    world.open_switched().expect("first start");
    // A page holds no more puts than a commit's writes, so the source goes in one record a commit, beside its header.
    for (number, (key, value)) in (1..).zip(Records::story().rows(UNPATCHED)) {
        let writes = [Write::Save(root::header(number)), Write::Put { key, value }];
        assert_eq!(world.commit(Box::new(writes)), Some(number), "source commit");
    }
    world.crash(SEED);
    world.patch_sets(SETS);
    world.open_switched().expect("upgrade opens");
    let progress: Vec<_> =
        world.patched().iter().map(|(owner, patch, count)| (*owner, *patch, count.records, count.commits)).collect();
    let app_first = u64::from(patches::STORY_LABELS + patches::STORY_SHELVES);
    let jig = u64::from(patches::STORY_NOTES);
    let app_last = u64::from(patches::STORY_SHELVES);
    assert_eq!(
        progress,
        [
            (Owner::Application, 1, app_first, app_first.div_ceil(PAGE_ROWS)),
            (Owner::Jig, 1, jig, jig.div_ceil(PAGE_ROWS)),
            (Owner::Application, 2, app_last, app_last.div_ceil(PAGE_ROWS)),
            // Jig's second patch walks fixture-frame alone, which the world's sets leave out.
            (Owner::Jig, 2, 0, 0),
        ]
    );
}

#[test]
fn written_keys_and_held_values_survive_paging() {
    // store.md 10.3: the shell sets the record key in Context, and held bytes are patched by their own family.
    let mut records = Records::story();
    records.shelves[0].key = b"s/a".to_vec();
    records.shelves[1].key = b"s/a-much-longer-key".to_vec();
    records.shelves[2].key = b"s/z".to_vec();
    records.shelves[1].caption = b"held across a page".to_vec();
    let mut world = pending(&records);
    world.open_switched().expect("upgrade opens");
    records.check(&mut world, PATCHED, &"key context and nested label");
}

#[test]
fn wrong_record_version_refuses_with_its_key() {
    // store.md 10.2 step 4: a record at another version refuses Patch with its key.
    let records = Records::story();
    let key = b"l/1";
    let replacement = records.rows(PATCHED).get(key.as_slice()).expect("modeled label").clone();
    let mut writes = records.writes(UNPATCHED);
    for write in &mut writes {
        if let Write::Put { key: found, value } = write
            && found.as_ref() == key
        {
            *value = replacement.clone();
        }
    }
    let mut world = World::unopened_observed(SEED, patches::limits(), None);
    world.patch_sets(BASE);
    world.open_switched().expect("first start");
    world.commit(writes).expect("source commit");
    world.crash(SEED);
    world.patch_sets(SETS);
    let before = patches::source(&mut world);
    assert_eq!(world.open_switched(), None);
    assert!(
        matches!(world.refusal(), Some(OpenRefusal::Patch { owner: Owner::Application, patch: 1, key: refused, .. }) if refused.as_ref() == key),
        "{:?}",
        world.refusal()
    );
    assert_eq!(world.active(), b"1\n");
    patches::kept(&before, &patches::source(&mut world), &"refused record version");
}

#[test]
fn written_refusal_repeats_and_old_binary_reopens() {
    // store.md 10.2: every start refuses the same record, and the build before opens the source as it was.
    let mut records = Records::story();
    records.shelves[2].books = 0;
    let mut world = pending(&records);
    let original = patches::source(&mut world);
    assert_eq!(world.open_switched(), None);
    let refusal = world.refusal().expect("written refusal").clone();
    assert!(
        matches!(&refusal, OpenRefusal::Patch { owner: Owner::Application, patch: 2, key, .. } if key.as_ref() == b"s/2"),
        "{refusal:?}"
    );
    assert_eq!(world.active(), b"1\n");
    let after_refusal = patches::source(&mut world);
    world.crash(SEED);
    assert_eq!(world.open_switched(), None);
    assert_eq!(world.refusal(), Some(&refusal));
    world.crash(SEED);
    world.patch_sets(BASE);
    assert_eq!(world.open_switched(), Some(1));
    records.check(&mut world, UNPATCHED, &"old binary rollback");
    assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/2")));
    patches::kept(&original, &after_refusal, &"written refusal");
}

#[test]
fn jig_baseline_does_not_hide_unleveled_application() {
    // store.md 5.3: Unleveled includes another owner's none in a row whose jig level is set.
    let mut world = Records::story().committed(SEED, JIG_ONLY);
    world.patch_sets(BASE);
    let before = world.contents();
    assert_eq!(world.open_switched(), None);
    assert_eq!(world.refusal(), Some(&OpenRefusal::OlderBuild { why: Older::Unleveled }));
    patches::kept(&before, &world.contents(), &"unleveled application");
}

#[test]
fn manifest_is_opaque_and_unchanged_at_open() {
    // store.md 2: nothing at open reads the manifest; it is the operator's record.
    let mut world = World::unopened(SEED);
    world.lay(&[
        Item::directory(b"generations"),
        Item::directory(b"generations/1"),
        Item::directory(b"generations/1/store"),
        Item::directory(b"generations/1/secrets").mode(0o700),
        Item::file(b"generations/1/manifest", b"deliberately not a manifest"),
        Item::file(b"active", b"1\n"),
    ]);
    assert_eq!(world.open_switched(), Some(0));
    assert_eq!(
        world.contents().get(b"generations/1/manifest".as_slice()).and_then(Option::as_deref),
        Some(b"deliberately not a manifest".as_slice())
    );
}

#[test]
fn stray_cleanup_restarts_listing_and_keeps_previous() {
    // store.md 10.1 step 5: directories are listed again from their first page until empty; active and previous stay.
    const STRAY_FILES: u32 = 35; // More than the world's 32-entry directory page.
    let mut world = pending(&Records::story());
    world.open_switched().expect("upgrade opens");
    world.crash(SEED);
    let previous = patches::source(&mut world);
    world.lay(&[Item::directory(b"generations/9"), Item::directory(b"generations/9/deep")]);
    for n in 0..STRAY_FILES {
        world.lay(&[Item::file(format!("generations/9/deep/{n:02}").as_bytes(), b"stray")]);
    }
    world.lay(&[
        Item::file(b"generations/dropped", b"stray file"),
        Item::file(b"active.skein-0123456789abcdef", b"2\n"),
    ]);
    world.open_switched().expect("equal-level restart");
    patches::whole(&mut world);
    assert_eq!(patches::source(&mut world), previous);
    assert!(world.patched().is_empty());
}

#[test]
fn pin_already_passed_in_store_refuses_before_copy() {
    // store.md 10.3: after naming an upstream level already passed in the store refuses the set.
    let mut records = Records::story();
    records.notes.clear(); // The upstream is at level 1 with an empty family; application rows remain at version 1.
    let mut world = records.committed(SEED, JIG_APPLIED);
    world.patch_sets(SETS);
    let before = world.contents();
    assert_eq!(world.open_switched(), None);
    assert_eq!(world.refusal(), Some(&OpenRefusal::Set { owner: Owner::Application, patch: 1 }));
    patches::kept(&before, &world.contents(), &"already passed pin");
}

#[test]
fn first_start_recovers_after_every_disk_mutation() {
    // store.md 10.4: after each first-start write, sync and rename the next start opens one whole generation.
    let mut trace = World::unopened(SEED);
    trace.trace_writes();
    trace.open_switched().expect("traced first start");
    for &(operation, change) in trace.changes() {
        let mut world = World::unopened(SEED);
        world.trace_writes();
        world.cut_at(operation);
        let _ = world.open_switched();
        world.crash(SEED);
        assert_eq!(world.open_switched(), Some(0), "cut {operation} {change:?}: {:?}", world.refusal());
        assert_eq!(world.active(), b"1\n", "cut {operation} {change:?}");
        let paths = world.paths();
        assert!(paths.iter().any(|path| path == b"generations/1/manifest"));
        assert!(paths.iter().any(|path| path == b"transcripts"));
        assert!(!paths.iter().any(|path| path.windows(b".skein-".len()).any(|part| part == b".skein-")));
    }
}

#[test]
fn drop_copy_and_switch_recover_after_every_mutation() {
    // store.md 10.4: a cut during the previous drop, copy, patch or switch opens the old whole generation or the new.
    let mut trace = second_generation();
    trace.trace_writes();
    let expected_last = trace.open_switched().expect("traced second upgrade");
    for &(operation, change) in trace.changes() {
        let mut world = second_generation();
        let source: std::collections::BTreeMap<_, _> =
            world.contents().into_iter().filter(|(path, _)| path.starts_with(b"generations/2/")).collect();
        world.trace_writes();
        world.cut_at(operation);
        let _ = world.open_switched();
        world.crash(SEED);
        assert_eq!(world.open_switched(), Some(expected_last), "cut {operation} {change:?}: {:?}", world.refusal());
        assert_eq!(world.active(), b"3\n", "cut {operation} {change:?}");
        Records::story().check(&mut world, PATCHED, &(operation, change));
        let after = world.contents();
        for (path, bytes) in &source {
            assert_eq!(after.get(path), Some(bytes), "source changed at {operation}: {path:?}");
        }
        assert!(
            !after.keys().any(|path| path.starts_with(b"generations/1/") || path.starts_with(b"generations/dropped"))
        );
    }
}

#[test]
fn ahead_owners_precede_cleanup_and_missing_baselines() {
    for (made, owner, level) in [(SETS, Owner::Jig, JIG_LEVEL), (APP_ONLY, Owner::Application, APPLICATION_LEVEL)] {
        let mut world = Records::story().committed_at(SEED, made, PATCHED);
        world.patch_sets(BASE);
        stray(&mut world);
        let before = world.contents();
        assert_eq!(world.open_switched(), None);
        assert_eq!(world.refusal(), Some(&OpenRefusal::Ahead { owner, found: Some(level), wanted: Some(0) }));
        patches::kept(&before, &world.contents(), &owner);
    }
}

#[test]
fn invalid_sets_refuse_before_copy_or_stray_cleanup() {
    const DUPLICATE: PatchSets = PatchSets { sets: &[SETS.sets[0], SETS.sets[0], SETS.sets[1]] };
    for (label, sets, patch) in
        [("wrong owner", WRONG_OWNER, 1), ("pin past binary", PIN_TOO_FAR, 1), ("duplicate owner", DUPLICATE, 0)]
    {
        let mut world = pending(&Records::story());
        world.patch_sets(sets);
        stray(&mut world);
        let before = world.contents();
        assert_eq!(world.open_switched(), None, "{label}");
        assert_eq!(world.refusal(), Some(&OpenRefusal::Set { owner: Owner::Application, patch }), "{label}");
        patches::kept(&before, &world.contents(), &label);
    }
}
