use jig_protocol_store::{Check, Counts, Older, OpenRefusal, Owner, PatchSet, PatchSets, Write};
use jig_store_world::{
    patches::{self, BASE, Records, SETS},
    root,
};

#[test]
fn old_records_patch_in_order_and_the_root_continues() {
    let records = Records::story();
    let mut world = records.committed(901, BASE);
    let original = patches::source(&mut world);
    world.patch_sets(SETS);
    let last = world.open_switched().expect("patched open");
    assert_eq!(
        world.patched(),
        [
            (Owner::Application, 1, Counts { records: 5, commits: 2 }),
            (Owner::Jig, 1, Counts { records: 2, commits: 1 }),
            (Owner::Application, 2, Counts { records: 6, commits: 3 }),
            (Owner::Jig, 2, Counts { records: 0, commits: 0 }),
        ]
    );
    assert_eq!(last, 6); // Initial commit, four store patch pages and the seal; two secrets pages have their own numbers.
    patches::whole(&mut world);
    assert_eq!(
        patches::counts(&mut world),
        [
            (b"fixture-label".to_vec(), 2, 2),
            (b"fixture-shelf".to_vec(), 3, 3),
            (b"fixture-token".to_vec(), 3, 3),
            (b"fixture-note".to_vec(), 2, 2)
        ]
    );
    patches::kept(&original, &patches::source(&mut world), &"story");
    records.check(&mut world, 2, &"story");
    world.crash(902);
    // The fixture check committed this start's mark after the patch seal.
    assert_eq!(world.open_switched(), Some(last + 1));
    assert!(world.patched().is_empty());
    assert_eq!(world.active(), b"2\n");
    assert_eq!(world.commit(Box::new([Write::Save(root::header(last + 2)), Write::Wires])), Some(last + 2));
    world.crash(903);
    assert_eq!(world.reopen().last, last + 2);
    records.check(&mut world, 2, &"story");
}

#[test]
fn level_and_set_refusals_leave_every_file_as_it_was() {
    const BAD: PatchSets =
        PatchSets { sets: &[PatchSet { patches: &["patch 1\nowner other 1\n"], ..SETS.sets[0] }, SETS.sets[1]] };
    let records = Records::story();
    for (made, version, opened, expected) in [
        (SETS, 2, BASE, OpenRefusal::Ahead { owner: Owner::Jig, found: Some(2), wanted: Some(0) }),
        (PatchSets::NONE, 1, SETS, OpenRefusal::OlderBuild { why: Older::Unleveled }),
        (BASE, 1, BAD, OpenRefusal::Set { owner: Owner::Application, patch: 1 }),
    ] {
        let mut world = records.committed_at(904, made, version);
        world.patch_sets(opened);
        let before = world.contents();
        assert_eq!(world.open_switched(), None);
        assert_eq!(world.refusal(), Some(&expected));
        patches::kept(&before, &world.contents(), &expected);
    }
}

#[test]
fn an_empty_shelf_refuses_again_from_an_unchanged_source() {
    let mut records = Records::story();
    records.shelves[1].books = 0;
    let mut world = records.committed(905, BASE);
    let before = patches::source(&mut world);
    world.patch_sets(SETS);
    for seed in [906, 907] {
        assert_eq!(world.open_switched(), None);
        let Some(OpenRefusal::Patch { owner, patch, key, halt }) = world.refusal() else {
            panic!("typed patch refusal: {:?}", world.refusal())
        };
        assert_eq!((*owner, *patch, key.as_ref()), (Owner::Application, 2, b"s/1".as_slice()));
        assert_eq!(*halt, skein_codec::patch::Halt::Refused(jig_test_patches::patches::refusal::p0002::EMPTY));
        assert_eq!(world.active(), b"1\n");
        patches::kept(&before, &patches::source(&mut world), &seed);
        assert!(!world.paths().iter().any(|path| path == b"generations/2/failed-copy"));
        // Cleanup must remove this marker with the old failed copy.
        world.lay(&[skein_fake_machine::Item::file(b"generations/2/failed-copy", b"remove on retry")]);
        world.crash(seed);
    }
    // The second attempt removed the first marker before making a fresh copy.
}

#[test]
fn every_patch_write_cut_recovers_all_records_and_no_stray() {
    let records = Records::story();
    let mut completed = records.committed(908, BASE);
    completed.patch_sets(SETS);
    completed.trace_writes();
    assert!(completed.open_switched().is_some());
    let cuts = patches::cuts(&completed);
    eprintln!("patch-phase disk-changing cuts: {}", cuts.len());
    for cut in cuts {
        let mut world = records.committed(908, BASE);
        world.patch_sets(SETS);
        world.trace_writes();
        world.cut_at(cut);
        assert_eq!(world.open_switched(), None, "cut {cut} reached");
        world.crash(u64::from(cut));
        assert!(world.open_switched().is_some(), "clean start after cut {cut}: {:?}", world.refusal());
        patches::whole(&mut world);
        records.check(&mut world, 2, &(908, cut));
    }
}

#[test]
fn each_check_refusal_leaves_the_generation_before_active() {
    const SEED: u64 = 912;
    const ENDED: u64 = root::LIVE + 1;
    // The base's token commits have become a snapshot, so lowering the commit
    // limit tests the patched put rather than refusing recovery of the log.
    let mut token_limits = patches::limits();
    token_limits.secrets.snapshot_after = 1;
    let mut token_open = token_limits;
    // 26 frame + 9 put + 10 key + 32 v1 token; v2 adds one byte.
    token_open.secrets.commit = 77;
    let mut load_limits = patches::limits();
    load_limits.core.tasks.dependencies = 0;
    let mut live = root::live_task(root::LIVE);
    let jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task)) = &mut live else { panic!("live task") };
    task.dependencies = Box::new([ENDED]);
    let read =
        jig_domain_core::Record::People(jig_domain_people::Stored::EndedRead { task: ENDED, party: 1, through: 1 });
    let key = |row: &jig_domain_core::Record| jig_protocol_store::keys::encode(&row.key(), 64).expect("fixture key");
    for (base_limits, open_limits, rows, why) in [
        (token_limits, token_open, vec![], Check::Large { key: Box::new(0_u64.to_be_bytes()) }),
        (patches::limits(), load_limits, vec![live.clone()], Check::Load { key: key(&live) }),
        (patches::limits(), patches::limits(), vec![read.clone()], Check::Join { key: key(&read) }),
    ] {
        let records = Records::story();
        let mut world = records.committed_with(SEED, BASE, 1, base_limits, &rows);
        let before = patches::source(&mut world);
        world.patch_sets(SETS);
        world.restart_with_limits(SEED, open_limits);
        let expected = OpenRefusal::Check { why };
        for seed in [SEED + 1, SEED + 2] {
            assert_eq!(world.open_switched(), None, "{expected:?}");
            assert_eq!(world.refusal(), Some(&expected));
            assert_eq!(world.active(), b"1\n");
            patches::kept(&before, &patches::source(&mut world), &expected);
            assert!(!world.paths().iter().any(|path| path == b"generations/2/check-copy"));
            world.lay(&[skein_fake_machine::Item::file(b"generations/2/check-copy", b"remove on retry")]);
            world.crash(seed);
        }
    }
}

#[test]
fn selected_open_defers_a_missing_ended_owner_and_refusing_start_keeps_the_source() {
    use jig_domain_core::{Census, Key, Record, RestoreRange};
    use jig_protocol_store::{Restore, restore::Item};
    let position = Record::People(jig_domain_people::Stored::EndedRead { task: root::LIVE + 1, party: 1, through: 1 });
    let mut world = Records::story().committed_with(913, BASE, 1, patches::limits(), core::slice::from_ref(&position));
    let before = patches::source(&mut world);
    world.patch_sets(SETS);
    world.start_restore();
    world.open().expect("selected open leaves the patched copy pending");
    assert_eq!(world.start_open_failure(), None);
    assert_eq!(world.active(), b"1\n");
    let page = world
        .restore_page(
            Restore::Jig(Box::new([RestoreRange::Census(Census { questions: Box::new([]), results: Box::new([]) })])),
            None,
            8,
            65_536,
        )
        .expect("typed census");
    let [Item::MissingLink { record, target }] = page.items.as_ref() else { panic!("missing owner: {:?}", page.items) };
    assert_eq!(*record, position.key());
    assert_eq!(*target, Key::Tasks(jig_domain_tasks::Key::Ended(root::LIVE + 1)));
    patches::kept(&before, &patches::source(&mut world), &"no switch before link");
    // A root refusing the deferred obligation makes no Switch. The next
    // process starts with default selection and retains its Join refusal.
    world.crash(914);
    assert_eq!(world.open(), None);
    assert!(matches!(world.refusal(), Some(OpenRefusal::Check { why: Check::Join { .. } })));
    assert_eq!(world.start_open_failure(), None);
    assert_eq!(world.active(), b"1\n");
    patches::kept(&before, &patches::source(&mut world), &"refused start");
}

#[test]
fn selected_open_still_refuses_an_undecodable_live_row_before_switch() {
    use jig_protocol_store::restore::{Cause, DecodeFailure, OpenFailure};
    let mut live = root::live_task(root::LIVE);
    let jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task)) = &mut live else { panic!("live") };
    task.dependencies = Box::new([root::LIVE + 1]);
    let key = jig_protocol_store::keys::encode(&live.key(), 64).expect("task key");
    let mut world = Records::story().committed_with(915, BASE, 1, patches::limits(), &[live]);
    let before = patches::source(&mut world);
    let mut lowered = patches::limits();
    lowered.core.tasks.dependencies = 0;
    world.patch_sets(SETS);
    world.restart_with_limits(916, lowered);
    world.start_restore();
    assert_eq!(world.open(), None);
    assert_eq!(world.refusal(), Some(&OpenRefusal::Check { why: Check::Load { key: key.clone() } }));
    let Some(OpenFailure::Data(failure)) = world.start_open_failure() else { panic!("typed row failure") };
    assert_eq!(failure.cause, Cause::Decode(DecodeFailure::Value));
    assert_eq!(failure.row.as_ref().expect("live row").key, key);
    assert_eq!(world.active(), b"1\n");
    patches::kept(&before, &patches::source(&mut world), &"selected decode refusal");
}

#[test]
fn selected_open_preserves_present_ended_owner_corruption_for_the_census() {
    use jig_domain_core::{Census, Record, RestoreRange};
    use jig_protocol_store::{
        Restore,
        restore::{Cause, DecodeFailure, JoinFailure},
    };
    let mut ended = root::live_task(root::LIVE + 1);
    let Record::Tasks(jig_domain_tasks::Stored::Live(mut task)) = ended else { panic!("task") };
    task.spec.words = Box::from(b"past lowered limit".as_slice());
    task.phase = jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(jig_domain_tasks::TaskResult::Report {
        words: Box::new([]),
    }));
    task.ended_at = Some(skein_lib::Wall::EPOCH);
    ended = Record::Tasks(jig_domain_tasks::Stored::Ended(task));
    let position = Record::People(jig_domain_people::Stored::EndedRead { task: root::LIVE + 1, party: 1, through: 1 });
    let mut world = Records::story().committed_with(917, BASE, 1, patches::limits(), &[ended, position]);
    let mut lowered = patches::limits();
    lowered.core.tasks.spec_bytes = 1;
    world.patch_sets(SETS);
    world.restart_with_limits(918, lowered);
    world.start_restore();
    world.open().expect("owner decoding belongs to census");
    let failure = world
        .restore_page(
            Restore::Jig(Box::new([RestoreRange::Census(Census { questions: Box::new([]), results: Box::new([]) })])),
            None,
            8,
            65_536,
        )
        .expect_err("present owner is corrupt under lowered codec");
    assert!(matches!(failure.cause, Cause::Join { failure: JoinFailure::Decode(DecodeFailure::Value), .. }));
    assert_eq!(world.active(), b"1\n");
}

#[test]
fn selected_open_keeps_the_fresh_closing_walk_and_source() {
    use jig_domain_core::{CoreKey, Key};
    use jig_store_world::patches::{FRESH_AFTER, FRESH_WINDOWS, fresh_committed, fresh_proof};
    let mut world = fresh_committed(919, FRESH_WINDOWS, false);
    let before = patches::source(&mut world);
    world.patch_sets(FRESH_AFTER);
    world.start_restore();
    world.open().expect("selected fresh copy pending");
    assert_eq!(world.active(), b"1\n");
    for (index, window) in FRESH_WINDOWS.iter().enumerate() {
        let task = root::LIVE + u64::try_from(index).expect("fixture task");
        assert_eq!(world.get(Key::Core(CoreKey::RunProof(task))), fresh_proof(task, *window, true));
    }
    patches::kept(&before, &patches::source(&mut world), &"fresh source unchanged");
    world.switch().expect("switch after fresh records restored");
    assert_eq!(world.active(), b"2\n");
}
