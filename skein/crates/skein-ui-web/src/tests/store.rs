use super::fixtures::{Call, FakePlatform};
use crate::store::{Record, Refused, Store};

#[test]
fn records_are_base64_under_the_prefix() {
    let mut platform = FakePlatform::default();
    Store::new(b"notes-", None)
        .write(b"key", b"record", &mut platform)
        .expect("fixture write has no quota or size bound");
    assert_eq!(platform.calls, vec![Call::Write(b"notes-a2V5".to_vec(), b"cmVjb3Jk".to_vec())]);
}
#[test]
fn a_write_past_the_bound_is_refused() {
    let mut platform = FakePlatform::default();
    let result = Store::new(b"notes-", Some(17)).write(b"key", b"record", &mut platform);
    assert_eq!(result, Err(Refused));
}
#[test]
fn records_of_another_prefix_are_not_read() {
    let platform = FakePlatform {
        records: vec![Record { key: b"other-a2V5".as_slice().into(), value: b"cmVjb3Jk".as_slice().into() }],
        ..FakePlatform::default()
    };
    assert_eq!(Store::new(b"notes-", None).read_all(&platform), vec![]);
}
#[test]
fn records_are_decoded_at_start() {
    let platform = FakePlatform {
        records: vec![Record { key: b"notes-a2V5".as_slice().into(), value: b"cmVjb3Jk".as_slice().into() }],
        ..FakePlatform::default()
    };
    assert_eq!(
        Store::new(b"notes-", None).read_all(&platform),
        vec![Record { key: b"key".as_slice().into(), value: b"record".as_slice().into() }]
    );
}
#[test]
fn a_platform_quota_refuses_a_write() {
    let mut platform = FakePlatform { quota: Some(0), ..FakePlatform::default() };
    assert_eq!(Store::new(b"notes-", None).write(b"key", b"record", &mut platform), Err(Refused));
}
#[test]
fn removal_uses_the_encoded_key() {
    let mut platform = FakePlatform::default();
    Store::new(b"notes-", None).remove(b"key", &mut platform);
    assert_eq!(platform.calls, vec![Call::Remove(b"notes-a2V5".to_vec())]);
}

#[test]
fn the_kept_store_is_the_small_stores_form_in_local_storage() {
    // notes- plus the encoded key and value holds 18 text bytes.
    const RECORD_BYTES: u64 = 18;
    let record = |value: &[u8]| Record { key: b"notes-a2V5".as_slice().into(), value: value.into() };
    let cases = [
        (
            Store::new as fn(&[u8], Option<u64>) -> Store,
            b"session".as_slice(),
            Call::Write(b"notes-a2V5".to_vec(), b"cmVjb3Jk".to_vec()),
            Call::Remove(b"notes-a2V5".to_vec()),
        ),
        (
            Store::kept as fn(&[u8], Option<u64>) -> Store,
            b"kept".as_slice(),
            Call::WriteKept(b"notes-a2V5".to_vec(), b"cmVjb3Jk".to_vec()),
            Call::RemoveKept(b"notes-a2V5".to_vec()),
        ),
    ];
    for (make, expected, write, remove) in cases {
        let mut platform = FakePlatform {
            records: vec![record(b"c2Vzc2lvbg==")],
            kept: vec![record(b"a2VwdA==")],
            ..FakePlatform::default()
        };
        let mut store = make(b"notes-", Some(RECORD_BYTES));
        assert_eq!(store.read_all(&platform), vec![Record { key: b"key".as_slice().into(), value: expected.into() }]);
        assert_eq!(store.write(b"key", b"record", &mut platform), Ok(()));
        assert_eq!(platform.calls, vec![write]);
        platform.calls.clear();
        store.remove(b"key", &mut platform);
        assert_eq!(store.read_all(&platform), vec![]);
        assert_eq!(platform.calls, vec![remove]);
        platform.calls.clear();
        assert_eq!(make(b"notes-", Some(RECORD_BYTES - 1)).write(b"key", b"record", &mut platform), Err(Refused));
        assert_eq!(platform.calls.len(), 0);
        platform.quota = Some(0);
        assert_eq!(store.write(b"key", b"record", &mut platform), Err(Refused));
    }
    let mut platform = super::fixtures::SessionOnly;
    let mut store = Store::kept(b"notes-", None);
    assert_eq!(store.read_all(&platform), vec![]);
    assert_eq!(store.write(b"key", b"record", &mut platform), Err(Refused));
    store.remove(b"key", &mut platform);
}
