use super::records::{STORE_RECORD, WALL, run};
use hearth_builder_world::{fixtures, protocol};
use hearth_fake_machine::linux::Item;
use hearth_protocol_tree::{limits, store as record};
use skein_lib::{Time, Wall};

fn store_record(uid: u32) -> Box<[u8]> {
    record::write(&record::Store { layout: 1, made: Wall::EPOCH, range: None, uid }, &limits::SHIPPED)
        .expect("store record")
}

#[test]
fn an_unreadable_store_record_is_written_again() {
    let scenario = fixtures::thin();
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let mut bytes = store_record(fixtures::BUILDER.user);
    *bytes.get_mut(11).expect("version byte") = 2;
    store.lay([Item::file(STORE_RECORD, 0o600, &bytes)]);
    let run = run(&mut store);
    let bytes = run.shared.contents(STORE_RECORD).expect("rewritten record");
    let record = record::read(&bytes, &limits::SHIPPED).expect("version 1 again");
    assert_eq!(record.layout, 1);
    assert_eq!(record.range, None);
}

#[test]
fn a_store_recorded_by_another_uid_is_refused_on_standard_error() {
    let scenario = fixtures::thin();
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let bytes = store_record(fixtures::BUILDER.user + 1);
    store.lay([Item::file(STORE_RECORD, 0o600, &bytes)]);
    let run = run(&mut store);
    assert_eq!(run.stdout, b"");
    assert_eq!(run.stderr, b"hearth: cannot open the store /srv/store: store.record is uid 1002's, not uid 1001's\n");
    assert_eq!(run.code, Some(75));
}

#[test]
fn a_lock_released_before_the_notice_says_nothing() {
    let release = Time::from_nanos(20_000_000);
    let run = protocol::with_lock(&fixtures::thin(), fixtures::SIMULATION_SEED, release, &fixtures::notice_limits());
    assert!(run.lock_times.len() > 1, "the first lock ask met a contender");
    assert_eq!(run.stderr, b"");
    assert_eq!(run.stdout, fixtures::THIN_TEXT.as_bytes());
}

#[test]
fn a_stray_in_the_cache_is_reported_once_on_standard_error() {
    let scenario = fixtures::Scenario { laid: &[fixtures::Laid::File("cache/notes")], ..fixtures::thin() };
    let mut store = protocol::Store::new(&scenario, fixtures::SIMULATION_SEED);
    let run = store.run(WALL, &hearth::limits());
    assert_eq!(run.stderr, b"strays in the store /srv/store: 1, the first cache/notes\n");
    assert_eq!(run.stdout, fixtures::THIN_TEXT.as_bytes());
}
