//! skein-kv's on-disk format's golden bytes (data/kv.md, 9): a snapshot and
//! the segments from its start. The tests compare bytes on the simulated
//! disk and report differences without writing any host files.

use std::collections::BTreeMap;

use crate::world::World;
use skein_fake_machine::Item;
use skein_kv::{Event, Limits, Op, Page, Request};
use skein_lib::{Duration, Token};

/// The limits the kept files were made under: small, so that the
/// sequence takes a snapshot of several chunks and then rolls a segment.
/// A change to one changes the kept files.
pub const LIMITS: Limits = Limits {
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
    record: 1,
    log: 1 << 40,
    logs: 16,
    deadline: Duration::from_secs(1),
};

/// The kept files, by name, as the store's root holds them.
pub const KEPT: [(&[u8], &[u8]); 3] = [
    (b"log-00000000000000000002", include_bytes!("../golden/log-00000000000000000002")),
    (b"log-00000000000000000005", include_bytes!("../golden/log-00000000000000000005")),
    (b"snapshot", include_bytes!("../golden/snapshot")),
];

/// The commit number of the sequence's last commit.
pub const LAST: u64 = 5;

/// The fixed sequence of commits, each a named step.
#[must_use]
pub fn commits() -> Vec<Box<[Op]>> {
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
#[must_use]
pub fn model() -> BTreeMap<Box<[u8]>, Box<[u8]>> {
    let mut map = BTreeMap::new();
    for ops in commits() {
        for op in ops {
            match op {
                Op::Append { .. } | Op::Drop { .. } => unreachable!("map-only fixture"),
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

pub fn expect_event(world: &mut World, expected: &Event) {
    match world.pop() {
        event if event == *expected => {}
        event => panic!("setup expected {expected:?}, got {event:?}"),
    }
}

#[must_use]
pub fn open(items: &[Item], last: u64) -> World {
    let mut world = World::with_items(LIMITS, items);
    world.request(Request::Open { owner: Token::new(1), root: world.root });
    expect_event(&mut world, &Event::Opened { owner: Token::new(1), last });
    world
}
