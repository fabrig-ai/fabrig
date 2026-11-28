//! Dev's registered layouts open by read conversion (temper/docs/design/protocol/floor.md, 6).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary Rust directory-store fixtures"
)]
use super::{
    dev_layouts::{self as dev, DEV, Dev},
    directory_jobs::{SETS, component, disk::Disk, header},
    fixtures,
};
use jig_protocol_store::{self as store, Event, Get, OwnerSet, OwnerSets, Request, Row, Store, Write};
use skein_codec::{Record as _, schema};
use skein_lib::Token;
use temper_engine_domain_forge_vocabulary as d;

// Dev holds both change parts, but no person briefs, limit holds, branch writes or repository revisions.
// Of those, only limit holds are not registered in the current set.
const NEW_TYPES: usize = 3;
const fn after_dev(from: &[u8]) -> bool {
    from.len() == 1
        && (from[0] == crate::keys::PERSON_BRIEF
            || from[0] == crate::keys::LIMIT_HOLD
            || from[0] == crate::keys::WRITES
            || from[0] == crate::keys::TEMPER_REVISION)
}
const fn dev_types() -> [store::ValueType; crate::OWNER_SET.families[0].types.len() - NEW_TYPES] {
    let types = crate::OWNER_SET.families[0].types;
    let mut kept = [types[0]; crate::OWNER_SET.families[0].types.len() - NEW_TYPES];
    let mut read = 0;
    let mut written = 0;
    while read < types.len() {
        if !after_dev(types[read].keys[0].from) {
            kept[written] = types[read];
            written = written.checked_add(1).unwrap();
        }
        read = read.checked_add(1).unwrap();
    }
    assert!(written == kept.len());
    kept
}
const fn dev_keys() -> [store::Keys; crate::OWNER_SET.families[0].keys.len() - NEW_TYPES] {
    let keys = crate::OWNER_SET.families[0].keys;
    let mut kept = [keys[0]; crate::OWNER_SET.families[0].keys.len() - NEW_TYPES];
    let mut read = 0;
    let mut written = 0;
    while read < keys.len() {
        if !after_dev(keys[read].from) {
            kept[written] = keys[read];
            written = written.checked_add(1).unwrap();
        }
        read = read.checked_add(1).unwrap();
    }
    assert!(written == kept.len());
    kept
}
fn dev_sets() -> OwnerSets {
    const FAMILY: store::KeptFamily =
        store::KeptFamily { current: DEV, types: &dev_types(), keys: &dev_keys(), ..crate::OWNER_SET.families[0] };
    const APPLICATION: OwnerSet =
        OwnerSet { jobs: &[], families: &[FAMILY, crate::OWNER_SET.families[1]], ..crate::OWNER_SET };
    OwnerSets { sets: &[store::SKEIN_SET, store::SMITH_SET, jig_protocol_accounts::JIG_SET, APPLICATION] }
}
fn seed_dev(disk: &mut Disk, rows: Vec<store::JobRow>) {
    let sets = dev_sets();
    let text = schema::parse(DEV.as_bytes()).unwrap();
    for ty in dev_types() {
        assert!(text.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).is_some(), "{} {:?}", ty.root, ty.variant);
    }
    let mut before = component(disk, sets);
    let events = disk.open(&mut before);
    let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("first open: {events:?}")
    };
    let events = disk.request(&mut before, Request::Commit { number: 1, writes: Box::new([Write::Save(header(1))]) });
    let [Event::Committed { .. }] = events.as_slice() else { panic!("commit: {events:?}") };
    let writes = core::iter::once(Write::Save(header(2)))
        .chain(rows.into_iter().map(|row| Write::Put { key: row.key, value: row.value }))
        .collect();
    let events = disk.request(&mut before, Request::Commit { number: 2, writes });
    let [Event::Committed { .. }] = events.as_slice() else { panic!("commit: {events:?}") };
    drop(before);
    disk.restart();
}
fn unchanged_except_recovery(
    before: &alloc::collections::BTreeMap<Vec<u8>, Vec<u8>>,
    after: &alloc::collections::BTreeMap<Vec<u8>, Vec<u8>>,
    log_prefix: &[u8],
) {
    for (path, bytes) in before {
        assert!(after.get(path) == Some(bytes), "stored file unchanged: {}", String::from_utf8_lossy(path));
    }
    // KV recovery makes an empty segment before the open plans its registry.
    // It may add that file, never alter a source file or create a generation.
    for (path, bytes) in after {
        if !before.contains_key(path) {
            assert!(path.starts_with(log_prefix) && !path.ends_with(b"/"), "{}", String::from_utf8_lossy(path));
            assert!(bytes.is_empty(), "recovery only adds an empty log segment");
        }
    }
}
fn read_rows(disk: &mut Disk, store: &mut Store, expected: &[d::Stored]) {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for expected in expected {
        let key = crate::keys::encode(&d::stored_key(expected), u32::MAX).unwrap();
        let events = disk.request(store, Request::Get { owner: Token::new(1), key: Get::Application(key.clone()) });
        let [Event::Got { row: Some(Row::Application { key: got, value }), .. }] = events.as_slice() else {
            panic!("readback: {events:?}")
        };
        assert_eq!(got, &key);
        crate::v25::Row::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(value)).unwrap();
        assert_eq!(crate::records::decode(value, &bounds).unwrap(), *expected);
    }
}
#[test]
fn dev_rows_open_in_a_new_generation_and_reopen_without_writing() {
    let mut disk = Disk::new();
    let mut dev = Dev::new();
    let cases = [dev::landing_log(), dev::change_cold(), dev::finding(), dev::tier()];
    let mut rows: Vec<_> = cases
        .iter()
        .zip(["landing_log", "change_cold", "finding", "tier"])
        .map(|((stored, _), variant)| dev.row(stored, variant))
        .collect();
    // The cleanup job must find this repository present and keep its tier/log.
    rows.push(dev.row(&fixtures::adopted(d::Protection::Absent), "repository"));
    seed_dev(&mut disk, rows);
    assert_eq!(disk.bytes(b"active"), b"1\n");
    let source = disk.snapshot(b"generations/1");
    assert!(!source.is_empty());
    let mut current = component(&disk, SETS);
    let bounds = crate::bounds(&fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
    let mut events = alloc::collections::VecDeque::from(disk.open(&mut current));
    let mut ran = Vec::new();
    let mut last = None;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: store::Owner::Application, job, page } => {
                let present = crate::jobs::read_present(&mut current, &bounds).unwrap();
                let answer = crate::jobs::page(&job, &page, &present, &bounds);
                events.extend(disk.request(&mut current, Request::Job { owner, answer }));
            }
            Event::Ran { of: store::Owner::Application, job, counts, .. } => {
                let declaration = crate::OWNER_SET.jobs.iter().find(|decl| decl.name == job.as_ref()).unwrap();
                assert_eq!(counts.iter().map(|count| count.name.as_ref()).collect::<Vec<_>>(), declaration.counts);
                assert!(counts.iter().all(|count| count.count == 0), "{job}: {counts:?}");
                ran.push(job);
            }
            Event::Opened { pending: true, last: number, .. } => {
                assert!(last.replace(number).is_none());
            }
            Event::Census { .. } => {}
            other => panic!("dev open: {other:?}; {:?}", current.start_open_failure()),
        }
    }
    let last = last.expect("opened with pending registry/job writes");
    // The engine's Link step uses this flag; it isn't a store job.
    assert!(crate::introduced_branch_writes(&current));
    assert!(current.introduced_value_type("temper-store", 53));
    assert_eq!(
        ran.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
        [
            "test-ids-landing",
            "test-ids-receipt",
            "test-ids-tier",
            "test-ids-verification",
            "deleted-repository-forge-rows"
        ]
    );
    let events = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = events.as_slice() else { panic!("switch: {events:?}") };
    assert_eq!(disk.bytes(b"active"), b"2\n", "registration writes a new generation");
    unchanged_except_recovery(&source, &disk.snapshot(b"generations/1"), b"generations/1/store/log-");
    let expected: Vec<_> = cases.into_iter().map(|(_, expected)| expected).collect();
    read_rows(&mut disk, &mut current, &expected);
    let opened = disk.snapshot(b"generations");
    drop(current);
    disk.restart();
    let mut again = component(&disk, SETS);
    let events = disk.open(&mut again);
    let [Event::Opened { last: found, pending: false, .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("second open writes no jobs or registry: {events:?}")
    };
    assert_eq!(*found, last);
    assert!(!crate::introduced_branch_writes(&again));
    assert!(!again.introduced_value_type("temper-store", 53));
    assert_eq!(disk.bytes(b"active"), b"2\n");
    unchanged_except_recovery(&opened, &disk.snapshot(b"generations"), b"generations/2/store/log-");
    read_rows(&mut disk, &mut again, &expected);
}
