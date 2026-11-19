//! Openings reads the recovered verdict rows, never the task logs (opening.md, 5).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::match_like_matches_macro,
    reason = "step test fixtures and event assertions are ordinary Rust"
)]
use super::fixtures::{
    self,
    disk::{Disk, Seen},
};
use crate::{Event, Generations, KeptWire, OpenRefusal, Reach, Request, WorkerOpening, Write};
use skein_fake_machine::Item;
use skein_io::kernel::Error;
use skein_lib::{Token, Wall};

// source() seals a snapshot and two segments, leaving its third segment writable.
const LINKED_FILES: u32 = 3;
// Refusing the snapshot leaves only the two sealed segments linked.
const LINKS_AFTER_REFUSAL: u32 = 2;

fn opening(name: &str) -> WorkerOpening {
    WorkerOpening {
        worker: name.into(),
        wires: Box::new([crate::OpenedWire { name: "fixture-wire".into(), lowest: 1, layouts: Box::new([[7; 32]]) }]),
        output: 4096,
        terms: Box::new([crate::OpenedTerm { kind: 3, largest: 64 }]),
        at: Wall::from_nanos(17),
    }
}

fn source() -> Disk {
    // Recovery seals the torn, empty map with a snapshot before the first commit.
    let mut disk = Disk::new(&[
        Item::file(b"active", b"1\n"),
        Item::directory(b"generations").mode(0o700),
        Item::directory(b"generations/1").mode(0o700),
        Item::directory(b"generations/1/store").mode(0o700),
        Item::directory(b"generations/1/secrets").mode(0o700),
        Item::file(b"generations/1/store/log-00000000000000000001", b"torn"),
    ]);
    let mut limits = fixtures::store_limits();
    limits.store.segment = 1; // Each of the three commits starts its own segment.
    let mut store = disk.store_with_limits(&limits);
    assert!(store.wires(&[crate::Wire { name: "fixture-wire", version: 1 }]));
    assert!({ if let Some(Event::Opened { .. }) = disk.open(&mut store).first() { true } else { false } });
    let commits: [(u64, Box<[Write]>); 3] = [
        (1, Box::new([fixtures::header(1), Write::Wires])),
        (2, Box::new([fixtures::header(2), Write::Opening(opening("w2")), Write::Opening(opening("w1"))])),
        (3, Box::new([fixtures::header(3), fixtures::save(fixtures::question_line(fixtures::GOAL, 7, 1))])),
    ];
    for (number, writes) in commits {
        let events = disk.request(&mut store, fixtures::commit(number, writes));
        assert!({ if let [Event::Committed { .. }] = events.as_ref() { true } else { false } }, "{events:?}");
    }
    let events = disk.request(
        &mut store,
        Request::Secrets {
            owner: Token::new(1),
            request: crate::Secrets::Keep {
                owner: crate::SecretOwner::Application,
                number: 1,
                value: crate::SecretValue::Application(skein_lib::Secret::new(Box::from(b"fixture secret".as_slice()))),
            },
        },
    );
    assert!({ if let [Event::Kept { .. }] = events.as_ref() { true } else { false } }, "{events:?}");
    disk
}

fn rehearse(disk: &mut Disk, reach: Reach) -> Box<[Event]> {
    let mut store = disk.store();
    let directory = disk.root();
    let scratch = disk.scratch_root();
    let events = disk.request(
        &mut store,
        Request::Rehearse { owner: Token::new(1), directory, scratch, installation: fixtures::INSTALLATION, reach },
    );
    if reach == Reach::Whole {
        let closed = disk.request(&mut store, Request::Close { owner: Token::new(1) });
        assert!(
            { if let [Event::Closed { removed: true, .. }] = closed.as_ref() { true } else { false } },
            "{closed:?}"
        );
    }
    events
}

#[test]
fn openings_matches_whole_rows_and_self_closes_without_copying_task_logs() {
    let items = source().items();
    let mut whole = Disk::new(&items);
    let before = whole.contents();
    assert!(
        before.iter().any(|(path, kind, _, _)| path.starts_with(b"generations/1/store/logs/")
            && *kind == skein_io::kernel::Kind::File)
    );
    let whole_events = rehearse(&mut whole, Reach::Whole);
    assert_eq!(whole.contents(), before);
    let Some(Event::Rehearsing { wires, openings, .. }) = whole_events.first() else { panic!("{whole_events:?}") };
    assert_eq!(*wires, Some(Box::from([KeptWire { name: "fixture-wire".into(), version: 1 }])));
    assert_eq!(openings.as_ref(), &[opening("w1"), opening("w2")]);
    assert!(
        whole_events.iter().any(|event| { if let Event::Opened { .. } = event { true } else { false } }),
        "{whole_events:?}"
    );
    assert!(
        whole
            .requests()
            .iter()
            .any(|(_, request)| { if let Seen::Make(path) = request { path.as_ref() == b"logs" } else { false } })
    );

    let mut shallow = Disk::new(&items);
    let events = rehearse(&mut shallow, Reach::Openings);
    let [Event::Rehearsing { wires: found_wires, openings: found_openings, .. }, Event::Closed { removed: true, .. }] =
        events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!(found_wires, wires);
    assert_eq!(found_openings, openings);
    assert!(
        !shallow
            .requests()
            .iter()
            .any(|(_, request)| { if let Seen::Make(path) = request { path.as_ref() == b"logs" } else { false } })
    );
    let log_names: Vec<_> = before
        .iter()
        .filter(|(path, kind, _, _)| {
            path.starts_with(b"generations/1/store/logs/") && *kind == skein_io::kernel::Kind::File
        })
        .map(|(path, _, _, _)| path.rsplit(|byte| *byte == b'/').next().unwrap())
        .collect();
    assert!(!shallow.requests().iter().any(|(_, request)| {
        if let Seen::Create { name, .. } = request { log_names.contains(&name.as_ref()) } else { false }
    }));
    assert_eq!(shallow.contents(), before, "source bytes, modes and tree unchanged");
}

/// The newest segment lies beyond a page, after two sealed segments and other files.
fn paged_items() -> Vec<Item> {
    let mut items = source().items();
    for number in 0..crate::generations::PAGE {
        let path = [
            b"generations/1/store/log-00000000000000000002-other-".as_slice(),
            skein_lib::Decimal::of(u64::from(number)).as_bytes(),
        ]
        .concat();
        items.push(Item::file(&path, b"other store file").mode(0o600));
    }
    items
}

fn paged_disk(items: &[Item]) -> Disk {
    let limits = fixtures::disk::file_limits();
    Disk::with_file_limits(items, skein_io::file_layer::Limits { entries: 128, ..limits })
}

fn linked(disk: &Disk) -> Vec<Box<[u8]>> {
    disk.requests()
        .iter()
        .filter_map(|(driver, request)| {
            if let Seen::Link { from, to } = request {
                assert_eq!(*driver, 2, "generation copy uses the transcripts driver");
                assert_eq!(from, to);
                Some(from.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Each copy reads without following links, creates exclusively 0o600, writes and syncs.
fn copied(disk: &Disk) -> Vec<(Box<[u8]>, u64)> {
    let mut copied = Vec::new();
    for (index, pair) in disk.requests().windows(2).enumerate() {
        let [(_, previous), (driver, Seen::Create { name, mode })] = pair else { continue };
        assert_eq!(*driver, 2);
        assert_eq!(previous, &Seen::ReadOpen { name: name.clone() });
        assert_eq!(*mode, 0o600);
        let transfer = disk.requests().iter().skip(index.checked_add(2).expect("trace index fits"));
        let mut read = Vec::new();
        let mut written = Vec::new();
        let mut synced = false;
        for (_, request) in transfer {
            if *request == Seen::SyncFile {
                synced = true;
                break;
            }
            if let Seen::Read { max } = request {
                read.push(*max);
            }
            if let Seen::Write { bytes } = request {
                written.push(*bytes);
            }
        }
        assert!(synced, "copied file synced");
        assert_eq!(read, written, "each bounded read written whole");
        copied.push((name.clone(), written.into_iter().map(u64::from).sum()));
    }
    copied.sort();
    copied
}

fn source_files(items: &[Item], reach: Reach, refused: Option<&[u8]>) -> Vec<(Box<[u8]>, u64)> {
    let mut files = Vec::new();
    for item in items {
        let skein_fake_machine::Made::File(bytes) = &item.made else { continue };
        let store = item.path.strip_prefix(b"generations/1/store/");
        let secrets = item.path.strip_prefix(b"generations/1/secrets/");
        let Some(name) = store.or(secrets) else { continue };
        if reach == Reach::Openings && store.is_some() {
            if name.starts_with(b"logs/") {
                continue;
            }
            if [b"snapshot".as_slice(), b"log-00000000000000000001", b"log-00000000000000000002"].contains(&name)
                && refused != Some(name)
            {
                continue;
            }
        }
        files.push((
            Box::from(name.rsplit(|byte| *byte == b'/').next().expect("file name")),
            u64::try_from(bytes.len()).expect("fixture length"),
        ));
    }
    files.sort();
    files
}

#[test]
fn openings_links_snapshot_and_sealed_segments_across_roots_and_pages() {
    let items = paged_items();
    let mut whole = paged_disk(&items);
    let before = whole.contents();
    let whole_events = rehearse(&mut whole, Reach::Whole);
    assert!(linked(&whole).is_empty(), "Whole copies even linkable files");
    assert_eq!(copied(&whole), source_files(&items, Reach::Whole, None));
    assert_eq!(whole.contents(), before);

    let mut disk = paged_disk(&items);
    let events = rehearse(&mut disk, Reach::Openings);
    let [
        Event::Rehearsing { wires, openings, linked: LINKED_FILES, copied: shallow_copied, .. },
        Event::Closed { removed: true, .. },
    ] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    let Some(Event::Rehearsing {
        wires: whole_wires, openings: whole_openings, linked: 0, copied: whole_copied, ..
    }) = whole_events.first()
    else {
        panic!("{whole_events:?}")
    };
    assert_eq!(
        usize::try_from(*whole_copied).expect("fixture count fits"),
        source_files(&items, Reach::Whole, None).len()
    );
    assert_eq!(
        usize::try_from(*shallow_copied).expect("fixture count fits"),
        source_files(&items, Reach::Openings, None).len()
    );
    assert_eq!(wires, whole_wires);
    assert_eq!(openings, whole_openings);
    assert_eq!(
        linked(&disk),
        [b"log-00000000000000000001".as_slice(), b"log-00000000000000000002", b"snapshot"].map(Box::from)
    );
    assert_eq!(copied(&disk), source_files(&items, Reach::Openings, None));
    assert_eq!(disk.contents(), before, "all source bytes, modes and names survive recovery and close");
}

#[test]
fn openings_copies_one_refused_link_and_still_self_closes() {
    let items = source().items();
    let mut disk = Disk::new(&items);
    let before = disk.contents();
    disk.refuse_link_once(b"snapshot", Error::Other(18)); // EXDEV
    let events = rehearse(&mut disk, Reach::Openings);
    let [
        Event::Rehearsing { wires: Some(_), openings, linked: LINKS_AFTER_REFUSAL, copied: count, .. },
        Event::Closed { removed: true, .. },
    ] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!(openings.as_ref(), [opening("w1"), opening("w2")]);
    assert_eq!(
        linked(&disk),
        [b"log-00000000000000000001".as_slice(), b"log-00000000000000000002", b"snapshot"].map(Box::from)
    );
    assert_eq!(
        usize::try_from(*count).expect("fixture count fits"),
        source_files(&items, Reach::Openings, Some(b"snapshot")).len()
    );
    assert_eq!(copied(&disk), source_files(&items, Reach::Openings, Some(b"snapshot")));
    assert_eq!(disk.contents(), before, "fallback and close leave every source file unchanged");
}

#[test]
fn openings_keeps_busy_and_unfinished_refusals_before_reading_rows() {
    for reach in [Reach::Whole, Reach::Openings] {
        let mut disk = Disk::new(&source().items());
        let before = disk.contents();
        let scratch = disk.scratch_root();
        let held = disk.second_root();
        let locked = disk.file(0, skein_io::file::Request::Lock { owner: Token::new(3), root: held.token });
        assert!({ if let skein_io::file::Event::Locked { .. } = locked.event { true } else { false } });
        let directory = disk.root();
        let mut store = disk.store();
        let events = disk.request(
            &mut store,
            Request::Rehearse { owner: Token::new(2), directory, scratch, installation: fixtures::INSTALLATION, reach },
        );
        assert!(
            {
                if let [
                    Event::Rehearsing { .. },
                    Event::OpenRefused { why: OpenRefusal::Generations { why: Generations::Busy }, .. },
                ] = events.as_ref()
                {
                    true
                } else {
                    false
                }
            },
            "{events:?}"
        );
        assert_eq!(disk.contents(), before);

        let mut items = source().items();
        items.push(Item::file(b"restore", b"outstanding intent"));
        let mut disk = Disk::new(&items);
        let before = disk.contents();
        let events = rehearse(&mut disk, reach);
        assert!(
            {
                if let [
                    Event::Rehearsing { .. },
                    Event::OpenRefused { why: OpenRefusal::Generations { why: Generations::Unfinished }, .. },
                ] = events.as_ref()
                {
                    true
                } else {
                    false
                }
            },
            "{events:?}"
        );
        assert_eq!(disk.contents(), before);
    }
}
