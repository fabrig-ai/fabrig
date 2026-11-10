//! Public calls and freshness (jig/docs/design/protocol/client/default-http.md, 5).
use super::{fixtures::mapping, *};
use skein_lib::List;
fn path(ids: &[u64]) -> PublicPath {
    let mut list = List::with_capacity(2);
    for id in ids {
        list.push(*id).unwrap();
    }
    PublicPath { owner: 0, version: 1, kind: 3, ids: list }
}
#[test]
fn the_public_call_is_a_get_with_no_bearer_or_body() {
    let call = public(&mapping(), &path(&[1, 2]));
    assert_eq!(call.method, Method::Get);
    assert!(call.body.is_empty());
    assert!(call.headers.is_empty());
    assert_eq!(call.path.as_ref(), b"/jig/v1/public/0/1/3/0000000000000001/0000000000000002");
    assert_eq!(PUBLIC_KEEP, &["cache-control", "age", "retry-after"]);
}
#[test]
fn the_printed_target_is_the_canonical_path() {
    for (path, expected) in [
        (path(&[]), b"/v1/public/0/1/3".as_slice()),
        (path(&[0x0123_4567_89ab_cdef]), b"/v1/public/0/1/3/0123456789abcdef"),
        (
            PublicPath { owner: 65535, version: 65535, kind: 65535, ..path(&[0, u64::MAX]) },
            b"/v1/public/65535/65535/65535/0000000000000000/ffffffffffffffff",
        ),
    ] {
        assert_eq!(public_target(&path).as_ref(), expected);
    }
}
#[test]
#[expect(clippy::disallowed_types, reason = "the public HTTP boundary takes String header pairs")]
fn freshness_reads_present_absent_and_malformed_headers() {
    for (fields, fresh, retry_after) in [
        (&[("cache-control", "max-age=5"), ("age", "2")][..], 3, None),
        (&[("cache-control", "max-age=5"), ("age", "9")], 0, None),
        (&[], 0, None),
        (&[("cache-control", "no-store, max-age=5")], 0, None),
        (&[("cache-control", "max-age=x")], 0, None),
        (&[("cache-control", "max-age=5"), ("age", "-1")], 0, None),
        (&[("retry-after", "1")], 0, Some(1)),
        (&[("retry-after", "soon")], 0, None),
        (&[("cache-control", "public, max-age=5, must-revalidate")], 5, None),
        (&[("cache-control", "MAX-AGE = 5"), ("age", "0")], 5, None),
        (&[("cache-control", "max-age=5, max-age=9")], 0, None),
        (&[("cache-control", "max-age=5"), ("age", "0"), ("age", "1")], 0, None),
        (&[("cache-control", "max-age=4294967296")], 0, None),
        (&[("cache-control", "max-age=5"), ("age", "x")], 0, None),
        (&[("cache-control", "max-age=+5")], 0, None),
        (&[("cache-control", "max-age=5"), ("age", "")], 0, None),
        (&[("retry-after", "-1")], 0, None),
        (&[("cache-control", "max-age=5"), ("cache-control", "no-store")], 0, None),
    ] {
        let mut headers = List::with_capacity(u32::try_from(fields.len()).unwrap());
        for (name, value) in fields {
            headers.push((String::from(*name), String::from(*value))).unwrap();
        }
        assert_eq!(freshness(headers.as_slice()), Freshness { fresh, retry_after }, "{fields:?}");
    }
}
#[test]
#[expect(clippy::disallowed_types, reason = "the public HTTP boundary takes String header pairs")]
fn statuses_map_to_their_answers() {
    let headers =
        [(String::from("cache-control"), String::from("max-age=5")), (String::from("retry-after"), String::from("1"))];
    for (status, expected) in [
        (200, PublicHeard::Document { body: Box::from(b"doc".as_slice()), fresh: 5 }),
        (404, PublicHeard::Unknown { fresh: 5 }),
        (503, PublicHeard::Busy { retry_after: Some(1) }),
        (201, PublicHeard::Lost),
        (421, PublicHeard::Lost),
        (500, PublicHeard::Lost),
    ] {
        assert_eq!(public_answered(status, Box::from(b"doc".as_slice()), &headers), expected);
    }
}
