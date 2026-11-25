//! Openings reads the recovered verdict rows, never the task logs (protocol/store/opening-rehearsal.md, 1).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::match_like_matches_macro,
    reason = "step test fixtures and event assertions are ordinary Rust"
)]
use super::fixtures::{
    self,
    disk::{Disk, Seen},
};
use crate::{Event, Generations, KeptOpening, KeptWire, KeptWorker, OpenRefusal, Reach, Request, WorkerOpening, Write};
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
        wires: Box::new([crate::OpenedWire {
            name: "fixture-wire".into(),
            layout: [7; 32],
            text: b"kept text".as_slice().into(),
        }]),
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
    let Some(Event::Rehearsing { wires, openings, workers, linked: 0, copied, .. }) = whole_events.first() else {
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
    assert_eq!(openings.as_ref(), &[opening("w1"), opening("w2")].map(KeptOpening::Current));
    assert_eq!(workers.as_deref(), Some([].as_slice()));
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
    let [
        Event::Rehearsing { wires: found_wires, openings: found_openings, workers: found_workers, .. },
        Event::Closed { removed: true, .. },
    ] = events.as_ref()
    else {
        panic!("{events:?}")
    };
    assert_eq!(found_wires, wires);
    assert_eq!(found_openings, openings);
    assert_eq!(found_workers, workers);
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
        assert_eq!(
            openings.as_ref(),
            [opening("w1"), opening("w2")].map(KeptOpening::Current),
            "scratch recovered the rows"
        );
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

fn registered(disk: &Disk) -> crate::Store {
    let mut store = disk.store_with_limits(&fixtures::registered_store_limits());
    store.owner_sets(crate::OwnerSets { sets: &[crate::SKEIN_SET, crate::SMITH_SET, crate::JIG_STORE_SET] });
    store
}

fn rehearse_registered(disk: &mut Disk, reach: Reach) -> Box<[Event]> {
    let mut store = registered(disk);
    let directory = disk.root();
    let scratch = disk.scratch_root();
    disk.request(
        &mut store,
        Request::Rehearse { owner: Token::new(1), directory, scratch, installation: fixtures::INSTALLATION, reach },
    )
}

fn raw(disk: &mut Disk, store: &mut crate::Store, key: Box<[u8]>, value: Box<[u8]>) {
    let number = store.stores.store.last().checked_add(1).unwrap();
    let header = fixtures::header_record(number);
    let header_key = crate::keys::encode(&header.key(), store.limits.store.key).unwrap();
    let mut header_value = crate::codec::encode(&header, &store.codec_limits, &store.names).unwrap();
    store.registry.write(&header_key, &mut header_value).unwrap();
    let ops = Box::new([skein_kv::Op::Put { key: header_key, value: header_value }, skein_kv::Op::Put { key, value }]);
    let env = crate::memory_env(store.limits.store);
    let mut above = skein_lib::Queue::uncapped();
    let mut below = fixtures::below();
    skein_kv::down(
        &mut store.stores.store,
        &env,
        skein_kv::Request::Commit { owner: Token::new(1), ops },
        &mut above,
        &mut below.store,
    );
    disk.drive(&mut below, |io, below| {
        let crate::Io::Store(arrived) = io else { panic!("raw public-map write") };
        skein_kv::up(&mut store.stores.store, &env, arrived.event, &mut above, &mut below.store);
    });
    let Some(skein_kv::Event::Committed { .. }) = above.pop() else { panic!("raw write: {above:?}") };
}

#[test]
fn an_earlier_openings_head_is_enough_for_both_reaches_and_the_next_start_clears_it() {
    let mut source = Disk::new(&[]);
    let mut store = registered(&source);
    let events = source.open(&mut store);
    let Some(Event::Opened { .. }) = events.first() else { panic!("{events:?}") };
    let writes = Box::new([fixtures::header(1), Write::Wires, Write::Opening(opening("current"))]);
    let events = source.request(&mut store, fixtures::commit(1, writes));
    let [Event::Committed { .. }] = events.as_ref() else { panic!("{events:?}") };
    let old = crate::keys::encode_own(&crate::keys::Own::Opening("earlier".into()), store.limits.store.key).unwrap();
    // Nothing resembling a body, so even attempting a v1 decode would refuse it.
    raw(&mut source, &mut store, old.clone(), Box::new([0, 1]));
    let items = source.items();
    for reach in [Reach::Openings, Reach::Whole] {
        let events = rehearse_registered(&mut Disk::new(&items), reach);
        let Some(Event::Rehearsing { openings, .. }) = events.first() else { panic!("{events:?}") };
        assert_eq!(
            openings.as_ref(),
            [KeptOpening::Current(opening("current")), KeptOpening::BeforeLayouts { worker: "earlier".into() },]
        );
        if reach == Reach::Whole {
            let [.., Event::Opened { .. }, Event::Census { .. }] = events.as_ref() else { panic!("{events:?}") };
        }
    }
    let mut disk = Disk::new(&items);
    let mut store = registered(&disk);
    let events = disk.open(&mut store);
    let Some(Event::Opened { last: 2, .. }) = events.first() else { panic!("{events:?}") };
    let events = disk.request(&mut store, fixtures::commit(3, Box::new([fixtures::header(3), Write::Wires])));
    let [Event::Committed { .. }] = events.as_ref() else { panic!("{events:?}") };
    let page = crate::pages::load(&mut store, skein_kv::Range::prefix(&[0, 5])).unwrap();
    assert!(page.rows.is_empty(), "both openings are cleared with the start's wires");
}

#[test]
fn both_reaches_read_worker_changes_in_number_order_and_report_an_undecodable_change() {
    use jig_domain_core_vocabulary::{self as core, settings as s};
    let mut disk = Disk::new(&[]);
    let mut store = registered(&disk);
    let events = disk.open(&mut store);
    let Some(Event::Opened { .. }) = events.first() else { panic!("{events:?}") };
    let expected = [
        KeptWorker { number: 1, name: b"changed".as_slice().into(), withdrawn: false },
        KeptWorker { number: 2, name: b"added".as_slice().into(), withdrawn: false },
        KeptWorker { number: 3, name: b"withdrawn".as_slice().into(), withdrawn: true },
    ];
    let setting = |worker: &KeptWorker| {
        s::Value::Worker(Box::new(s::WorkerSetting {
            number: worker.number,
            name: worker.name.clone(),
            places: 1,
            digest: Some([5; 32]),
            withdrawn: worker.withdrawn,
        }))
    };
    let default = s::Record::Default {
        setting: s::Setting::Worker(1),
        default: s::Default { revision: 1, value: setting(&expected[0]) },
    };
    let save = |record| Write::Save(core::Record::Core(core::CoreRecord::Settings(record)));
    let writes = Box::new([fixtures::header(1), Write::Wires, save(default)]);
    let events = disk.request(&mut store, fixtures::commit(1, writes));
    let [Event::Committed { .. }] = events.as_ref() else { panic!("{events:?}") };
    let mut writes = vec![fixtures::header(2)];
    for worker in expected.iter().rev() {
        writes.push(save(s::Record::Changed {
            setting: s::Setting::Worker(worker.number),
            changed: s::Changed { revision: 2, value: Some(setting(worker)) },
        }));
    }
    let events = disk.request(&mut store, fixtures::commit(2, writes.into_boxed_slice()));
    let [Event::Committed { .. }] = events.as_ref() else { panic!("{events:?}") };
    let items = disk.items();
    for reach in [Reach::Openings, Reach::Whole] {
        let events = rehearse_registered(&mut Disk::new(&items), reach);
        let Some(Event::Rehearsing { workers, .. }) = events.first() else { panic!("{events:?}") };
        assert_eq!(workers.as_deref(), Some(expected.as_slice()));
    }
    let key = core::Key::Core(core::CoreKey::Settings(s::Key::Changed(s::Setting::Worker(2))));
    let key = crate::keys::encode(&key, store.limits.store.key).unwrap();
    raw(&mut disk, &mut store, key, Box::new([0, 3]));
    let items = disk.items();
    for reach in [Reach::Openings, Reach::Whole] {
        let events = rehearse_registered(&mut Disk::new(&items), reach);
        let Some(Event::Rehearsing { workers: None, .. }) = events.first() else { panic!("{events:?}") };
        match reach {
            Reach::Openings => {
                let Some(Event::Closed { removed: true, .. }) = events.last() else { panic!("{events:?}") };
            }
            Reach::Whole => {
                let Some(Event::Undecodable { family, .. }) = events.last() else { panic!("{events:?}") };
                assert_eq!(family.as_ref(), "jig-store-settings");
            }
        }
    }
}
