//! Generation copy, cleanup and power loss at equal levels (protocol/store/opening.md, 2 and 4).
use jig_store_world::{
    World,
    patches::{self, BASE, Records, SETS},
};
use skein_fake_machine::Item;
const SEED: u64 = 0x5141;

fn pending(records: &Records) -> World {
    let mut world = records.committed(SEED, BASE);
    world.owner_sets(SETS);
    world
}

pub(crate) fn second_generation() -> World {
    let mut world = super::logs::source(SEED);
    world.owner_sets(SETS);
    world.open_switched().expect("intermediate upgrade opens");
    world.crash(SEED);
    world.owner_sets(patches::NEXT);
    world
}

#[test]
fn manifest_is_opaque_and_unchanged_at_open() {
    // protocol/store/README.md 2: nothing at open reads the manifest; it is the operator's record.
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
    // protocol/store/opening.md 1 step 5: directories are listed again from their first page until empty; active and previous stay.
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
    assert_eq!(world.ran(), []);
}

#[test]
fn first_start_recovers_after_every_disk_mutation() {
    // protocol/store/opening.md 4: after each first-start write, sync and rename the next start opens one whole generation.
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

pub(crate) fn a_drop_copy_or_switch_cut_recovers(
    operation: u32,
    change: jig_store_world::world::Change,
    expected_last: u64,
    link_error: Option<skein_io::kernel::Error>,
) {
    let mut world = second_generation();
    world.refuse_links(link_error);
    let source: std::collections::BTreeMap<_, _> =
        world.contents().into_iter().filter(|(path, _)| path.starts_with(b"generations/2/")).collect();
    world.trace_writes();
    world.cut_at(operation);
    let _ = world.open_switched();
    assert_eq!(world.operation_count(), operation, "drop/copy/switch cut reached");
    // Before the switch's rename is even attempted, only the drop's rename was seen.
    if world.changes().iter().filter(|(_, change)| *change == jig_store_world::world::Change::Rename).count() < 2 {
        assert_eq!(world.active(), b"2\n", "copy cuts keep A active");
    }
    world.crash(SEED);
    assert_eq!(world.open_switched(), Some(expected_last), "cut {operation} {change:?}: {:?}", world.refusal());
    assert_eq!(world.active(), b"3\n", "cut {operation} {change:?}");
    Records::story().check(&mut world, &(operation, change));
    let after = world.contents();
    for (path, bytes) in &source {
        assert_eq!(after.get(path), Some(bytes), "source changed at {operation}: {path:?}");
        if let Some(suffix) = path.strip_prefix(b"generations/2/store/logs/")
            && bytes.is_some()
        {
            let to = [b"generations/3/store/logs/", suffix].concat();
            assert_eq!(after.get(&to), Some(bytes), "retried copy's log bytes");
        }
    }
    assert!(!after.keys().any(|path| path.starts_with(b"generations/1/") || path.starts_with(b"generations/dropped")));
}

#[test]
fn drop_copy_and_switch_recover_at_each_step_s_first_mutation() {
    use jig_store_world::world::Change;
    let mut trace = second_generation();
    trace.trace_writes();
    let expected_last = trace.open_switched().expect("traced second upgrade");
    let drop = patches::first_change(&trace, Change::Rename);
    let removal = trace
        .changes()
        .iter()
        .find(|(operation, change)| *operation > drop && *change == Change::Remove)
        .expect("drop removal after its rename")
        .0;
    let copy = trace
        .changes()
        .iter()
        .find(|(operation, change)| *operation > drop && *change == Change::Make)
        .expect("copy directory after the drop")
        .0;
    let rename = patches::last_change(&trace, Change::Rename);
    let sync = trace
        .changes()
        .iter()
        .find(|(operation, change)| *operation > rename && *change == Change::Sync)
        .expect("switch sync")
        .0;
    for (why, operation, change) in [
        ("drop rename", drop, Change::Rename),
        ("drop removal", removal, Change::Remove),
        ("copy directory", copy, Change::Make),
        ("switch rename", rename, Change::Rename),
        ("switch sync", sync, Change::Sync),
    ] {
        eprintln!("{why}: {operation}");
        a_drop_copy_or_switch_cut_recovers(operation, change, expected_last, None);
    }
}
