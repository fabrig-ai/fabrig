//! Recovery reads and heap follow the files' sizes (skein/docs/design/data/kv.md, 3 and 9).

use skein_heap::Span;
use skein_kv::{Event, Limits, Op, Page, Request};
use skein_kv_tests::world::World;
use skein_lib::{Duration, Token};

fn limits() -> Limits {
    Limits {
        key: 64,
        value: 64,
        ops: 8,
        commit: 1024,
        queued: 8,
        queued_bytes: 8192,
        budget: 8192,
        segment: 4096,
        snapshot_after: 1,
        chunk: 1_048_576,
        page: Page { rows: 16, bytes: 1024 },
        deadline: Duration::from_secs(1),
    }
}

#[test]
fn a_small_store_opens_with_reads_of_its_own_size() {
    let limits = limits();
    let mut world = World::new(limits);
    world.request(Request::Open { owner: Token::new(1), root: world.root });
    assert_eq!(world.pop(), Event::Opened { owner: Token::new(1), last: 0 }, "a new store opens at number 0");

    for (index, key) in (*b"abcdef").into_iter().enumerate() {
        let number = u64::try_from(index + 1).expect("six commits fit u64");
        let owner = Token::new(number + 1);
        let request = Request::Commit {
            owner,
            ops: Box::new([Op::Put { key: Box::from([key]), value: Box::from(&b"value"[..]) }]),
        };
        if index < 3 {
            world.request(request);
        } else {
            world.request_no_fire(request);
        }
        assert_eq!(world.pop(), Event::Committed { owner, number }, "commit {number} is durable");
    }
    world.request(Request::Close { owner: Token::new(8) });
    assert_eq!(world.pop(), Event::Closed { owner: Token::new(8) }, "the store closes before measuring its open");

    let files = world.files();
    assert!(files.contains_key(&b"snapshot"[..]), "the first three commits leave a snapshot");
    let mut file_bytes = 0_u64;
    let mut nonempty_files = 0_u64;
    for (name, bytes) in &files {
        if name.as_ref() == b"snapshot" || name.starts_with(b"log-") {
            file_bytes += u64::try_from(bytes.len()).expect("small file length fits u64");
            nonempty_files += u64::from(!bytes.is_empty());
        }
    }
    assert!(nonempty_files >= 2, "recovery reads a nonempty snapshot and log: {nonempty_files} files");

    world.asked_reads = 0;
    world.asked_bytes = 0;
    let span = Span::start();
    world.reopen();
    let grown = span.end();

    assert_eq!(world.pop(), Event::Opened { owner: Token::new(1), last: 6 }, "recovery opens at the last commit, 6");
    assert_eq!(world.asked_bytes, file_bytes, "requested bytes equal the snapshot and log bytes, {file_bytes}");
    assert!(
        world.asked_reads >= nonempty_files,
        "{} requested reads cover {nonempty_files} nonempty files",
        world.asked_reads
    );
    assert!(
        grown.peak < i64::from(limits.chunk / 2),
        "open's heap peak {} stays below half of chunk {}",
        grown.peak,
        limits.chunk
    );
    eprintln!(
        "open: file bytes {file_bytes}, asked bytes {}, asked reads {}, nonempty files {nonempty_files}, heap peak {}",
        world.asked_bytes, world.asked_reads, grown.peak
    );
}
