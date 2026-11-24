use super::records::{CACHE, STATIC, WALL, cache_bytes, cache_mode, run};
use hearth_builder_world::{fixtures, protocol};
use hearth_domain_store::limits::Limits;
use hearth_protocol_tree::{cache_file, limits};

const FULL_CACHE: &[fixtures::Laid] = &[
    fixtures::Laid::File("cache/0101010101010101010101010101010101010101010101010101010101010101"),
    fixtures::Laid::Record(
        "cache/0101010101010101010101010101010101010101010101010101010101010101.record",
        fixtures::D1,
    ),
    fixtures::Laid::File("cache/0202020202020202020202020202020202020202020202020202020202020202"),
    fixtures::Laid::Record(
        "cache/0202020202020202020202020202020202020202020202020202020202020202.record",
        hearth_lib::encode::Digest([2; 32]),
    ),
    fixtures::Laid::File("cache/0303030303030303030303030303030303030303030303030303030303030303"),
    fixtures::Laid::Record(
        "cache/0303030303030303030303030303030303030303030303030303030303030303.record",
        fixtures::D3,
    ),
];

#[test]
fn a_store_past_its_cache_limit_names_the_limit_and_refuses_every_new_file() {
    let scenario = fixtures::Scenario { laid: FULL_CACHE, ..fixtures::thin() };
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let limits = fixtures::with_store(Limits { cache_files: 2, ..hearth_domain_store::limits::SHIPPED });
    let run = store.run(WALL, &limits);
    assert_eq!(run.code, Some(75));
    assert_eq!(run.stdout, b"the store /srv/store is full: 2 cache files\n");
    assert_eq!(run.stderr, b"");
    for file in STATIC {
        assert_eq!(cache_bytes(&run, file.name), None, "{}", file.name);
    }
    assert_eq!(store.names(CACHE).len(), 6);
    assert_eq!(
        cache_bytes(&run, "0101010101010101010101010101010101010101010101010101010101010101").as_deref(),
        Some(b"fixture".as_slice())
    );
}

#[test]
fn a_cache_that_fills_while_rendering_removes_the_file_past_it() {
    let scenario = fixtures::thin();
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let limits = fixtures::with_store(Limits { cache_files: 1, ..hearth_domain_store::limits::SHIPPED });
    let run = store.run(WALL, &limits);
    assert_eq!(run.code, Some(75));
    assert_eq!(
        store.names(CACHE),
        [
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123.record",
        ]
    );
}

#[test]
fn rendered_files_are_listed_and_cached_by_sha256_without_a_temporary() {
    let files = fixtures::thin_with(fixtures::TEMPLATED);
    let scenario = fixtures::Scenario { files: &files, ..fixtures::thin() };
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let rendered = run(&mut store);
    assert_eq!(rendered.code, Some(0));
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  /etc/hostname", "  /etc/issue"]),
        "  /etc/hostname            file 0644 root:root 21c50adeb849\n  /etc/issue               file 0644 root:root e75f92b4b841\n"
    );
    for &(name, bytes) in fixtures::RENDERED_FILES {
        assert_eq!(cache_bytes(&rendered, name).as_deref(), Some(bytes), "{name}");
        assert_eq!(cache_mode(&rendered, name), 0o644, "{name}");
    }
    let temporaries: Vec<_> =
        store.names(CACHE).into_iter().filter(|name| name.as_bytes().ends_with(b".tmp")).collect();
    assert_eq!(temporaries, Vec::<String>::new());

    let bytes = cache_bytes(&rendered, "21c50adeb849aa6df1c779b5464f84f614f2b3775ef6d2ffb2ad0f0a1d650011.record")
        .expect("rendered cache record");
    let record = cache_file::read(&bytes, &limits::SHIPPED).expect("kind 5");
    assert_eq!(
        record.origin,
        cache_file::Origin::Rendered { path: b"tasks/motd/tree/etc/hostname.j2".as_slice().into() }
    );
    assert_eq!(record.size, 8);
    assert!(record.took > skein_lib::Duration::ZERO);
}
