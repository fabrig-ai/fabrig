use super::fixtures::SIZES_ONE_SLOT;
use crate::v17::Record;
use crate::{
    self as wire,
    sizes::{self, Sizes},
    v18,
};
use skein_lib::{List, Reader};

// jig/docs/design/protocol/channel/records.md, 1 bounds a login email in bytes.
const EMAIL_BYTES: usize = 255;

#[test]
fn source_caps_are_measured_without_run_slots() {
    let sizes = Sizes { slots: 0, ..SIZES_ONE_SLOT };
    // Source commands and a read list (337), plus the host-wide statement (620) and figures (10,062,744).
    assert_eq!(sizes::engine_cap(&sizes), Some(sizes::Cap { bytes: 10_063_701, frames: 8 }));
}
#[test]
fn source_record_bodies_are_measured() {
    let sizes = Sizes { slots: 0, ..SIZES_ONE_SLOT };
    for (kind, bytes) in [
        (wire::Kind::Sources, 76),
        (wire::Kind::Check, 36),
        (wire::Kind::SignOut, 36),
        (wire::Kind::Logins, 670),
        (wire::Kind::SignedOut, 58),
    ] {
        assert_eq!(sizes::largest_body(&sizes, kind), Some(bytes));
    }
}
#[test]
fn source_record_fields_refuse_one_byte_past_each_bound() {
    let limits = sizes::limits(&SIZES_ONE_SLOT).expect("tiny bounds");
    for length in [32, 33] {
        let name: Box<[u8]> = Box::from([b'a'; 33].get(..length).expect("field fixture"));
        assert_eq!((wire::Check { name: name.clone() }).encode(&limits).is_ok(), length == 32);
        assert_eq!((wire::SignOut { name: name.clone() }).encode(&limits).is_ok(), length == 32);
        assert_eq!((wire::ProgramSource { name: name.clone() }).encode(&limits).is_ok(), length == 32);
        assert_eq!((wire::LoginOtherwise { method: name.clone() }).encode(&limits).is_ok(), length == 32);
        assert_eq!(
            (wire::Login { name: name.clone(), state: wire::LoginState::Usable, email: None }).encode(&limits).is_ok(),
            length == 32
        );
        assert_eq!(
            (wire::SignedOut { name, result: wire::SignOutResult::SignedOut }).encode(&limits).is_ok(),
            length == 32
        );
    }
    // protocol/channel/records.md, 1: optional UTF-8 email, at most 255 bytes.
    for (email, valid) in [
        (None, true),
        (Some(Box::from(b"you@example.com".as_slice())), true),
        (Some(Box::from([b'e'; EMAIL_BYTES])), true),
        (Some(Box::from([b'e'; EMAIL_BYTES + 1])), false),
        (Some(Box::from([0xff])), false),
    ] {
        let row = wire::Login { name: b"a".as_slice().into(), state: wire::LoginState::Usable, email };
        let encoded = row.encode(&limits);
        assert_eq!(encoded.is_ok(), valid);
        if let Ok(bytes) = encoded {
            assert_eq!(wire::Login::decode(&limits, &mut Reader::new(&bytes)), Ok(row));
        }
    }
    for length in [16, 17] {
        assert_eq!(
            (wire::SourceFailed {
                reason: wire::SourceFailure::Exited,
                detail: Box::from([b'e'; 17].get(..length).expect("detail fixture")),
            })
            .encode(&limits)
            .is_ok(),
            length == 16
        );
    }
}
#[test]
fn whole_source_lists_enforce_bounds_through_round_trip() {
    let limits = sizes::limits(&SIZES_ONE_SLOT).expect("tiny bounds");
    for count in [0, 2, 3] {
        let mut names = List::with_capacity(count);
        let mut logins = List::with_capacity(count);
        for index in 0..count {
            let name: Box<[u8]> = Box::from([b'a' + u8::try_from(index).expect("three sources")].as_slice());
            names.push(wire::ProgramSource { name: name.clone() }).expect("list");
            logins.push(wire::Login { name, state: wire::LoginState::NotSignedIn, email: None }).expect("list");
        }
        let sources = wire::Sources { sources: names };
        let encoded = sources.encode(&limits);
        let logins = wire::Logins { logins }.encode(&limits);
        assert_eq!(encoded.is_ok(), count <= 2);
        assert_eq!(logins.is_ok(), count <= 2);
        if let Ok(bytes) = encoded {
            assert_eq!(wire::Sources::decode(&limits, &mut Reader::new(&bytes)), Ok(sources));
        }
    }
    v18::check_limits(&limits).expect("source limits");
}
