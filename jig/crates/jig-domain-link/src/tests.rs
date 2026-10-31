use super::{Membership, SourceName};
use alloc::boxed::Box;

#[test]
fn source_name_stored_carrier_boundaries() {
    let bytes = [b'a'; 32];
    for length in [0, 1, 32] {
        let value = &bytes[..length];
        let name = SourceName::from_stored(value).expect("stored bytes fit the carrier");
        assert_eq!(name.as_bytes(), value);
    }
    assert_eq!(SourceName::from_stored(&[b'a'; 33]), None);
}

#[test]
fn source_name_storage_does_not_judge_declaration_spelling() {
    for value in [b"Upper".as_slice(), b"punctuation!", b"\xff\x80", b"a\0b"] {
        let name = SourceName::from_stored(value).expect("stored bytes fit the carrier");
        assert_eq!(name.as_bytes(), value);
        assert_eq!(SourceName::new(value), None);
    }
    let value = b"source-09";
    let declared = SourceName::new(value).expect("valid declaration");
    assert_eq!(declared.as_bytes(), value);
    assert_eq!(SourceName::from_stored(value), Some(declared));
}

#[test]
fn source_name_declaration_boundaries() {
    assert_eq!(SourceName::new(b""), None);
    assert_eq!(SourceName::new(&[b'a'; 33]), None);
    for value in [b"a".as_slice(), &[b'a'; 32]] {
        let declared = SourceName::new(value).expect("valid declaration fits the carrier");
        assert_eq!(declared.as_bytes(), value);
        assert_eq!(SourceName::from_stored(value), Some(declared));
    }
}

fn membership() -> Membership {
    Membership::new(Box::new([Box::from(b"alpha".as_slice()), Box::from(b"beta".as_slice())]), 2, 32)
        .expect("bounded membership")
}
#[test]
fn membership_refuses_a_position_at_its_count() {
    let membership = membership();
    assert_eq!(membership.worker(2), None);
}
#[test]
fn membership_mints_the_last_position() {
    let membership = membership();
    let worker = membership.worker(1).expect("last member");
    assert_eq!(worker.number(), 1);
    assert_eq!(membership.name(worker), b"beta");
}
