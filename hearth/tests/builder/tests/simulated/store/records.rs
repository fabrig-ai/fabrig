use hearth_builder_world::{fixtures, protocol};
use hearth_protocol_tree::{
    cache_file::{self, Origin},
    limits, store as record,
};
use skein_lib::{Duration, Wall};

pub(super) const WALL: Wall = Wall::from_nanos(1_760_000_000_000_000_000);
const LATER: Wall = Wall::from_nanos(1_760_000_001_000_000_000);
pub(super) const CACHE: &[u8] = b"/srv/store/cache";
pub(super) const STORE_RECORD: &[u8] = b"/srv/store/store.record";
pub(super) const STORE_NAMES: &[&str] = &["cache", "resolutions", "staging", "store.record"];
pub(super) const THIN_CACHE_NAMES: &[&str] = &[
    "42ecbc10b6791c87aa0ca833e522946a40a7eece2297a745c5c10cea07baea96",
    "42ecbc10b6791c87aa0ca833e522946a40a7eece2297a745c5c10cea07baea96.record",
    "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
    "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123.record",
    "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745",
    "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745.record",
];
pub(super) const STATIC: &[StaticFile] = &[
    StaticFile {
        name: "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
        size: 15,
        oid: b"d23c59a84714a16a9597f6d92ce69390d6d13f9b",
        body: b"Welcome to x1.\n",
    },
    StaticFile {
        name: "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745",
        size: 29,
        oid: b"71318968f52ccfeabf6edd13442c868eaa35e49d",
        body: b"#!/bin/sh\nexec cat /etc/motd\n",
    },
];
pub(super) struct StaticFile {
    pub(super) name: &'static str,
    size: u64,
    oid: &'static [u8],
    body: &'static [u8],
}

pub(super) fn run(store: &mut protocol::Store<'_>) -> protocol::Run {
    store.run(WALL, &hearth::limits())
}

pub(super) fn cache_bytes(run: &protocol::Run, name: &str) -> Option<Vec<u8>> {
    run.shared.contents(format!("/srv/store/cache/{name}").as_bytes())
}

pub(super) fn cache_mode(run: &protocol::Run, name: &str) -> u32 {
    run.shared
        .borrow()
        .behind
        .machine()
        .linux()
        .facts(format!("/srv/store/cache/{name}").as_bytes())
        .expect("cached file")
        .mode
}

#[test]
fn the_first_render_lays_the_store_and_cache_records_and_the_second_keeps_them() {
    let scenario = fixtures::thin();
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let first = run(&mut store);
    assert_eq!(first.code, Some(0));
    assert_eq!(store.names(fixtures::STORE), STORE_NAMES);
    let bytes = first.shared.contents(STORE_RECORD).expect("first store record");
    let record = record::read(&bytes, &limits::SHIPPED).expect("kind 1");
    assert_eq!(record.layout, 1);
    assert_eq!(record.range, None);
    assert_eq!(record.uid, fixtures::BUILDER.user);

    assert_eq!(store.names(CACHE), THIN_CACHE_NAMES, "exact entries, no temporary");
    let records: Vec<_> = STATIC
        .iter()
        .map(|file| {
            let name = file.name;
            assert_eq!(cache_bytes(&first, name).as_deref(), Some(file.body), "{name}");
            assert_eq!(cache_mode(&first, name), 0o644, "{name}");
            let path = format!("{name}.record");
            let bytes = cache_bytes(&first, &path).expect("first cache record");
            let record = cache_file::read(&bytes, &limits::SHIPPED).expect("kind 5");
            assert_eq!(fixtures::hex_digest(record.sha256.0), name);
            assert_eq!(record.size, file.size, "{name}");
            assert_eq!(record.origin, Origin::Blob { oid: file.oid.into() }, "{name}");
            assert_eq!(record.at, WALL, "{name}");
            assert_ne!(record.took, Duration::ZERO, "{name}");
            (path, bytes)
        })
        .collect();

    let second = store.run(LATER, &hearth::limits());
    assert_eq!(second.code, Some(0));
    assert_eq!(second.shared.contents(STORE_RECORD), Some(bytes));
    for (name, bytes) in records {
        assert_eq!(cache_bytes(&second, &name).as_deref(), Some(bytes.as_slice()), "{name}");
    }
}
