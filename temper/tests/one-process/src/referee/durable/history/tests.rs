use super::*;
use crate::{
    fixtures,
    referee::durable::{Observed, Store},
};
use fixtures::durable as fixture;

fn frame(body: &[u8], compressed: bool) -> Vec<u8> {
    let mut encoded = vec![0; skein_deflate::bound(body.len()).expect("compression bound")];
    let size = skein_deflate::encode(body, &mut encoded).expect("compression fits");
    let payload = if compressed { &encoded[..size] } else { body };
    let length = u32::try_from(payload.len() + 17).expect("frame length");
    let mut bytes = length.to_be_bytes().to_vec();
    bytes.push(u8::from(compressed));
    bytes.extend_from_slice(&u32::try_from(body.len()).expect("record length").to_be_bytes());
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&transcripts::frame::crc32c(&bytes).to_be_bytes());
    bytes.extend_from_slice(&length.to_be_bytes());
    bytes
}
fn extent(length: usize) -> skein_kv::Op {
    let mut key = vec![0, 9];
    key.extend_from_slice(&2_u64.to_be_bytes());
    skein_kv::Op::Put { key: key.into(), value: u64::try_from(length).expect("log length").to_be_bytes().into() }
}
#[test]
fn history_is_observed_at_its_committed_extent_not_at_its_payload_sync() {
    for compressed in [false, true] {
        let mut store = Store::new();
        let root::Record::Core(row) = fixture::row() else { unreachable!() };
        let key = keys::encode(&row.key(), 1024).expect("terminal key");
        let value = jig_protocol_store::codec::encode(&row, &fixtures::codec(), fixtures::store_names())
            .expect("terminal bytes");
        let first = frame(
            &jig_protocol_store::logs::record::encode(&[(key.clone(), Some(value))]).expect("put record"),
            compressed,
        );
        let mut payload = first.clone();
        payload.extend(frame(
            &jig_protocol_store::logs::record::encode(&[(key, None)]).expect("erase record"),
            compressed,
        ));
        let mut events = vec![
            Observed::Bytes {
                path: b"generations/1/store/logs/0000000000000000/0000000000000002".to_vec(),
                at: 0,
                bytes: payload.clone(),
            },
            Observed::Sync(b"generations/1/store/logs/0000000000000000/0000000000000002".to_vec()),
        ];
        assert!(
            store
                .observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds())
                .expect("payload sync")
                .is_empty()
        );
        assert!(store.rows.is_empty(), "payload alone is not a durable commit");
        let first_commit = fixture::bytes_with_ops(false, Box::new([extent(first.len())])).frames.remove(0);
        let mut events = vec![
            Observed::Bytes { path: fixture::LOG.to_vec(), at: 0, bytes: first_commit.clone() },
            Observed::Sync(fixture::LOG.to_vec()),
        ];
        assert_eq!(
            store
                .observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds())
                .expect("first extent")
                .len(),
            1
        );
        assert_eq!(
            store.rows.get(&fixture::row().key()),
            Some(&fixture::row()),
            "uncommitted erase tail stays invisible"
        );
        let second_commit = fixture::bytes_with_ops(false, Box::new([extent(payload.len())])).frames.remove(1);
        let mut events = vec![
            Observed::Bytes { path: fixture::LOG.to_vec(), at: first_commit.len(), bytes: second_commit },
            Observed::Sync(fixture::LOG.to_vec()),
        ];
        assert_eq!(
            store
                .observe(&mut events, &fixtures::codec(), fixtures::store_names(), &fixture::bounds())
                .expect("second extent")
                .len(),
            1
        );
        assert!(store.rows.is_empty(), "the committed erase hides the terminal");
    }
}
