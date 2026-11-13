//! A run's bounded read and its new roots (domain/forge/reds.md, 5.4).
use super::fixtures;
use crate::v1;
use skein_lib::{List, Reader, Writer};

#[test]
fn reds_bound_the_count_and_key_when_built_and_decoded() {
    let limits = fixtures::limits();
    assert_eq!((v1::CEILINGS.reds_reds, v1::CEILINGS.red_key, v1::CEILINGS.red_head), (256, 512, 32));
    for count in [2, 3] {
        let mut reds = List::with_capacity(count);
        for _ in 0..count {
            reds.push(
                v1::Red::new(
                    &v1::CEILINGS,
                    v1::RedParts { key: b"lint p c f".as_slice().into(), head: Box::new([1; 20]) },
                )
                .unwrap(),
            )
            .unwrap();
        }
        let value = v1::Reds::new(&v1::CEILINGS, v1::RedsParts { reds }).unwrap();
        let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
        value.encode(&mut writer).unwrap();
        let bytes = writer.finish();
        assert_eq!(v1::Reds::new(&limits, value.into_parts()).is_ok(), count == 2);
        assert_eq!(v1::Reds::decode(&limits, &mut Reader::new(&bytes)).is_ok(), count == 2);
    }
    for length in [12, 13] {
        let value = v1::Red::new(
            &v1::CEILINGS,
            v1::RedParts { key: skein_lib::bytes::zeroed(length), head: Box::new([2; 32]) },
        )
        .unwrap();
        let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
        value.encode(&mut writer).unwrap();
        let bytes = writer.finish();
        assert_eq!(v1::Red::new(&limits, value.into_parts()).is_ok(), length == 12);
        assert_eq!(v1::Red::decode(&limits, &mut Reader::new(&bytes)).is_ok(), length == 12);
    }
}

#[test]
fn the_new_ask_and_answer_roots_cross_only_the_new_version() {
    let application = crate::application(&fixtures::limits()).unwrap();
    let bridge = application.bridges.get(0).unwrap();
    assert_eq!(crate::wire::VERSION, 8);
    for root in [b"Ask".as_slice(), b"Answer"] {
        assert_eq!(bridge.total(8, root), Some(true));
        assert_eq!(bridge.total(7, root), None, "the previous version has no counterpart");
    }
    assert_eq!(crate::ask(&v1::Ask::Reds).as_ref(), &[0]);
    let answer =
        v1::Answer::Reds(v1::Reds::new(&fixtures::limits(), v1::RedsParts { reds: List::with_capacity(0) }).unwrap());
    let bytes = crate::answer(&answer);
    assert_eq!(bytes.as_ref(), &[0, 0, 0, 0, 0]);
    assert_eq!(v1::Answer::decode(&fixtures::limits(), &mut Reader::new(&bytes)).unwrap(), answer);
}
