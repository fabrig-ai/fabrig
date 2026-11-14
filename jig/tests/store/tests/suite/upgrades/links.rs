//! Shared generation files, never shared writes (protocol/store/opening.md, 2, 4 and 5).
use jig_protocol_store::{Event, PatchSet, PatchSets, Write};
use jig_store_world::{
    World,
    patches::{self, BASE, Records, SETS},
    root,
    world::Change,
};
use skein_fake_machine::Item;
use skein_io::kernel::Error;
use std::collections::BTreeMap;

const SEED: u64 = 0x4c49_4e4b;
const MID: PatchSets = PatchSets {
    sets: &[
        PatchSet {
            families: &[SETS.sets[0].families[0], BASE.sets[0].families[1], BASE.sets[0].families[2]],
            patches: &[jig_test_patches::patches::PATCHES[0]],
            ..SETS.sets[0]
        },
        BASE.sets[1],
    ],
};
type Contents = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

/// A snapshot, then commits on two later opens: both sealed and newest segments stand.
fn previous() -> World {
    let mut initial = patches::limits();
    initial.store.snapshot_after = 1;
    let mut world = Records::story().committed_with(SEED, BASE, 1, initial, &[]);
    world.restart_with_limits(SEED, patches::limits());
    for last in 1..=2 {
        assert_eq!(world.open_switched(), Some(last));
        assert!(world.wires(&[]));
        assert_eq!(world.commit(Box::new([Write::Save(root::header(last + 1)), Write::Wires])), Some(last + 1));
        world.crash(SEED);
    }
    assert!(world.contents().contains_key(b"generations/1/store/snapshot".as_slice()));
    world
}

fn files(world: &mut World, generation: u64) -> Contents {
    let prefix = format!("generations/{generation}/");
    world.contents().into_iter().filter(|(path, _)| path.starts_with(prefix.as_bytes())).collect()
}

fn link_counts(world: &mut World, contents: &Contents) -> BTreeMap<Vec<u8>, u32> {
    contents.iter().filter(|(_, bytes)| bytes.is_some()).map(|(path, _)| (path.clone(), world.links(path))).collect()
}

#[test]
fn pending_copy_links_sealed_store_files_and_copies_the_rest_privately() {
    let mut world = previous();
    for name in ["snapshot.tmp", "log-1"] {
        world.lay(&[Item::file(format!("generations/1/store/{name}").as_bytes(), name.as_bytes()).mode(0o600)]);
    }
    world.patch_sets(SETS);
    world.trace_writes();
    world.open().expect("pending patched copy");
    assert!(world.pending);
    assert_eq!(world.active(), b"1\n");
    let source = files(&mut world, 1);
    let target = files(&mut world, 2);
    let segments: Vec<_> = source
        .keys()
        .filter(|path| {
            path.strip_prefix(b"generations/1/store/log-".as_slice())
                .is_some_and(|digits| digits.len() == 20 && digits.iter().all(u8::is_ascii_digit))
        })
        .collect();
    assert!(segments.len() >= 3, "several opens leave sealed segments beside the newest");
    let newest = segments.last().expect("segments");
    let mut linked = 0;
    for (path, bytes) in source.iter().filter(|(_, bytes)| bytes.is_some()) {
        let suffix = path.strip_prefix(b"generations/1/").expect("source prefix");
        if !(suffix.starts_with(b"store/") || suffix.starts_with(b"secrets/")) {
            continue;
        }
        let sealed = segments.contains(&path) && path != *newest;
        let links = if suffix == b"store/snapshot" || sealed {
            linked += 1;
            2
        } else {
            1
        };
        let to = [b"generations/2/", suffix].concat();
        assert_eq!(world.links(path), links, "source {}", String::from_utf8_lossy(path));
        assert_eq!(world.links(&to), links, "copy {}", String::from_utf8_lossy(&to));
        // Recovery may reuse the copied newest empty segment for patch commits.
        if links == 2 || suffix == b"store/snapshot.tmp" || suffix == b"store/log-1" {
            assert_eq!(target.get(&to), Some(bytes), "same bytes: {}", String::from_utf8_lossy(path));
        }
    }
    assert_eq!(world.changes().iter().filter(|(_, change)| *change == Change::Link).count(), linked);
    for (path, directory, mode) in
        world.layout_entries().into_iter().filter(|(path, _, _)| path.starts_with(b"generations/2/secrets"))
    {
        assert_eq!(mode, if directory { 0o700 } else { 0o600 }, "{}", String::from_utf8_lossy(&path));
    }
    Records::story().check_public(&mut world, 2, &"pending linked copy");
}

#[test]
fn cross_device_links_fall_back_to_single_link_files_and_all_records() {
    let mut world = previous();
    let before = files(&mut world, 1);
    world.refuse_links(Some(Error::Other(18))); // EXDEV
    world.trace_writes();
    world.patch_sets(SETS);
    world.open_switched().expect("copy fallback upgrades");
    assert!(world.changes().iter().any(|(_, change)| *change == Change::Link), "refusal seam exercised");
    patches::kept(&before, &files(&mut world, 1), &"refused links");
    let contents = world.contents();
    for (path, _) in contents.iter().filter(|(_, bytes)| bytes.is_some()) {
        assert_eq!(world.links(path), 1, "{}", String::from_utf8_lossy(path));
    }
    patches::whole(&mut world);
    Records::story().check(&mut world, 2, &"EXDEV upgrade");
}

#[test]
fn back_after_patch_snapshots_keeps_the_shared_previous_files_whole() {
    let mut world = previous();
    let before = files(&mut world, 1);
    let replay: u64 = before
        .iter()
        .filter(|(path, _)| path.starts_with(b"generations/1/store/log-"))
        .map(|(_, bytes)| u64::try_from(bytes.as_ref().expect("segment bytes").len()).expect("small fixture"))
        .sum();
    // The source replay stays below the ceiling; the first patch commit crosses it.
    let mut limits = patches::limits();
    limits.store.snapshot_after = replay + 1;
    world.restart_with_limits(SEED, limits);
    world.patch_sets(SETS);
    world.trace_writes();
    world.open_switched().expect("patch snapshots and switch");
    assert!(
        world.changes().iter().filter(|(_, change)| *change == Change::Link).count() >= 3,
        "snapshot and sealed segments were linked before patch snapshots replaced them"
    );
    let after = files(&mut world, 2);
    let old = before.get(b"generations/1/store/snapshot".as_slice()).expect("previous snapshot");
    assert_ne!(after.get(b"generations/2/store/snapshot".as_slice()), Some(old), "copy replaced its snapshot");
    assert_eq!(world.links(b"generations/1/store/snapshot"), 1, "replacement dropped only the copy's name");
    for (path, _) in
        before.iter().filter(|(path, bytes)| path.starts_with(b"generations/1/store/log-") && bytes.is_some())
    {
        let to = [b"generations/2/", path.strip_prefix(b"generations/1/").expect("source prefix")].concat();
        assert!(!after.contains_key(&to), "snapshot dropped the copy's sealed segment");
        assert_eq!(world.links(path), 1);
    }
    patches::kept(&before, &files(&mut world, 1), &"patch snapshots");
    world.crash(SEED);
    let events = world.back();
    assert!(matches!(events.as_slice(), [Event::Back { to: 1, .. }]), "{events:?}");
    world.restart_with_limits(SEED, patches::limits());
    world.patch_sets(BASE);
    assert_eq!(world.open_switched(), Some(3));
    patches::kept(&before, &files(&mut world, 1), &"back after snapshots");
    assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/2")));
    world.accept_recovery(3);
    Records::story().check(&mut world, 1, &"previous records");
}

#[test]
fn second_upgrade_drops_only_shared_names_then_links_the_survivors() {
    let mut trace = previous();
    trace.patch_sets(MID);
    trace.open_switched().expect("first upgrade");
    trace.crash(SEED);
    trace.patch_sets(SETS);
    trace.trace_writes();
    trace.open_switched().expect("second upgrade");
    let drop = patches::first_change(&trace, Change::Rename);
    let copy = trace
        .changes()
        .iter()
        .find(|(operation, change)| *operation > drop && *change == Change::Make)
        .expect("copy mkdir follows drop")
        .0;

    let mut world = previous();
    world.patch_sets(MID);
    world.open_switched().expect("first upgrade");
    let before = files(&mut world, 2);
    let counts = link_counts(&mut world, &before);
    let shared: Vec<_> = counts.iter().filter(|(_, links)| **links == 2).map(|(path, _)| path.clone()).collect();
    assert!(!shared.is_empty(), "first upgrade shares sealed files");
    world.crash(SEED);
    world.patch_sets(SETS);
    world.trace_writes();
    world.cut_at(copy);
    assert_eq!(world.open(), None);
    assert!(world.reached_cut());
    assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/1")));
    for path in &shared {
        assert_eq!(world.links(path), counts[path] - 1, "drop removed one name");
        assert_eq!(world.contents().get(path), before.get(path), "surviving bytes");
    }
    world.crash(SEED);
    world.open_switched().expect("second upgrade resumes");
    assert_eq!(world.active(), b"3\n");
    Records::story().check(&mut world, 2, &"two upgrades");
}

#[test]
fn cuts_at_each_link_remove_partial_names_and_retry_from_whole_source() {
    let mut trace = previous();
    trace.patch_sets(SETS);
    trace.trace_writes();
    trace.open_switched().expect("discover link cuts");
    let links: Vec<_> =
        trace.changes().iter().filter(|(_, change)| *change == Change::Link).map(|(operation, _)| *operation).collect();
    assert!(links.len() >= 2, "a cut between links leaves an unsynced link");
    let cuts = patches::cuts(&trace);
    assert!(links.iter().all(|link| cuts.contains(link)), "mutation sweeps include links");
    for cut in links {
        let mut world = previous();
        let before = files(&mut world, 1);
        let counts = link_counts(&mut world, &before);
        world.patch_sets(SETS);
        world.trace_writes();
        world.cut_at(cut);
        assert_eq!(world.open(), None);
        assert!(world.reached_cut());
        let made =
            world.changes().iter().filter(|(operation, change)| *operation < cut && *change == Change::Link).count();
        if made > 0 {
            assert!(link_counts(&mut world, &before).values().any(|links| *links == 2), "a link made before the cut");
        }
        world.crash(SEED);
        patches::kept(&before, &files(&mut world, 1), &cut);
        assert_eq!(link_counts(&mut world, &before), counts, "unsynced links lost at cut {cut}");
        world.open_switched().expect("partial copy removed and patched afresh");
        patches::kept(&before, &files(&mut world, 1), &cut);
        patches::whole(&mut world);
        // No partial generation leaks another name beside the successful copy.
        let source = files(&mut world, 1);
        for links in link_counts(&mut world, &source).values() {
            assert!(*links <= 2);
        }
        Records::story().check(&mut world, 2, &cut);
    }
}

#[test]
fn rehearsal_copies_even_a_sources_shared_files_without_changing_its_links() {
    let mut world = previous();
    world.patch_sets(SETS);
    world.open_switched().expect("source contains linked generations");
    world.crash(SEED);
    let before = world.contents();
    let counts = link_counts(&mut world, &before);
    assert!(counts.values().any(|links| *links == 2));
    let events = world.rehearse();
    assert!(matches!(events.last(), Some(Event::Opened { pending: false, .. })), "{events:?}");
    let scratch = world.scratch_contents();
    for (path, _) in scratch.iter().filter(|(_, bytes)| bytes.is_some()) {
        assert_eq!(world.scratch_links(path), 1, "{}", String::from_utf8_lossy(path));
    }
    assert_eq!(world.contents(), before);
    assert_eq!(link_counts(&mut world, &before), counts);
    Records::story().check_public(&mut world, 2, &"rehearsal copy");
}
