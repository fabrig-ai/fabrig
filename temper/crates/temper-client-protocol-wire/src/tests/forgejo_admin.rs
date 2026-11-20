use super::fixtures;
use crate::v1::Record;
use crate::{runner_label, v1};
use skein_lib::{Reader, Writer};

// client/engine.md: runner labels have at most 64 bytes.
const LABEL_BYTES: usize = 64;
// fixtures::BOUNDS.field_bytes, shared by SetUp and FirstAsk.
const FIELD_BYTES: usize = 16;
fn first(label: &[u8]) -> Result<v1::FirstAsk, v1::Problem> {
    {
        let value = v1::FirstAsk {
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            label: label.into(),
        };
        match value.encode(&fixtures::limits()) {
            Ok(_) => Ok(value),
            Err(problem) => Err(problem),
        }
    }
}
fn request(first: Option<v1::FirstAsk>) -> v1::Requesting {
    let admin = v1::ForgejoAdmin { first };
    let connect = v1::Connect { forge: super::kept(1), how: v1::How::ForgejoAdmin(admin) };
    v1::Requesting { request: v1::Request::Connect(connect) }
}
// An untrusted peer's Connect at the current version: Connect tag 2, Kept tag 0, forge 1,
// ForgejoAdmin tag 2, Some(first), work/hello.
fn peer(label: &[u8]) -> Box<[u8]> {
    const HEAD: &[u8] =
        &[2, 0, 0, 1, 2, 1, 0, 0, 0, 4, b'w', b'o', b'r', b'k', 0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o'];
    let mut writer = Writer::new(HEAD.len().checked_add(6).unwrap().checked_add(label.len()).unwrap());
    writer.put(&u16::try_from(crate::CLIENT_VERSION).unwrap().to_be_bytes()).unwrap();
    writer.put(HEAD).unwrap();
    writer.put(&u32::try_from(label.len()).unwrap().to_be_bytes()).unwrap();
    writer.put(label).unwrap();
    writer.finish()
}
#[test]
fn forgejo_admin_without_first_round_trips() {
    round_trip(None);
}
#[test]
fn forgejo_admin_with_first_round_trips() {
    round_trip(Some(first(b"docker").unwrap()));
}
fn round_trip(first: Option<v1::FirstAsk>) {
    let record = request(first);
    let bytes = crate::requesting(&record);
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
        v1::Problem { path: v1::paths::FIRST_ASK_LABEL, reason: skein_codec::Reason::Bound }
    );
}
#[test]
fn first_ask_names_past_the_setup_wire_bounds_are_refused() {
    let limits = fixtures::limits();
    let parts = first(b"docker").unwrap();
    for (org, repository, path) in [
        (&[b'a'; FIELD_BYTES + 1][..], b"hello".as_slice(), v1::paths::FIRST_ASK_ORG),
        (b"work".as_slice(), &[b'a'; FIELD_BYTES + 1][..], v1::paths::FIRST_ASK_REPOSITORY),
    ] {
        assert_eq!(
            (v1::FirstAsk { org: org.into(), repository: repository.into(), ..parts.clone() })
                .encode(&limits)
                .unwrap_err(),
            v1::Problem { path, reason: skein_codec::Reason::Bound }
        );
    }
}
#[test]
fn first_ask_names_at_the_setup_wire_bounds_round_trip() {
    let limits = fixtures::limits();
    let parts = first(b"docker").unwrap();
    let record =
        v1::FirstAsk { org: Box::new([b'a'; FIELD_BYTES]), repository: Box::new([b'a'; FIELD_BYTES]), ..parts };
    let mut writer = Writer::new(usize::try_from(record.measure(&v1::CEILINGS).expect("within ceilings")).unwrap());
    record.write(&v1::CEILINGS, &mut writer).unwrap();
    assert_eq!(v1::FirstAsk::decode(&limits, &mut Reader::new(&writer.finish())), Ok(record));
}
