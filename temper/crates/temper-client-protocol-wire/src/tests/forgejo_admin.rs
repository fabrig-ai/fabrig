use super::fixtures;
use crate::{runner_label, v1};
use skein_lib::{Reader, Writer};

// client/README.md, 8: runner labels have at most 64 bytes.
const LABEL_BYTES: usize = 64;
// fixtures::BOUNDS.field_bytes, shared by SetUp and FirstAsk.
const FIELD_BYTES: usize = 16;
fn first(label: &[u8]) -> Result<v1::FirstAsk, v1::Problem> {
    v1::FirstAsk::new(
        &fixtures::limits(),
        v1::FirstAskParts {
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            project: v1::SetUpProject::Existing(
                v1::ExistingProject::new(&v1::CEILINGS, v1::ExistingProjectParts { project: 7 }).unwrap(),
            ),
            label: label.into(),
        },
    )
}
fn request(first: Option<v1::FirstAsk>) -> v1::Requesting {
    let limits = fixtures::limits();
    let admin = v1::ForgejoAdmin::new(&limits, v1::ForgejoAdminParts { first }).unwrap();
    let connect = v1::Connect::new(
        &limits,
        v1::ConnectParts {
            forge: v1::ConnectForge::Existing(
                v1::ExistingForge::new(&v1::CEILINGS, v1::ExistingForgeParts { forge: 1 }).expect("existing forge"),
            ),
            how: v1::How::ForgejoAdmin(admin),
        },
    )
    .unwrap();
    v1::Requesting::new(&limits, v1::RequestingParts { request: v1::Request::Connect(connect) }).unwrap()
}
// An untrusted peer's Connect: version 11, Connect tag 3, forge 1,
// ForgejoAdmin tag 2, Some(first), work/hello, Existing project 7.
fn peer(label: &[u8]) -> Box<[u8]> {
    const HEAD: &[u8] = &[
        0, 11, 3, 0, 0, 1, 2, 1, 0, 0, 0, 4, b'w', b'o', b'r', b'k', 0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o', 0, 0, 0,
        0, 7,
    ];
    let mut writer = Writer::new(HEAD.len().checked_add(4).unwrap().checked_add(label.len()).unwrap());
    writer.put(HEAD).unwrap();
    writer.put(&u32::try_from(label.len()).unwrap().to_be_bytes()).unwrap();
    writer.put(label).unwrap();
    writer.finish()
}
#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire golden is test input")]
fn forgejo_admin_without_first_round_trips_its_golden() {
    let record = request(None);
    assert_eq!(crate::decode_requesting(&crate::requesting(&record), &fixtures::limits()), Some(record));
    how_golden(include_bytes!("../../golden/enum_how_forgejo_admin_smallest.bin"));
}
#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire golden is test input")]
fn forgejo_admin_with_first_round_trips_its_golden() {
    let record = request(Some(first(b"docker").unwrap()));
    assert_eq!(crate::decode_requesting(&crate::requesting(&record), &fixtures::limits()), Some(record));
    how_golden(include_bytes!("../../golden/enum_how_forgejo_admin_full.bin"));
}
fn how_golden(bytes: &[u8]) {
    let how = v1::How::decode(&v1::CEILINGS, &mut Reader::new(bytes)).unwrap();
    let mut writer = Writer::new(usize::try_from(how.measure()).unwrap());
    how.encode(&mut writer).unwrap();
    assert_eq!(writer.finish().as_ref(), bytes);
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
