//! Read-only restore and inventory on the simulated disk (store.md, 7 and 14).
use jig_domain_core as core;
use jig_protocol_store::{Restore, Row, SecretOwner, SecretValue, Secrets, Write, codec, keys, restore::Item, sizes};
use jig_store_world::{World, root};
use std::collections::BTreeMap;

const SEED: u64 = 19;
const PAGE_ROWS: u32 = 2;
const PAGE_BYTES: u64 = 65_536;

fn committed() -> World {
    let mut world = World::new(SEED);
    world.commit(root::first()).expect("first commit");
    world
}

fn inventory(world: &mut World, rows: u32, bytes: u64) -> BTreeMap<Box<[u8]>, Box<[u8]>> {
    let mut found = BTreeMap::new();
    let mut after = None;
    loop {
        let page = world.retained_page(after.as_deref(), rows, bytes).expect("retained page");
        let replay = world.retained_page(after.as_deref(), rows, bytes).expect("same exclusive place");
        assert_eq!(page.rows, replay.rows);
        assert_eq!(page.next, replay.next);
        for row in page.rows {
            assert!(after.as_deref().is_none_or(|after| row.key.as_ref() > after));
            assert!(found.insert(row.key, row.value).is_none(), "no duplicate keys");
        }
        after = page.next;
        if after.is_none() {
            break;
        }
    }
    found
}

fn empty_wires() -> Box<[u8]> {
    use jig_protocol_store::component_v3 as wire;
    let wires = wire::Wires::new(&wire::CEILINGS, wire::WiresParts { wires: skein_lib::List::with_capacity(0) })
        .expect("empty wires");
    let mut writer = skein_lib::Writer::new(usize::try_from(wires.measure()).expect("wires measure"));
    wires.encode(&mut writer).expect("measured wires");
    writer.finish()
}

fn raw(record: &core::Record) -> (Box<[u8]>, Box<[u8]>) {
    let limits = root::limits();
    (
        keys::encode(&record.key(), limits.store.key).expect("source key"),
        codec::encode(record, &codec::limits(&limits.core).expect("source limits"), &[]).expect("source value"),
    )
}

#[test]
fn inventory_reads_component_installation_domain_and_application_without_secrets() {
    let mut world = committed();
    let application = Box::from(b"retained".as_slice());
    let value: Box<[u8]> = Box::from(b"source bytes".as_slice());
    let source = root::live_task(root::LIVE);
    world
        .commit(Box::new([Write::Save(root::header(2)), Write::Put { key: application, value: value.clone() }]))
        .expect("application commit");
    world.secret(Secrets::Keep {
        owner: SecretOwner::Application,
        number: 42,
        value: SecretValue::Application(skein_lib::Secret::new(Box::from(b"secret-map-only".as_slice()))),
    });
    let before = world.contents();
    let found = inventory(&mut world, 1, PAGE_BYTES);
    let expected = BTreeMap::from([
        (
            keys::encode_own(&keys::Own::Format, root::limits().store.key).expect("format key"),
            jig_protocol_store::format::current(root::APPLICATION.version, root::DEPLOYMENT).encode().expect("format"),
        ),
        (
            keys::encode_own(&keys::Own::Layouts, root::limits().store.key).expect("layout key"),
            jig_protocol_store::format::layouts().encode(),
        ),
        (
            keys::installation(root::limits().store.key).expect("installation key"),
            codec::accounts::Installation { id: world.installation }.encode(),
        ),
        raw(&root::header(2)),
        raw(&source),
        raw(&root::run_proof(root::LIVE)),
        (keys::encode_own(&keys::Own::Wires, root::limits().store.key).expect("wires key"), empty_wires()),
        (keys::application(b"retained", root::limits().store.key).expect("application key"), value),
    ]);
    assert_eq!(found, expected);
    assert_eq!(world.contents(), before, "reader issues no durable effects");
    assert!(!found.values().any(|value| value.as_ref() == b"secret-map-only"));
}

#[test]
fn retained_readback_exposes_extra_missing_and_changed_commits() {
    let mut world = committed();
    let source = b"source".as_slice();
    let expected =
        BTreeMap::from([(keys::application(b"one", root::limits().store.key).expect("key"), Box::from(source))]);
    // Inspect only the independently committed application keyspace.
    for (index, mut writes) in [
        vec![Write::Put { key: Box::from(b"one".as_slice()), value: Box::from(source) }],
        vec![Write::Put { key: Box::from(b"extra".as_slice()), value: Box::from(source) }],
        vec![
            Write::Remove { key: Box::from(b"extra".as_slice()) },
            Write::Remove { key: Box::from(b"one".as_slice()) },
        ],
        vec![Write::Put { key: Box::from(b"one".as_slice()), value: Box::from(b"changed".as_slice()) }],
    ]
    .into_iter()
    .enumerate()
    {
        let number = u64::try_from(index).expect("four commits") + 2;
        writes.insert(0, Write::Save(root::header(number)));
        world.commit(writes.into_boxed_slice()).expect("variant commit");
        let found: BTreeMap<_, _> = inventory(&mut world, PAGE_ROWS, PAGE_BYTES)
            .into_iter()
            .filter(|(key, _)| key.first() == Some(&keys::APPLICATION))
            .collect();
        assert_eq!(found == expected, index == 0, "committed variant {index}");
    }
}

#[test]
fn restore_restore_serves_maximum_writer_rows_above_party_read_limits() {
    let mut limits = root::limits();
    limits.core.load_bytes = 1;
    let mut world = World::unopened_observed(SEED, limits, None);
    world.open().expect("open");
    world.commit(root::first()).expect("first commit");
    let first = vec![b'a'; usize::try_from(root::APPLICATION.key).expect("application key")].into_boxed_slice();
    let second = vec![b'b'; first.len()].into_boxed_slice();
    let value = vec![7; usize::try_from(root::APPLICATION.value).expect("application value")].into_boxed_slice();
    world
        .commit(Box::new([
            Write::Save(root::header(2)),
            Write::Put { key: first.clone(), value: value.clone() },
            Write::Put { key: second.clone(), value: value.clone() },
        ]))
        .expect("maximum writer rows");
    world.crash(SEED);
    world.open().expect("cold open");
    let bytes = sizes::restore_bytes(&limits).expect("writer page bytes");
    let mut after = None;
    for (index, key) in [first, second].into_iter().enumerate() {
        let page = world
            .restore_page(Restore::Application(Box::new([skein_kv::Range::prefix(b"")])), after.as_ref(), 1, bytes)
            .expect("writer row restores");
        assert!(matches!(page.items.as_ref(), [Item::Row(Row::Application { key: found, value: found_value })]
            if *found == key && *found_value == value));
        assert_eq!(page.next.is_some(), index == 0);
        after = page.next;
    }
}

#[test]
fn fixed_pages_hold_bounded_memory_as_inventory_grows() {
    let mut peaks = Vec::new();
    // Both inventories fit the existing world's 65,536-byte main map.
    for (selected, count) in [(false, 10_u64), (false, 100), (true, 10), (true, 100)] {
        let mut world = if selected {
            let mut world = World::unopened(SEED);
            world.start_restore();
            world.open().expect("selected open");
            world.commit(root::first()).expect("first commit");
            world
        } else {
            committed()
        };
        let mut writes = vec![Write::Save(root::header(2))];
        for number in 0..count {
            writes.push(Write::Put { key: number.to_be_bytes().into(), value: Box::new([42]) });
        }
        world.commit(writes.into_boxed_slice()).expect("inventory growth commits");
        let meter = skein_heap::Meter::new();
        meter.start();
        let page = world.retained_page(None, PAGE_ROWS, PAGE_BYTES).expect("bounded inventory page");
        let measured = meter.end();
        eprintln!(
            "selected={selected} retained page over {count} application rows: peak {} held {}",
            measured.peak(),
            measured.held()
        );
        assert_eq!(page.rows.len(), usize::try_from(PAGE_ROWS).expect("page rows"));
        peaks.push(measured.peak());
        let meter = skein_heap::Meter::new();
        meter.start();
        let restored = world
            .restore_page(Restore::Application(Box::new([skein_kv::Range::prefix(b"")])), None, PAGE_ROWS, PAGE_BYTES)
            .expect("bounded restore page");
        let measured = meter.end();
        eprintln!(
            "selected={selected} restore page over {count} application rows: peak {} held {}",
            measured.peak(),
            measured.held()
        );
        assert_eq!(restored.items.len(), usize::try_from(PAGE_ROWS).expect("page rows"));
        peaks.push(measured.peak());
    }
    for pair in peaks.chunks_exact(2) {
        assert_eq!(&peaks[..2], pair, "page memory does not grow with the inventory or selection");
    }
}

#[test]
fn census_pairs_with_writer_sized_payloads_fit_exact_decoded_bytes() {
    use jig_domain_tasks as tasks;
    let mut limits = root::limits();
    limits.core.load_bytes = 1;
    let core::Record::Tasks(tasks::Stored::Live(mut task)) = root::live_task(root::LIVE) else {
        panic!("task fixture")
    };
    task.spec.words =
        vec![b'w'; usize::try_from(limits.core.tasks.spec_bytes).expect("writer words")].into_boxed_slice();
    task.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
        words: vec![b'r'; usize::try_from(limits.core.tasks.result_bytes).expect("writer result")].into_boxed_slice(),
    }));
    task.ended_at = Some(skein_lib::Wall::EPOCH);
    let ended = core::Record::Tasks(tasks::Stored::Ended(task));
    let position =
        core::Record::People(jig_domain_people::Stored::EndedRead { task: root::LIVE, party: 1, through: 1 });
    let bytes = u64::try_from(raw(&ended).1.len() + raw(&position).1.len()).expect("pair decoded bytes");
    let mut world = World::unopened_observed(SEED, limits, None);
    world.open().expect("open");
    world
        .commit(Box::new([Write::Save(root::header(1)), Write::Save(ended.clone()), Write::Save(position.clone())]))
        .expect("writer pair commits");
    let page = world
        .restore_page(
            Restore::Jig(Box::new([core::RestoreRange::Census(core::Census {
                results: Box::new([]),
                questions: Box::new([]),
            })])),
            None,
            PAGE_ROWS,
            bytes,
        )
        .expect("exact pair bytes");
    let records: Vec<_> = page
        .items
        .into_iter()
        .map(|item| match item {
            Item::Row(Row::Jig(record)) => record,
            Item::Row(Row::Application { .. }) | Item::MissingLink { .. } => panic!("joined domain row"),
        })
        .collect();
    assert_eq!(records, [position, ended]);
    assert!(page.next.is_none());
}

#[test]
fn selected_open_stays_within_the_small_profile_open_ceiling() {
    let limits = jig_protocol_config::profile(jig_protocol_config::Profile::Small).store;
    let meter = skein_heap::Meter::new();
    meter.start();
    let mut world = World::unopened_observed(SEED, limits, None);
    world.start_restore();
    let opened = world.open();
    let measured = meter.end();
    // The existing small-profile construction/open ceiling (orphan_memory.rs).
    let peak = meter.check(measured, 96_000, &"selected small store construction through open");
    assert_eq!(opened, Some(0));
    eprintln!("selected small store construction through open: {peak} bytes (ceiling 96000)");
}
