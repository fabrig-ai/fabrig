//! skein-kv's on-disk format's golden bytes (kv.md, 9): a snapshot and
//! the segments from its start. This test never writes the kept files;
//! a deliberate change copies the files its failure names over `golden/`.

use std::collections::BTreeMap;
use std::path::Path;

use skein_fake_machine::Item;
use skein_kv::{Event, Limits, Op, Page, Range, Request};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Token};

/// The limits the kept files were made under: small, so that the
/// sequence takes a snapshot of several chunks and then rolls a segment.
/// A change to one changes the kept files.
const LIMITS: Limits = Limits {
    key: 8,
    value: 16,
    ops: 4,                            // The first commit's four puts.
    commit: 128,                       // The first frame is 120 bytes.
    queued: 1,                         // Each commit is answered before the next starts.
    queued_bytes: 128,                 // Room for one frame at the commit limit.
    budget: 1024,                      // Four map entries, including their 128-byte charges.
    segment: 80,                       // Commits 2..=4 exceed this; commit 5 rolls.
    snapshot_after: 100,               // Commit 1's 120 bytes start the snapshot at 2.
    chunk: 52,                         // Empty + a fit; b and the maximum row each need a chunk.
    page: Page { rows: 1, bytes: 24 }, // One maximum key/value pair per page.
    deadline: Duration::from_secs(1),
};

/// The kept files, by name, as the store's root holds them.
const KEPT: [(&[u8], &[u8]); 3] = [
    (b"log-00000000000000000002", include_bytes!("../../golden/log-00000000000000000002")),
    (b"log-00000000000000000005", include_bytes!("../../golden/log-00000000000000000005")),
    (b"snapshot", include_bytes!("../../golden/snapshot")),
];

/// The commit number of the sequence's last commit.
const LAST: u64 = 5;

/// The fixed sequence of commits, each a named step.
fn commits() -> Vec<Box<[Op]>> {
    let key = vec![b'z'; usize::try_from(LIMITS.key).expect("key bound fits usize")];
    let value = vec![b'x'; usize::try_from(LIMITS.value).expect("value bound fits usize")];
    vec![
        // 1: Both bounds and two middle rows make three snapshot chunks;
        // its frame passes snapshot_after, so the snapshot starts at 2.
        Box::new([put(b"", b""), put(b"a", &value), put(b"b", &value), put(&key, &value)]),
        // 2: The kept log puts a changed value and erases a key the map holds.
        Box::new([put(b"a", b"new"), Op::Erase { key: Box::from(&b"b"[..]) }]),
        // 3: The kept log represents an empty commit with no operations.
        Box::new([]),
        // 4: Both bounds are puts in the kept log; its offset passes segment.
        Box::new([put(&key, &value), put(b"", b"")]),
        // 5: This put rolls to a second kept segment, starting at commit 5.
        Box::new([put(b"c", &value)]),
    ]
}

fn put(key: &[u8], value: &[u8]) -> Op {
    Op::Put { key: Box::from(key), value: Box::from(value) }
}

/// The map the sequence leaves: its ops applied in order to a `BTreeMap`.
fn model() -> BTreeMap<Box<[u8]>, Box<[u8]>> {
    let mut map = BTreeMap::new();
    for ops in commits() {
        for op in ops {
            match op {
                Op::Put { key, value } => {
                    map.insert(key, value);
                }
                Op::Erase { key } => {
                    map.remove(&key);
                }
            }
        }
    }
    map
}

fn format_name(name: &[u8]) -> &'static str {
    if name == b"snapshot" {
        "skein-kv's snapshot format (kv.md 3 and 5)"
    } else {
        "skein-kv's log frame format (kv.md 3)"
    }
}

/// Compares the files made with KEPT; on a mismatch writes the files made
/// under `CARGO_TARGET_TMPDIR` and fails naming the format.
fn assert_kept(made: &BTreeMap<Box<[u8]>, Box<[u8]>>) {
    let kept: BTreeMap<&[u8], &[u8]> = KEPT.into_iter().collect();
    let mut differences = Vec::new();
    for (&name, &bytes) in &kept {
        let format = format_name(name);
        let label = String::from_utf8_lossy(name);
        match made.get(name) {
            None => differences.push(format!("missing file {label}: {format}")),
            Some(written) if written.as_ref() != bytes => {
                let offset = bytes
                    .iter()
                    .zip(written.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or(bytes.len().min(written.len()));
                differences.push(format!(
                    "{label}: {format}: first differing offset {offset}, kept length {}, made length {}",
                    bytes.len(),
                    written.len()
                ));
            }
            Some(_) => {}
        }
    }
    for name in made.keys() {
        if !kept.contains_key(name.as_ref()) {
            differences.push(format!("extra file {}: {}", String::from_utf8_lossy(name), format_name(name)));
        }
    }
    if differences.is_empty() {
        return;
    }
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("kv-golden-made");
    if path.exists() {
        std::fs::remove_dir_all(&path).expect("remove the previous failure's output");
    }
    std::fs::create_dir_all(&path).expect("create the failure's output directory");
    for (name, bytes) in made {
        let name = std::str::from_utf8(name).expect("store filenames are ASCII");
        std::fs::write(path.join(name), bytes).expect("write the files made for inspection");
    }
    panic!(
        "{}\nfrom the first release a deployment's store holds these bytes: changing them needs an upgrade of the store's own, run before any records are patched (kv.md 9), and moves the format's version\nfiles made written to {}",
        differences.join("\n"),
        path.display()
    );
}

fn expect_event(world: &mut World, expected: &Event) {
    match world.pop() {
        event if event == *expected => {}
        event => panic!("setup expected {expected:?}, got {event:?}"),
    }
}

fn open(items: &[Item], last: u64) -> World {
    let mut world = World::with_items(LIMITS, items);
    world.request(Request::Open { owner: Token::new(1), root: world.root });
    expect_event(&mut world, &Event::Opened { owner: Token::new(1), last });
    world
}

#[test]
fn the_fixed_commits_write_the_kept_files_byte_for_byte() {
    let mut world = open(&[], 0);
    for (index, ops) in commits().into_iter().enumerate() {
        let number = u64::try_from(index).expect("commit index fits u64") + 1;
        world.request_no_fire(Request::Commit { owner: Token::new(2), ops });
        expect_event(&mut world, &Event::Committed { owner: Token::new(2), number });
        if number == 1 {
            // Finish the snapshot before later commits. They retain its start
            // and roll the log without starting another snapshot before close.
            world.drive(true);
        }
    }
    world.request(Request::Close { owner: Token::new(3) });
    expect_event(&mut world, &Event::Closed { owner: Token::new(3) });
    assert_kept(&world.files());
}

#[test]
fn the_kept_files_recover_to_the_models_map() {
    let items: Vec<Item> = KEPT.iter().map(|(name, bytes)| Item::file(name, bytes)).collect();
    let mut world = open(&items, LAST);
    let mut map = BTreeMap::new();
    let mut range = Range::prefix(b"");
    loop {
        world.request(Request::Load { owner: Token::new(4), range: range.clone(), max: LIMITS.page });
        let Event::Loaded { rows, next, .. } = world.pop() else {
            panic!("setup expected a loaded page");
        };
        for row in rows {
            map.insert(row.key, row.value);
        }
        match next {
            Some(start) => range.start = start,
            None => break,
        }
    }
    assert_eq!(map, model());
}
