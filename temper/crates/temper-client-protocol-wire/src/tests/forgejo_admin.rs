use super::fixtures;
use crate::{runner_label, v1};
use skein_lib::{Reader, Writer};

// client/engine.md: runner labels have at most 64 bytes.
const LABEL_BYTES: usize = 64;
// fixtures::BOUNDS.field_bytes, shared by SetUp and FirstAsk.
const FIELD_BYTES: usize = 16;
fn first(label: &[u8]) -> Result<v1::FirstAsk, v1::Problem> {
    v1::FirstAsk::new(
        &fixtures::limits(),
        v1::FirstAskParts {
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            label: label.into(),
        },
    )
}
fn request(first: Option<v1::FirstAsk>) -> v1::Requesting {
    let limits = fixtures::limits();
    let admin = v1::ForgejoAdmin::new(&limits, v1::ForgejoAdminParts { first }).unwrap();
    let connect =
        v1::Connect::new(&limits, v1::ConnectParts { forge: super::kept(1), how: v1::How::ForgejoAdmin(admin) })
            .unwrap();
    v1::Requesting::new(&limits, v1::RequestingParts { request: v1::Request::Connect(connect) }).unwrap()
}
// An untrusted peer's Connect: version 16, Connect tag 2, Kept tag 0, forge 1,
// ForgejoAdmin tag 2, Some(first), work/hello.
fn peer(label: &[u8]) -> Box<[u8]> {
    const HEAD: &[u8] =
        &[0, 16, 2, 0, 0, 1, 2, 1, 0, 0, 0, 4, b'w', b'o', b'r', b'k', 0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o'];
    let mut writer = Writer::new(HEAD.len().checked_add(4).unwrap().checked_add(label.len()).unwrap());
    writer.put(HEAD).unwrap();
    writer.put(&u32::try_from(label.len()).unwrap().to_be_bytes()).unwrap();
    writer.put(label).unwrap();
    writer.finish()
}
#[test]
fn forgejo_admin_without_first_round_trips_its_golden() {
    golden(None, "connect_forgejo_admin_without_first.bin");
}
#[test]
fn forgejo_admin_with_first_round_trips_its_golden() {
    golden(Some(first(b"docker").unwrap()), "connect_forgejo_admin_with_first.bin");
}
fn golden(first: Option<v1::FirstAsk>, name: &str) {
    let record = request(first);
    let path = skein_scratch::paths::package().join("golden").join(name);
    let encoded = crate::requesting(&record);
    let write = temper_test_env::rewrite();
    if write {
        std::fs::write(&path, &encoded).unwrap();
    }
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(encoded.as_ref(), bytes);
    assert_eq!(crate::decode_requesting(&bytes, &fixtures::limits()), Some(record));
}
#[test]
fn runner_labels_are_accepted_on_both_sides() {
    for label in [b"docker".as_slice(), b"temper-host", b"a.b_c", &[b'a'; LABEL_BYTES]] {
        assert!(runner_label(label), "{label:?}");
        let record = request(Some(first(label).unwrap()));
        assert_eq!(crate::decode_requesting(&peer(label), &fixtures::limits()), Some(record));
    }
}
#[test]
fn unsafe_runner_labels_are_refused_on_both_sides() {
    for label in [b"".as_slice(), b" ", b"\n", b":", b"#", &[b'a'; LABEL_BYTES + 1]] {
        assert!(!runner_label(label), "{label:?}");
        assert_eq!(crate::decode_requesting(&peer(label), &fixtures::limits()), None, "{label:?}");
    }
}
#[test]
fn first_ask_refuses_labels_past_the_schema_bound() {
    assert_eq!(
        first(&[b'a'; LABEL_BYTES + 1]).unwrap_err(),
        v1::Problem { path: v1::Path::FirstAskLabel, reason: skein_codec::Reason::Bound }
    );
}
#[test]
fn first_ask_names_past_the_setup_wire_bounds_are_refused() {
    let limits = fixtures::limits();
    let parts = first(b"docker").unwrap().into_parts();
    for (org, repository, path) in [
        (&[b'a'; FIELD_BYTES + 1][..], b"hello".as_slice(), v1::Path::FirstAskOrg),
        (b"work".as_slice(), &[b'a'; FIELD_BYTES + 1][..], v1::Path::FirstAskRepository),
    ] {
        assert_eq!(
            v1::FirstAsk::new(
                &limits,
                v1::FirstAskParts { org: org.into(), repository: repository.into(), ..parts.clone() }
            )
            .unwrap_err(),
            v1::Problem { path, reason: skein_codec::Reason::Bound }
        );
    }
}
#[test]
fn first_ask_names_at_the_setup_wire_bounds_round_trip() {
    let limits = fixtures::limits();
    let parts = first(b"docker").unwrap().into_parts();
    let record = v1::FirstAsk::new(
        &limits,
        v1::FirstAskParts { org: Box::new([b'a'; FIELD_BYTES]), repository: Box::new([b'a'; FIELD_BYTES]), ..parts },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    assert_eq!(v1::FirstAsk::decode(&limits, &mut Reader::new(&writer.finish())), Ok(record));
}
