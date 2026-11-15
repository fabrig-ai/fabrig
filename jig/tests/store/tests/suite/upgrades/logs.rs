//! Task logs travel with their generation (protocol/store/logs.md, 7).
use jig_protocol_store::{Event, Request, codec, keys};
use jig_store_world::{
    World,
    patches::{self, BASE, Records, SETS},
    root,
};
use skein_fake_machine::Item;
use skein_io::kernel::Error;
use skein_lib::Token;
use std::collections::BTreeMap;

const SEED: u64 = 0x4c4f_4753;
const LOGS: &[u64] = &[6, 7, 0x1008];
type Contents = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

fn path(generation: u64, log: u64) -> Vec<u8> {
    format!("generations/{generation}/store/logs/{:016x}/{log:016x}", log >> 12).into_bytes()
}
fn bytes(log: u64) -> Vec<u8> {
    format!("task log {log}\n").into_bytes()
}

/// Raw files and their raw extent rows: opening a log-aware kv must not orphan them.
/// The component's log writers and readers are not part of this fixture.
pub(crate) fn source(seed: u64) -> World {
    source_with(seed, LOGS)
}
fn source_with(seed: u64, logs: &[u64]) -> World {
    let mut world = Records::story().committed(seed, BASE);
    let mut rows = Vec::new();
    world.lay(&[Item::directory(b"generations/1/store/logs").mode(0o700)]);
    let mut shards = std::collections::BTreeSet::new();
    for &log in logs {
        if shards.insert(log >> 12) {
            world
                .lay(&[Item::directory(format!("generations/1/store/logs/{:016x}", log >> 12).as_bytes()).mode(0o700)]);
        }
        let payload = bytes(log);
        world.lay(&[Item::file(&path(1, log), &payload).mode(0o600)]);
        rows.push((
            [b"\x00\x09".as_slice(), &log.to_be_bytes()].concat().into_boxed_slice(),
            u64::try_from(payload.len()).expect("fixture length").to_be_bytes().to_vec().into_boxed_slice(),
        ));
    }
    let contents = world.contents();
    let newest = contents
        .keys()
        .filter_map(|path| {
            let digits = path.strip_prefix(b"generations/1/store/log-")?;
            std::str::from_utf8(digits).ok()?.parse::<u64>().ok()
        })
        .max()
        .expect("source segment");
    let mut frames = Vec::new();
    let limits = patches::limits();
    for (offset, rows) in rows.chunks(31).enumerate() {
        let position = 2 + u64::try_from(offset).expect("fixture commits");
        let header = root::header(position);
        let mut rows = rows.to_vec();
        rows.push((
            keys::encode(&header.key(), limits.store.key).expect("header key"),
            codec::encode(&header, &codec::limits(&limits.core).expect("fixture codec"), &[]).expect("header"),
        ));
        frames.extend(super::baseline::source_commit(&rows, position));
    }
    world.lay(&[Item::file(format!("generations/1/store/log-{:020}", newest + 1).as_bytes(), &frames).mode(0o600)]);
    world
}

fn logs_at(world: &mut World, generation: u64) -> Contents {
    let prefix = format!("generations/{generation}/store/logs");
    world.contents().into_iter().filter(|(path, _)| path.starts_with(prefix.as_bytes())).collect()
}
fn check_logs(world: &mut World, generation: u64, logs: &[u64], links: u32) {
    let contents = world.contents();
    for &log in logs {
        let path = path(generation, log);
        assert_eq!(contents.get(&path), Some(&Some(bytes(log))), "{}", String::from_utf8_lossy(&path));
        assert_eq!(world.links(&path), links, "{}", String::from_utf8_lossy(&path));
    }
}

#[test]
fn task_logs_link_across_shard_and_log_pages_and_survive_a_cut_after_copy() {
    // More than one page at both levels; no list of the whole tree is needed.
    let logs: Vec<_> = (0..33).chain((1..33).map(|shard| (shard << 12) + 6)).collect();
    let mut world = source_with(SEED, &logs);
    world.patch_sets(SETS);
    assert!(world.open().is_some(), "logs copied before pending switch: {:?}", world.refusal());
    assert_eq!(world.active(), b"1\n");
    check_logs(&mut world, 1, &logs, 2);
    check_logs(&mut world, 2, &logs, 2);
    for (path, directory, mode) in
        world.layout_entries().into_iter().filter(|(path, _, _)| path.starts_with(b"generations/2/store/logs"))
    {
        assert_eq!(mode, if directory { 0o700 } else { 0o600 }, "{}", String::from_utf8_lossy(&path));
    }
    let copied = logs_at(&mut world, 2);
    world.crash(SEED);
    assert_eq!(logs_at(&mut world, 2), copied, "every shard and logs synced after their links");
    check_logs(&mut world, 1, &logs, 2);
    world.patch_sets(BASE);
    world.open_switched().expect("old binary removes partial copy");
    assert!(logs_at(&mut world, 2).is_empty());
    check_logs(&mut world, 1, &logs, 1);
}

#[test]
fn refused_task_log_links_copy_bytes_with_one_name() {
    for error in [Error::Permission, Error::TooManyLinks, Error::Other(18)] {
        let mut world = source(SEED);
        world.refuse_links(Some(error));
        world.patch_sets(SETS);
        world.open().expect("refused log links copied");
        check_logs(&mut world, 1, LOGS, 1);
        check_logs(&mut world, 2, LOGS, 1);
        let copied = logs_at(&mut world, 2);
        world.crash(SEED);
        assert_eq!(logs_at(&mut world, 2), copied, "fallback bytes and names durable: {error:?}");
    }
}

#[test]
fn copy_refuses_directories_outside_logs_and_below_shards() {
    for unexpected in ["generations/1/store/other", "generations/1/store/logs/0000000000000000/nested"] {
        let mut world = source(SEED);
        world.lay(&[Item::directory(unexpected.as_bytes()).mode(0o700)]);
        let before = logs_at(&mut world, 1);
        world.patch_sets(SETS);
        assert_eq!(world.open(), None, "{unexpected}");
        assert!(world.refusal().is_some(), "unexpected entry refused");
        assert_eq!(world.active(), b"1\n");
        assert_eq!(logs_at(&mut world, 1), before);
        world.crash(SEED);
        world.patch_sets(BASE);
        world.open_switched().expect("refused copy cleaned with every handle drained");
        assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/2")));
    }
}

#[test]
fn rehearsal_copies_task_logs_privately_and_empties_the_deep_scratch() {
    for (sets, pending) in [(BASE, false), (SETS, true)] {
        let mut world = source(SEED);
        world.patch_sets(sets);
        // Also exercise a patch inside the scratch on a file system without links.
        if pending {
            world.refuse_links(Some(Error::Other(18)));
        }
        let before = world.contents();
        world.trace_writes();
        let events = world.rehearse();
        assert!(matches!(events.last(), Some(Event::Opened { pending: found, .. }) if *found == pending), "{events:?}");
        if !pending {
            assert!(
                !world.changes().iter().any(|(_, change)| *change == jig_store_world::world::Change::Link),
                "rehearsal never attempts links into its scratch"
            );
        }
        let scratch = world.scratch_contents();
        for generation in 1..=if pending { 2 } else { 1 } {
            for &log in LOGS {
                let copied = path(generation, log);
                assert_eq!(scratch.get(&copied), Some(&Some(bytes(log))));
                assert_eq!(world.scratch_links(&copied), 1);
                assert_eq!(world.links(&path(1, log)), 1);
            }
        }
        let events = world.request(Request::Close { owner: Token::new(900) });
        assert!(matches!(events.as_slice(), [Event::Closed { removed: true, .. }]), "{events:?}");
        assert!(world.scratch_contents().is_empty());
        assert_eq!(world.contents(), before);
    }
}

#[test]
fn back_before_the_first_own_commit_reopens_the_previous_task_logs() {
    let mut world = source(SEED);
    let before = logs_at(&mut world, 1);
    world.patch_sets(SETS);
    world.open_switched().expect("upgraded with logs");
    check_logs(&mut world, 2, LOGS, 2);
    world.crash(SEED);
    let events = world.back();
    assert!(matches!(events.as_slice(), [Event::Back { to: 1, .. }]), "{events:?}");
    world.crash(SEED);
    world.patch_sets(BASE);
    assert_eq!(world.open_switched(), Some(2));
    assert_eq!(world.active(), b"1\n");
    assert_eq!(logs_at(&mut world, 1), before);
    check_logs(&mut world, 1, LOGS, 1);
    assert!(logs_at(&mut world, 2).is_empty());
}
