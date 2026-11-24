use super::records::{CACHE, STORE_NAMES, THIN_CACHE_NAMES, run};
use hearth_builder_world::{fixtures, protocol};
use hearth_fake_machine::linux::Item;
use hearth_lib::encode::Digest;
use hearth_protocol_tree::{limits, resolution};

const CRASH: &[fixtures::Laid] = &[
    fixtures::Laid::File("cache/0101010101010101010101010101010101010101010101010101010101010101"),
    fixtures::Laid::Record(
        "cache/0101010101010101010101010101010101010101010101010101010101010101.record",
        fixtures::D1,
    ),
    fixtures::Laid::File("cache/0202020202020202020202020202020202020202020202020202020202020202"),
    fixtures::Laid::Record(
        "cache/0303030303030303030303030303030303030303030303030303030303030303.record",
        fixtures::D3,
    ),
    fixtures::Laid::File("cache/0101010101010101010101010101010101010101010101010101010101010101.tmp"),
    fixtures::Laid::File(
        "cache/0101010101010101010101010101010101010101010101010101010101010101.record.skein-0123456789abcdef",
    ),
    fixtures::Laid::File("cache/notes"),
];

#[test]
fn the_scan_settles_what_a_crash_left_in_the_cache() {
    let scenario = fixtures::Scenario { laid: CRASH, ..fixtures::thin() };
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let run = run(&mut store);
    assert_eq!(run.code, Some(0));
    let mut expected = vec![
        "0101010101010101010101010101010101010101010101010101010101010101",
        "0101010101010101010101010101010101010101010101010101010101010101.record",
    ];
    expected.extend_from_slice(THIN_CACHE_NAMES);
    expected.push("notes");
    assert_eq!(store.names(CACHE), expected);
}

#[test]
fn the_scan_removes_the_store_records_temporary() {
    let scenario =
        fixtures::Scenario { laid: &[fixtures::Laid::File("store.record.skein-0123456789abcdef")], ..fixtures::thin() };
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    drop(run(&mut store));
    assert_eq!(store.names(fixtures::STORE), STORE_NAMES);
}

const RESOLUTIONS: &[(&str, Digest, &[u8], Digest, u8)] = &[
    (
        "01ea811afc99308a9fcc336c944966c7f9dd98857b27592fe00a1fb84bd49d00.record",
        fixtures::RESOLUTION_KEY,
        b"apt",
        fixtures::CLOSURE,
        1,
    ),
    (
        "0202020202020202020202020202020202020202020202020202020202020202.record",
        Digest([2; 32]),
        b"minbase",
        fixtures::CLOSURE,
        1,
    ),
    (
        "0303030303030303030303030303030303030303030303030303030303030303.record",
        fixtures::RESOLUTION_KEY,
        b"apt",
        fixtures::CLOSURE,
        1,
    ),
    (
        "a33fbba1e283ca942abc7b2376b18e4871d28e766ee1a0f1e90470d7b82e21bb.record",
        fixtures::MINBASE_KEY,
        b"minbase",
        Digest([5; 32]),
        1,
    ),
    (
        "0404040404040404040404040404040404040404040404040404040404040404.record",
        fixtures::RESOLUTION_KEY,
        b"apt",
        fixtures::CLOSURE,
        2,
    ),
];

#[test]
fn the_scan_removes_resolutions_whose_keys_or_digests_do_not_recompute() {
    let scenario = fixtures::thin();
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    store.lay(RESOLUTIONS.iter().map(|&(name, key, variant, digest, version)| {
        let mut bytes =
            resolution::write(&fixtures::resolution(key, variant, digest), &limits::SHIPPED).expect("resolution");
        *bytes.get_mut(11).expect("frame version") = version;
        Item::file(format!("/srv/store/resolutions/{name}").as_bytes(), 0o600, &bytes)
    }));
    store.lay([
        Item::file(
            b"/srv/store/resolutions/01ea811afc99308a9fcc336c944966c7f9dd98857b27592fe00a1fb84bd49d00.record.skein-0123456789abcdef",
            0o600,
            b"temporary",
        ),
        Item::file(b"/srv/store/resolutions/junk", 0o600, b"stray"),
    ]);
    drop(run(&mut store));
    assert_eq!(
        store.names(b"/srv/store/resolutions"),
        [
            "01ea811afc99308a9fcc336c944966c7f9dd98857b27592fe00a1fb84bd49d00.record",
            "0404040404040404040404040404040404040404040404040404040404040404.record",
            "junk",
        ]
    );
}

#[test]
fn the_scan_empties_staging_and_keeps_its_links_targets() {
    let scenario = fixtures::thin();
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let mut path = "/srv/store/staging".to_owned();
    let mut tree = Vec::new();
    for name in
        ["0101010101010101010101010101010101010101010101010101010101010101", "root", "var", "lib", "apt", "lists"]
    {
        path.push('/');
        path.push_str(name);
        tree.push(Item::directory(path.as_bytes(), 0o700));
    }
    tree.push(Item::directory(format!("{path}/partial").as_bytes(), 0o700));
    tree.push(Item::file(format!("{path}/x").as_bytes(), 0o600, b"temporary"));
    tree.push(Item::file(b"/srv/kept", 0o600, b"outside"));
    tree.push(Item::link(b"/srv/store/staging/link", b"/srv/kept"));
    store.lay(tree);
    let run = run(&mut store);
    assert_eq!(store.names(b"/srv/store/staging"), Vec::<String>::new());
    assert_eq!(run.shared.contents(b"/srv/kept").as_deref(), Some(b"outside".as_slice()));
}
