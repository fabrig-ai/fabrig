use hearth_builder_world::fixtures::{self, Release, Repository};
use hearth_domain_store as store;
use hearth_protocol_tree::{limits, store as record};
use skein_lib::Duration;
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
};
#[test]
fn a_render_over_a_locked_store_waits_then_renders_once_it_is_released() {
    let repository = Repository::make("locked", fixtures::THIN);
    // Five asks fit the test: shipped retry is 200 ms.
    let limits =
        fixtures::with_store(store::limits::Limits { retry: Duration::from_millis(20), ..store::limits::SHIPPED });
    let held = fixtures::render_holding(&repository, &limits, Release::After(Duration::from_millis(100)));
    assert!(held.printed_while_held.is_empty(), "nothing renders while the lock is busy");
    assert_eq!(String::from_utf8_lossy(&held.rendered.stdout), fixtures::THIN_TEXT);
}
#[test]
fn the_store_records_its_real_owner_and_uses_private_directories() {
    let repository = Repository::make("first-record", fixtures::THIN);
    drop(fixtures::render_with(&repository, &hearth::limits()));
    let store = repository.scratch().path().join("store");
    let record =
        record::read(&fs::read(store.join("store.record")).expect("store record"), &limits::SHIPPED).expect("kind 1");
    assert_eq!(record.uid, fs::metadata(&store).expect("store owner").uid());
    for name in ["cache", "resolutions", "staging"] {
        assert_eq!(fs::metadata(store.join(name)).expect("directory").permissions().mode() & 0o7777, 0o700, "{name}");
    }
}

#[test]
fn staging_keeps_its_private_mode_after_the_scan() {
    let repository = Repository::make("scan-staging", fixtures::THIN);
    drop(fixtures::render_with(&repository, &hearth::limits()));
    let staging = repository.scratch().path().join("store/staging");
    fs::write(staging.join("temporary"), b"temporary").expect("staging file");
    drop(fixtures::render_with(&repository, &hearth::limits()));
    assert_eq!(fs::metadata(staging).expect("kept staging").permissions().mode() & 0o7777, 0o700);
}
