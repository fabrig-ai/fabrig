//! A run's bounded read and its new roots (domain/forge/reds.md, 5.4).
use super::fixtures;
use crate::v1::{self, Record};
use skein_lib::{List, Reader};

#[test]
fn reds_bound_the_count_and_key_when_encoded_and_decoded() {
    let limits = fixtures::limits();
    assert_eq!((v1::CEILINGS.reds_reds, v1::CEILINGS.red_key, v1::CEILINGS.red_head), (256, 512, 32));
    for count in [2, 3] {
        let mut reds = List::with_capacity(count);
        for _ in 0..count {
            reds.push(v1::Red { key: b"lint p c f".as_slice().into(), head: Box::new([1; 20]) }).unwrap();
        }
        let value = v1::Reds { reds };
        let bytes = value.encode(&v1::CEILINGS).expect("within bounds");
        assert_eq!(value.encode(&limits).is_ok(), count == 2);
        assert_eq!(v1::Reds::decode(&limits, &mut Reader::new(&bytes)).is_ok(), count == 2);
    }
    for length in [12, 13] {
        let value = v1::Red { key: skein_lib::bytes::zeroed(length), head: Box::new([2; 32]) };
        let bytes = value.encode(&v1::CEILINGS).expect("within bounds");
        assert_eq!(value.encode(&limits).is_ok(), length == 12);
        assert_eq!(v1::Red::decode(&limits, &mut Reader::new(&bytes)).is_ok(), length == 12);
    }
}
