use super::fixtures::SIZES_ONE_SLOT;
use crate::{
    self as wire,
    sizes::{self, Sizes},
    v10,
};
use skein_lib::{List, Reader, Writer};

// jig/docs/design/protocol/channel/records.md, 1 bounds a login email in bytes.
const EMAIL_BYTES: usize = 255;

#[test]
fn source_caps_are_measured_without_run_slots() {
    let sizes = Sizes { slots: 0, ..SIZES_ONE_SLOT };
    assert_eq!(sizes::engine_cap(&sizes), Some(sizes::Cap { bytes: 337, frames: 6 }));
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
        assert_eq!(wire::Check::new(&limits, wire::CheckParts { name: name.clone() }).is_ok(), length == 32);
        assert_eq!(wire::SignOut::new(&limits, wire::SignOutParts { name: name.clone() }).is_ok(), length == 32);
        assert_eq!(
            wire::ProgramSource::new(&limits, wire::ProgramSourceParts { name: name.clone() }).is_ok(),
            length == 32
        );
        assert_eq!(
            wire::LoginOtherwise::new(&limits, wire::LoginOtherwiseParts { method: name.clone() }).is_ok(),
            length == 32
        );
        assert_eq!(
            wire::Login::new(
                &limits,
                wire::LoginParts { name: name.clone(), state: wire::LoginState::Usable, email: None }
            )
            .is_ok(),
            length == 32
        );
        assert_eq!(
            wire::SignedOut::new(&limits, wire::SignedOutParts { name, result: wire::SignOutResult::SignedOut })
                .is_ok(),
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
        let row = wire::Login::new(
            &limits,
            wire::LoginParts { name: b"a".as_slice().into(), state: wire::LoginState::Usable, email },
        );
        assert_eq!(row.is_ok(), valid);
        if let Ok(row) = row {
            let mut writer = Writer::new(skein_lib::length::usize_of(row.measure()));
            row.encode(&mut writer).expect("encode login");
            assert_eq!(wire::Login::decode(&limits, &mut Reader::new(&writer.finish())), Ok(row));
        }
    }
    for length in [16, 17] {
        assert_eq!(
            wire::SourceFailed::new(
                &limits,
                wire::SourceFailedParts {
                    reason: wire::SourceFailure::Exited,
                    detail: Box::from([b'e'; 17].get(..length).expect("detail fixture")),
                }
            )
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
            names
                .push(wire::ProgramSource::new(&limits, wire::ProgramSourceParts { name: name.clone() }).expect("name"))
                .expect("list");
            logins
                .push(
                    wire::Login::new(
                        &limits,
                        wire::LoginParts { name, state: wire::LoginState::NotSignedIn, email: None },
                    )
                    .expect("row"),
                )
                .expect("list");
        }
        let sources = wire::Sources::new(&limits, wire::SourcesParts { sources: names });
        let logins = wire::Logins::new(&limits, wire::LoginsParts { logins });
        assert_eq!(sources.is_ok(), count <= 2);
        assert_eq!(logins.is_ok(), count <= 2);
        if let Ok(sources) = sources {
            let mut writer = Writer::new(skein_lib::length::usize_of(sources.measure()));
            sources.encode(&mut writer).expect("encode");
            assert_eq!(wire::Sources::decode(&limits, &mut Reader::new(&writer.finish())), Ok(sources));
        }
    }
    v10::check_limits(&limits).expect("source limits");
}
