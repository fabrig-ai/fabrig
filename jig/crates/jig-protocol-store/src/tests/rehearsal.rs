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
    let Some(Event::Rehearsing { wires, openings, linked: 0, copied, .. }) = whole_events.first() else {
        panic!("{whole_events:?}")
    };
    let files = before
        .iter()
        .filter(|(path, kind, _, _)| {
            *kind == skein_io::kernel::Kind::File
                && (path.starts_with(b"generations/1/store/") || path.starts_with(b"generations/1/secrets/"))
        })
        .count();
    assert_eq!(usize::try_from(*copied).expect("fixture count fits"), files);
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

#[test]
fn openings_links_the_same_files_as_a_patch() {
    let mut items = source().items();
    // Compare their common reach: patches also link task logs, which Openings skips.
    items.retain(|item| !item.path.starts_with(b"generations/1/store/logs"));
    let mut patch = Disk::new(&items);
    assert_eq!(
        super::operations::copy(&mut patch, 1, fixtures::store_limits().store.chunk),
        crate::generations::Progress::Done
    );
    for refused in [false, true] {
        let mut disk = Disk::new(&items);
        let before = disk.contents();
        let files = before
            .iter()
            .filter(|(path, kind, _, _)| {
                *kind == skein_io::kernel::Kind::File
                    && (path.starts_with(b"generations/1/store/") || path.starts_with(b"generations/1/secrets/"))
                    && !path.starts_with(b"generations/1/store/logs/")
            })
            .count();
        if refused {
            disk.refuse_link_once(b"snapshot", Error::Other(18)); // EXDEV
        }
        let events = rehearse(&mut disk, Reach::Openings);
        let [Event::Rehearsing { wires: Some(_), openings, linked, copied, .. }, Event::Closed { removed: true, .. }] =
            events.as_ref()
        else {
            panic!("{events:?}")
        };
        assert_eq!(*linked, if refused { LINKS_AFTER_REFUSAL } else { LINKED_FILES });
        assert_eq!(usize::try_from(linked + copied).expect("fixture count fits"), files);
        assert_eq!(openings.as_ref(), [opening("w1"), opening("w2")], "scratch recovered the rows");
        // Link requests include the refused attempt; only the reported counts change.
        assert_eq!(disk.linked(), patch.linked());
        assert_eq!(disk.contents(), before, "recovery, fallback and close leave the source unchanged");
    }
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
