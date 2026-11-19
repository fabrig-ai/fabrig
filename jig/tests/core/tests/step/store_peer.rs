//! Isolation of immutable applied store evidence.
use crate::store_peer_support::{header, opaque_turn, writes};
use jig_core_world::{Fault, Store, store_peer::StorePeer};
use jig_domain_core as core;
use jig_test_domain::{self as root, Write};
use skein_lib::Time;

#[test]
fn whole_opaque_rows_remain_immutable_after_later_save_erase_and_store_mutation() {
    let mut store = Store::new(&jig_test_profile::limits(), &[]);
    let mut peer = StorePeer::new();
    let original = opaque_turn(&[0, 255, 128, 1, 0, 254]);
    let key = original.key();
    store.submit(1, writes(1, Write::Save(original.clone())), 0);
    assert_eq!(store.tick(false), Ok(Some(1)));
    peer.capture(Time::from_nanos(17), &store);
    let first = peer.take().pop().expect("first durable snapshot");
    assert_eq!(first.rows.get(&key), Some(&original));
    let replacement = opaque_turn(b"replacement allocation");
    store.submit(2, writes(2, Write::Save(replacement.clone())), 0);
    assert_eq!(store.tick(false), Ok(Some(2)));
    peer.capture(Time::from_nanos(18), &store);
    store.submit(3, writes(3, Write::Erase(key.clone())), 0);
    assert_eq!(store.tick(false), Ok(Some(3)));
    peer.capture(Time::from_nanos(19), &store);
    let later = peer.take();
    assert_eq!(later[0].rows.get(&key), Some(&replacement));
    assert_eq!(later[1].rows, header_rows(3));
    store.submit(4, writes(4, Write::Save(opaque_turn(b"external mutation"))), 0);
    assert_eq!(store.tick(false), Ok(Some(4)));
    assert_eq!(first.rows.get(&key), Some(&original), "all allocated bytes and scalar facts survived");
    assert_eq!(later[0].rows.get(&key), Some(&replacement));
    assert_eq!(later[1].rows, header_rows(3));
}

#[test]
fn pending_failed_and_abandoned_transactions_emit_nothing_and_crash_preserves_captured_rows() {
    let mut store = Store::new(&jig_test_profile::limits(), &[]);
    let mut peer = StorePeer::new();
    let durable = opaque_turn(b"actually applied");
    let key = durable.key();
    store.submit(1, writes(1, Write::Save(durable.clone())), 0);
    assert_eq!(store.tick(true), Ok(None));
    peer.capture(Time::from_nanos(20), &store);
    store.submit(2, writes(2, Write::Erase(key.clone())), 2);
    assert_eq!(store.tick(false), Ok(None));
    peer.capture(Time::from_nanos(21), &store);
    store.crash(18).expect("crash readback");
    peer.capture(Time::from_nanos(22), &store);
    let captured = peer.take();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].rows.get(&key), Some(&durable));
    store.fault(Fault::Fail { commit: 2 });
    store.submit(2, writes(2, Write::Erase(key.clone())), 0);
    store.submit(3, writes(3, Write::Save(opaque_turn(b"abandoned after failure"))), 0);
    assert_eq!(store.tick(false), Err(2));
    peer.capture(Time::from_nanos(23), &store);
    assert_eq!(peer.take().len(), 0);
    assert_eq!(store.applied, 1);
    assert_eq!(store.rows.get(&key), Some(&durable));
    assert!(store.pending.is_empty());
    store.crash(18).expect("crash readback");
    peer.capture(Time::from_nanos(24), &store);
    assert_eq!(peer.take().len(), 0);
    assert_eq!(captured[0].rows.get(&key), Some(&durable));
}

fn header_rows(commits: u64) -> std::collections::BTreeMap<root::Key, root::Record> {
    let header = header(commits);
    std::collections::BTreeMap::from([(root::Key::Core(core::Key::Core(core::CoreKey::Deployment)), header)])
}
