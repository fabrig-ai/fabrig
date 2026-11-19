//! Independent checks for protocol/store/README.md, 2 and protocol/store/secrets.md and protocol/store/opening.md, 1–4 and protocol/store/README.md, 11.
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_protocol_store::{Check, Event, KeptFamily, OpenRefusal, Owner, OwnerSet, OwnerSets, SecretOwner, Secrets};
use jig_store_world::{World, patches, root};
use skein_fake_machine::Item;
use skein_lib::Secret;

pub(crate) const SEED: u64 = 401;
// fixture-token's schema bounds its payload at 64 bytes.
const TOKEN_BOUND: usize = 64;
// Manifest's family and name bounds (protocol/store/opening.md 3).
const FAMILY_BOUND: usize = 64;
const NAME_BOUND: usize = 64;
const CARRIED_FIRST: u64 = 1_u64 << 63;

fn empty() -> patches::Records {
    patches::Records { labels: vec![], shelves: vec![], notes: vec![], tokens: vec![] }
}

fn manifest(world: &mut World, generation: u64) -> jig_protocol_store::codec::component::Manifest {
    let contents = world.contents();
    let path = format!("generations/{generation}/manifest").into_bytes();
    let bytes = contents.get(&path).expect("generation manifest").as_ref().expect("manifest file");
    jig_protocol_store::codec::component::Manifest::decode(bytes).expect("manifest decodes")
}

fn load_key(record: &core::Record) -> Box<[u8]> {
    jig_protocol_store::keys::encode(&record.key(), patches::limits().store.key).expect("fixture key fits")
}

fn refuses_check(world: &mut World, expected: &Check) {
    let before = patches::source(world);
    world.owner_sets(patches::SETS);
    for attempt in 0..2 {
        assert_eq!(world.open_switched(), None, "check attempt {attempt}");
        assert!(
            matches!(world.refusal(), Some(OpenRefusal::Check { why }) if why == expected),
            "expected {expected:?}, got {:?}",
            world.refusal()
        );
        assert_eq!(world.active(), b"1\n");
        patches::kept(&before, &patches::source(world), &attempt);
        assert!(world.paths().iter().any(|path| path == b"generations/2/store"));
        assert!(!world.paths().iter().any(|path| path == b"generations/2/manifest"));
        // A marker proves that the next start removes the refused copy.
        assert!(!world.paths().iter().any(|path| path == b"generations/2/refused-copy"));
        world.lay(&[Item::file(b"generations/2/refused-copy", b"stray")]);
        world.crash(SEED);
    }
    world.owner_sets(patches::BASE);
    assert_eq!(world.open_switched(), Some(1), "the build before opens its source");
    assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/2")));
    patches::kept(&before, &patches::source(world), &"build before");
}

#[test]
fn tokens_at_bounds_keep_numbers_across_many_pages() {
    // Two-row pages, seventeen tokens, including both ends of the drawn-number exclusion.
    const TOKENS: u64 = 17;
    let mut records = patches::Records::story();
    records.tokens = (0..TOKENS - 1).map(|n| (n, vec![b'x'; TOKEN_BOUND])).collect();
    records.tokens.push((CARRIED_FIRST - 1, vec![]));
    let mut limits = patches::limits();
    // Only the secrets map has one operation and room for a single token per page.
    limits.secrets.ops = 1;
    limits.secrets.value = 128;
    limits.secrets.page.bytes = 128;
    limits.secrets.record = 1;
    let mut world = records.committed_with(SEED, patches::BASE, limits, &[]);
    let before = patches::source(&mut world);
    world.owner_sets(patches::SETS);
    let last = world.open_switched().expect("tokens copy with their map's bounds");
    patches::whole(&mut world);
    patches::kept(&before, &patches::source(&mut world), &"tokens");
    assert_eq!(
        patches::counts(&mut world),
        vec![
            (b"fixture-label".to_vec(), 2, 2),
            (b"fixture-shelf".to_vec(), 3, 3),
            (b"fixture-token".to_vec(), TOKENS, TOKENS),
            (b"fixture-note".to_vec(), 2, 2),
        ]
    );
    let seal = manifest(&mut world, 2);
    assert_eq!(seal.store, last);
    assert_eq!(seal.secrets, Some(TOKENS));
    let core::Record::Core(core::CoreRecord::Deployment(header)) = world.get(root::header(last).key()) else {
        panic!("deployment header");
    };
    assert_eq!(header.commits, last);
    // The start's wires follow the sealing commit (protocol/store/opening.md 2, step 8).
    assert_eq!(world.mark(), Some(last + 1));
    records.check(&mut world, &"both token bounds and number bounds");
    world.crash(SEED);
    world.reopen();
    records.check(&mut world, &"equal-level restart");
}

#[test]
fn empty_families_have_zero_manifest_counts() {
    let records = empty();
    let mut world = records.committed(SEED, patches::BASE);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("empty families upgrade");
    records.check(&mut world, &"no tokens or records");
    assert_eq!(
        patches::counts(&mut world),
        vec![
            (b"fixture-label".to_vec(), 0, 0),
            (b"fixture-shelf".to_vec(), 0, 0),
            (b"fixture-token".to_vec(), 0, 0),
            (b"fixture-note".to_vec(), 0, 0),
        ]
    );
}

#[test]
fn sparse_counts_exclude_unkept_opaque_application_rows() {
    let mut records = empty();
    records.labels.push((b"l/0".to_vec(), b"kept".to_vec()));
    records.tokens.push((9, b"credential".to_vec()));
    let mut world = records.committed(SEED, patches::BASE);
    assert_eq!(world.open_switched(), Some(1));
    let key: Box<[u8]> = Box::from(b"z/opaque".as_slice());
    let value: Box<[u8]> = Box::from(b"not a versioned record".as_slice());
    let writes = Box::new([
        jig_protocol_store::Write::Save(root::header(2)),
        jig_protocol_store::Write::Put { key: key.clone(), value: value.clone() },
    ]);
    assert_eq!(world.commit(writes), Some(2));
    world.crash(SEED);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("unkept rows are copied as bytes");
    let mut expected = records.rows();
    expected.insert(key, value);
    assert_eq!(world.application(false), expected);
    assert_eq!(
        patches::counts(&mut world),
        vec![
            (b"fixture-label".to_vec(), 1, 1),
            (b"fixture-shelf".to_vec(), 0, 0),
            (b"fixture-token".to_vec(), 1, 1),
            (b"fixture-note".to_vec(), 0, 0),
        ]
    );
}

#[test]
fn first_manifest_counts_each_empty_kept_family() {
    let mut world = World::unopened_observed(SEED, patches::limits(), None);
    world.owner_sets(patches::SETS);
    assert_eq!(world.open_switched(), Some(0));
    let first = manifest(&mut world, 1);
    let counts: Vec<_> = first.counts.iter().map(|row| (row.family.to_vec(), row.before, row.after)).collect();
    assert_eq!(
        counts,
        vec![
            (b"fixture-label".to_vec(), 0, 0),
            (b"fixture-shelf".to_vec(), 0, 0),
            (b"fixture-token".to_vec(), 0, 0),
            (b"fixture-note".to_vec(), 0, 0),
        ]
    );
    assert_eq!(first.store, 0);
    assert_eq!(first.secrets, None);
}

#[test]
fn first_manifest_without_sets_has_no_counts() {
    let mut world = World::unopened_observed(SEED, patches::limits(), None);
    assert_eq!(world.open_switched(), Some(0));
    assert!(manifest(&mut world, 1).counts.is_empty());
}

#[test]
fn carried_counter_and_unwalked_values_survive_switch() {
    let records = empty();
    let mut world = records.committed(SEED, patches::BASE);
    assert_eq!(world.open_switched(), Some(1));
    world.mark_if_needed();
    let carried = world.secret(Secrets::Carry {
        kind: jig_protocol_store::CarriedKind::Value,
        value: Secret::new(Box::from(b"carried".as_slice())),
    });
    assert!(matches!(carried.as_slice(), [Event::Carried { number, .. }] if *number == CARRIED_FIRST));
    world.crash(SEED);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("job upgrade with carried row");
    world.mark_if_needed();
    let found = world.secret(Secrets::Get { owner: SecretOwner::Application, number: CARRIED_FIRST });
    assert!(
        matches!(found.as_slice(), [Event::Secret { value: Some(jig_protocol_store::SecretValue::Carried { kind: jig_protocol_store::CarriedKind::Value, value }), .. }] if value.expose() == b"carried")
    );
    world.mark_if_needed();
    let carried = world.secret(Secrets::Carry {
        kind: jig_protocol_store::CarriedKind::Value,
        value: Secret::new(Box::from(b"next".as_slice())),
    });
    assert!(matches!(carried.as_slice(), [Event::Carried { number, .. }] if *number == CARRIED_FIRST + 1));
    assert_eq!(patches::counts(&mut world)[2], (b"fixture-token".to_vec(), 0, 0));
}

#[test]
fn erased_carried_number_is_never_drawn_after_switch() {
    let mut world = empty().committed(SEED, patches::BASE);
    assert_eq!(world.open_switched(), Some(1));
    world.mark_if_needed();
    let carried = world.secret(Secrets::Carry {
        kind: jig_protocol_store::CarriedKind::Value,
        value: Secret::new(Box::from(b"erased".as_slice())),
    });
    assert!(matches!(carried.as_slice(), [Event::Carried { number, .. }] if *number == CARRIED_FIRST));
    let erased = world.secret(Secrets::Erase { owner: SecretOwner::Application, number: CARRIED_FIRST });
    assert!(matches!(erased.as_slice(), [Event::Erased { .. }]));
    world.crash(SEED);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("job upgrade with an erased carried value");
    world.mark_if_needed();
    let carried = world.secret(Secrets::Carry {
        kind: jig_protocol_store::CarriedKind::Value,
        value: Secret::new(Box::from(b"next".as_slice())),
    });
    assert!(matches!(carried.as_slice(), [Event::Carried { number, .. }] if *number == CARRIED_FIRST + 1));
}

#[test]
fn stray_secrets_file_is_copied_before_privacy_check() {
    let records = patches::Records::story();
    let mut world = records.committed(SEED, patches::BASE);
    // The source file is public; the copy must make it private.
    let mut stray = Item::file(b"generations/1/secrets/stray", b"fixture stray");
    stray.mode = 0o644;
    world.lay(&[stray]);
    let before = patches::source(&mut world);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("copy makes even a stray regular secrets file private");
    patches::kept(&before, &patches::source(&mut world), &"secrets stray");
    assert_eq!(world.contents().get(b"generations/2/secrets/stray".as_slice()), Some(&Some(b"fixture stray".to_vec())));
    records.check(&mut world, &"stray beside secrets map");
}

fn task_with_words(number: u64, bytes: usize) -> core::Record {
    let mut row = root::live_task(number);
    let core::Record::Tasks(tasks::Stored::Live(task)) = &mut row else {
        panic!("task fixture");
    };
    task.spec.words = vec![b'w'; bytes].into_boxed_slice();
    row
}

#[test]
fn live_record_exactly_at_decode_bound_switches() {
    let bound = skein_lib::length::usize_of(root::TASKS_LIMITS.spec_bytes);
    let row = task_with_words(root::LIVE, bound);
    let mut world = empty().committed_with(SEED, patches::BASE, patches::limits(), std::slice::from_ref(&row));
    world.owner_sets(patches::SETS);
    world.open_switched().expect("codec limit is inclusive");
    assert_eq!(world.get(row.key()), row);
}

#[test]
fn ended_position_with_its_task_passes_join() {
    let core::Record::Tasks(tasks::Stored::Live(mut task)) = root::live_task(root::LIVE) else {
        panic!("task fixture");
    };
    task.ended_at = Some(skein_lib::Wall::EPOCH);
    let ended = core::Record::Tasks(tasks::Stored::Ended(task));
    let position =
        core::Record::People(jig_domain_people::Stored::EndedRead { task: root::LIVE, party: 1, through: 0 });
    let mut world = empty().committed_with(SEED, patches::BASE, patches::limits(), &[ended, position]);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("ended read position's task is held");
    assert_eq!(world.active(), b"2\n");
}

#[test]
fn live_sets_past_arena_limits_switch_before_restart() {
    let records = empty();
    // The check decodes each row, but does not enforce the child's live-set bound (protocol/store/README.md 11).
    let rows = [root::live_task(root::LIVE), root::live_task(root::LIVE + 1), root::live_task(root::LIVE + 2)];
    let mut world = records.committed_with(SEED, patches::BASE, patches::limits(), &rows);
    let before = patches::source(&mut world);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("live set's arena bound is outside the check");
    assert_eq!(world.active(), b"2\n");
    world.crash(SEED);
    let restored = world.reopen();
    for row in &rows {
        assert_eq!(restored.rows.get(&row.key()), Some(row));
    }
    patches::kept(&before, &patches::source(&mut world), &"set past child bound");
    world.crash(SEED);
    world.rewrite(b"active", b"1\n");
    world.owner_sets(patches::BASE);
    assert_eq!(world.open_switched(), Some(1));
    assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/2")));
}

#[test]
fn child_semantic_rule_is_outside_pre_switch_check() {
    let mut row = root::live_task(root::LIVE);
    let core::Record::Tasks(tasks::Stored::Live(task)) = &mut row else {
        panic!("task fixture");
    };
    task.number = 0;
    let mut world = empty().committed_with(SEED, patches::BASE, patches::limits(), &[row]);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("codec-valid child rule is not checked before switch");
    assert_eq!(world.active(), b"2\n");
}

fn bounded_sets(families: usize, name_bytes: usize) -> OwnerSets {
    let application = &patches::SETS.sets[0];
    let upstream = &patches::SETS.sets[1];
    let mut kept = application.families.to_vec();
    // One family is in the upstream set; each additional family has an empty key range.
    for index in kept.len()..families - upstream.families.len() {
        let name: &'static str = Box::leak(format!("f{index:0width$}", width = name_bytes - 1).into_boxed_str());
        let text: &'static str =
            Box::leak(format!("family {name} 1\nversioned record Empty {{ n: u64 }}\n").into_boxed_str());
        kept.push(KeptFamily {
            retired: &[],
            current: text,
            types: &[],
            frames: None,
            name,
            map: jig_protocol_store::KeptMap::Store,
            keys: &[],
        });
    }
    let kept = Box::leak(kept.into_boxed_slice());
    let sets = Box::leak(vec![OwnerSet { families: kept, ..*application }, *upstream].into_boxed_slice());
    OwnerSets { sets }
}

#[test]
fn manifest_bounds_refuse_sets_before_any_first_start_write() {
    for (families, name_bytes, owner) in
        [(FAMILY_BOUND + 1, NAME_BOUND, Owner::Jig), (FAMILY_BOUND, NAME_BOUND + 1, Owner::Application)]
    {
        let mut world = World::unopened_observed(SEED, patches::limits(), None);
        world.owner_sets(bounded_sets(families, name_bytes));
        let before = world.contents();
        world.trace_writes();
        assert_eq!(world.open_switched(), None);
        assert!(
            matches!(world.refusal(), Some(OpenRefusal::Set { owner: found }) if *found == owner),
            "{:?}",
            world.refusal()
        );
        assert_eq!(world.contents(), before);
        assert_eq!((world.changes()).len(), 0);
    }
}

#[test]
fn manifest_bounds_refuse_job_upgrade_without_removing_strays() {
    for (families, name_bytes, owner) in
        [(FAMILY_BOUND + 1, NAME_BOUND, Owner::Jig), (FAMILY_BOUND, NAME_BOUND + 1, Owner::Application)]
    {
        let mut world = empty().committed(SEED, patches::BASE);
        world.lay(&[Item::directory(b"generations/9"), Item::file(b"generations/9/stray", b"untouched")]);
        let before = world.contents();
        world.owner_sets(bounded_sets(families, name_bytes));
        assert_eq!(world.open_switched(), None);
        assert!(
            matches!(world.refusal(), Some(OpenRefusal::Set { owner: found }) if *found == owner),
            "{:?}",
            world.refusal()
        );
        patches::kept(&before, &world.contents(), &"invalid manifest bounds");
        assert_eq!(world.active(), b"1\n");
    }
}

pub(crate) fn seal_and_switch(complete: &World) -> (u32, u32, u32) {
    let cuts = patches::cuts(complete);
    // The drop/copy sweep owns the copy's mutations. Start at the seal's sync, then cut
    // every operation through the read-only check and the final switch.
    let seal = complete
        .written()
        .iter()
        .find_map(|(operation, raw)| {
            let puts = super::commits::commit_keys(raw)?;
            puts.iter()
                .any(|(_, value)| {
                    jig_protocol_store::codec::component::Format::decode(value)
                        .is_ok_and(|format| format.levels.application == Some(0) && format.levels.jig == Some(0))
                })
                .then_some(*operation)
        })
        .expect("seal writes the unchanged levels");
    let first = complete
        .changes()
        .iter()
        .find(|(operation, change)| *operation > seal && *change == jig_store_world::world::Change::Sync)
        .expect("seal sync precedes the check")
        .0;
    let after_switch = cuts.last().expect("switch has cut points") + 1;

    let rename = patches::last_change(complete, jig_store_world::world::Change::Rename);
    (first, after_switch, rename)
}

pub(crate) fn check_cuts(records: &patches::Records, cuts: Vec<u32>) -> Vec<bool> {
    let mut world = records.committed(SEED, patches::BASE);
    let before = patches::source(&mut world);
    world.owner_sets(patches::SETS);
    world.trace_writes();
    let records = records.clone();
    let sweep = crate::sweep::Sweep::start(&mut world, cuts, move |world, cut| {
        a_check_cut_recovers(world, &records, &before, cut)
    });
    world.open_switched().expect("uncut check and switch");
    sweep.finish(&mut world)
}

fn a_check_cut_recovers(
    world: &mut World,
    records: &patches::Records,
    before: &std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    cut: u32,
) -> bool {
    assert_eq!(world.operation_count(), cut, "check/switch cut reached");
    world.crash(SEED);
    // The build without the job opens either whole generation at equal levels.
    world.owner_sets(patches::BASE);
    let source_active = world.active() == b"1\n";
    if source_active {
        assert_eq!(world.open_switched(), Some(1), "cut {cut}");
        world.accept_recovery(1);
        records.check_public(world, &cut);
        assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/2")));
    } else {
        assert!(world.open_switched().is_some(), "cut {cut}: {:?}", world.refusal());
        assert!(!world.pending);
        records.check_public(world, &cut);
    }
    patches::kept(before, &patches::source(world), &cut);
    // Observe recovery before check commits the start's wires to serve secrets (protocol/store/records.md 3).
    if source_active {
        records.check(world, &cut);
    }
    world.crash(SEED);
    world.owner_sets(patches::SETS);
    world.open_switched().expect("retry after cut");
    patches::whole(world);
    records.check(world, &cut);
    source_active
}

#[test]
fn cuts_at_the_seal_the_check_and_the_switch_recover_whole_maps() {
    let records = patches::Records::story();
    let mut complete = records.committed(SEED, patches::BASE);
    complete.owner_sets(patches::SETS);
    complete.trace_writes();
    complete.open_switched().expect("complete job upgrade to enumerate its cuts");
    let (seal_sync, after_switch, rename) = seal_and_switch(&complete);
    let check_read = seal_sync + 1;
    assert!(
        !complete.changes().iter().any(|(operation, _)| *operation == check_read),
        "the check's first read changes nothing"
    );
    let mut source_opened = false;
    let mut switched = false;
    let cuts = [
        ("seal sync", seal_sync),
        ("check read", check_read),
        ("switch rename", rename),
        ("past switch sync", after_switch),
    ]
    .into_iter()
    .map(|(why, cut)| {
        eprintln!("{why}: {cut}");
        cut
    })
    .collect();
    for source_active in check_cuts(&records, cuts) {
        source_opened |= source_active;
        switched |= !source_active;
    }
    assert!(source_opened && switched, "cuts reached each whole generation");
}

#[test]
fn admitted_manifest_bounds_hold_at_first_start() {
    const SHORT_NAME: usize = 8;
    const SPARSE_FAMILIES: usize = 5;
    const LARGE_CHUNK: u32 = 8192;
    for (families, name_bytes, chunk) in [
        (FAMILY_BOUND, NAME_BOUND, patches::limits().store.chunk),
        (FAMILY_BOUND, SHORT_NAME, patches::limits().store.chunk),
        (SPARSE_FAMILIES, NAME_BOUND, patches::limits().store.chunk),
        (FAMILY_BOUND, NAME_BOUND, LARGE_CHUNK),
    ] {
        let mut limits = patches::limits();
        limits.store.chunk = chunk;
        let mut world = World::unopened_observed(SEED, limits, None);
        world.owner_sets(bounded_sets(families, name_bytes));
        assert_eq!(
            world.open_switched(),
            Some(0),
            "families={families}, name={name_bytes}, chunk={chunk}: {:?}",
            world.refusal()
        );
        let first = manifest(&mut world, 1);
        assert_eq!(skein_lib::length::usize_of(first.counts.len()), families);
        assert!(first.counts.iter().all(|row| row.before == 0 && row.after == 0));
    }
}

#[test]
fn admitted_manifest_bounds_hold_after_job() {
    const LARGE_CHUNK: u32 = 8192;
    for chunk in [patches::limits().store.chunk, LARGE_CHUNK] {
        let mut limits = patches::limits();
        limits.store.chunk = chunk;
        let mut world = empty().committed(SEED, patches::BASE);
        world.restart_with_limits(SEED, limits);
        world.owner_sets(bounded_sets(FAMILY_BOUND, NAME_BOUND));
        assert!(world.open_switched().is_some(), "chunk={chunk}: {:?}", world.refusal());
        let counts = patches::counts(&mut world);
        assert_eq!(counts.len(), FAMILY_BOUND);
        assert!(counts.iter().all(|(_, before, after)| *before == 0 && *after == 0));
    }
}

#[test]
fn invalid_live_rows_refuse_load_before_joins_in_key_order() {
    let ordinary = patches::limits();
    let mut wider = ordinary;
    wider.core.tasks.spec_bytes += 1;
    let past_bound = task_with_words(root::LIVE, skein_lib::length::usize_of(wider.core.tasks.spec_bytes));
    let row = root::live_task(root::LIVE);
    let position =
        core::Record::People(jig_domain_people::Stored::EndedRead { task: root::LIVE + 1, party: 1, through: 0 });
    for (label, initial, bound, rows, expected) in [
        ("smaller configured bound", ordinary, 1, vec![row.clone()], load_key(&row)),
        ("one byte past", wider, ordinary.core.tasks.spec_bytes, vec![past_bound.clone()], load_key(&past_bound)),
        ("key order", ordinary, 1, vec![root::live_task(root::LIVE + 1), row.clone()], load_key(&row)),
        ("load before join", ordinary, 1, vec![position, row.clone()], load_key(&row)),
    ] {
        let mut world = empty().committed_with(SEED, patches::BASE, initial, &rows);
        let mut limits = ordinary;
        limits.core.tasks.spec_bytes = bound;
        world.restart_with_limits(SEED, limits);
        refuses_check(&mut world, &Check::Load { key: expected });
        assert_eq!(world.active(), b"1\n", "{label}");
    }
}
