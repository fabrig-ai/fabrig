//! The person's act's bounds, beside the generated codecs' round trips
//! (temper/docs/design/client/forge/records.md, `OpenChange`).
use crate::v1 as w;
use crate::v1::Record;

fn repeated(byte: u8, length: u32) -> Box<[u8]> {
    let mut bytes = skein_lib::List::with_capacity(length);
    for _ in 0..length {
        bytes.push(byte).unwrap();
    }
    bytes.into_boxed()
}

fn parts() -> w::OpenChange {
    w::OpenChange {
        project: 1,
        repository: None,
        branch: b"docs/x".as_slice().into(),
        at: None,
        into: None,
        title: b"Fix the docs".as_slice().into(),
        body: None,
        budget: w::OpenBudget::AsAllowed,
    }
}
fn decoded(parts: w::OpenChange) -> Option<w::Requesting> {
    let record = w::Requesting { request: w::Request::OpenChange(parts) };
    crate::decode_requesting(&crate::requesting(&record), &w::CEILINGS)
}
#[test]
fn opening_fields_keep_their_minima_and_commit_text_is_exact() {
    assert!(decoded(parts()).is_some());
    for width in [40, 64] {
        let mut parts = parts();
        parts.at = Some(repeated(b'f', width));
        assert!(decoded(parts).is_some());
    }
    for case in 0_u8..8 {
        let mut parts = parts();
        match case {
            0 => parts.branch = Box::new([]),
            1 => parts.title = Box::new([]),
            2 => parts.body = Some(Box::new([])),
            3 => parts.into = Some(Box::new([])),
            4 => parts.at = Some(repeated(b'a', 39)),
            5 => parts.at = Some(repeated(b'a', 41)),
            6 => parts.at = Some(repeated(b'A', 40)),
            7 => parts.at = Some(repeated(b'g', 64)),
            _ => unreachable!(),
        }
        assert!(decoded(parts).is_none(), "case {case}");
    }
    for case in 0_u8..5 {
        let mut parts = parts();
        match case {
            0 => parts.branch = repeated(b'x', 1025),
            1 => parts.into = Some(repeated(b'x', 1025)),
            2 => parts.title = repeated(b'x', 257),
            3 => parts.body = Some(repeated(b'x', 16_385)),
            4 => parts.at = Some(repeated(b'a', 65)),
            _ => unreachable!(),
        }
        assert!((parts).encode(&w::CEILINGS).is_err(), "case {case}");
    }
}
