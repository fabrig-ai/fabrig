//! What each version from the floor promises (protocol/channel/bridges.md, 2, 4 and 6.2).
use crate::{patches::kept::jig_worker_wire as kept, wire};
use skein_codec::schema::{Shape, VariantIx, parse};

#[test]
fn credentials_have_one_layout_from_the_floor() {
    let current = parse(kept::TEXTS.last().expect("current text").as_bytes()).expect("current schema");
    for text in kept::TEXTS {
        let schema = parse(text.as_bytes()).expect("kept schema");
        if schema.version() < wire::FLOOR {
            continue;
        }
        for name in [b"Credential".as_slice(), b"Wire"] {
            let at = schema.find(name).expect("credential record");
            let latest = current.find(name).expect("current credential record");
            assert_eq!(
                schema.fingerprints().declaration(at),
                current.fingerprints().declaration(latest),
                "credential layout at {}: {name:?}",
                schema.version()
            );
        }
    }
}

#[test]
fn running_attempt_kinds_are_present_and_total_from_the_floor() {
    let versions = u32::try_from(kept::TEXTS.len()).expect("kept version count");
    let forwards = u32::try_from(crate::patches::PATCHES.len()).expect("forward plans");
    let reverses = u32::from(wire::VERSION.checked_sub(wire::FLOOR).expect("kept floor"));
    let bridge = kept::bridge(skein_codec::patch::Capacity {
        versions,
        patches: forwards.checked_add(reverses).expect("forward and reverse plans"),
        text: 32_768,
        bytes: 1024,
        nodes: 128,
    })
    .expect("worker bridge accepts records and enumeration roots");
    // Enumeration roots are bridgeable too, even though a frame holds a record alone.
    assert_eq!(bridge.total(wire::VERSION, b"FromEngine"), Some(true));
    assert_eq!(bridge.total(wire::VERSION, b"FromWorker"), Some(true));
    for text in kept::TEXTS {
        let schema = parse(text.as_bytes()).expect("kept schema");
        if schema.version() < wire::FLOOR || schema.version() == wire::VERSION {
            continue;
        }
        let at = schema.find(b"FromEngine").expect("engine kinds");
        let Shape::Enum { first, count } = schema.declaration(at).shape else { panic!("kinds enum") };
        for name in [
            b"message".as_slice(),
            b"relayed",
            b"turn_kept",
            b"turn_busy",
            b"acknowledge",
            b"cancel",
            b"grant",
            b"sources",
            b"check",
            b"sign_out",
            b"read_list",
        ] {
            let mut found = None;
            for tag in 0..count {
                let variant = schema.variant(VariantIx::new(first.index() + tag));
                if schema.spelled(variant.name) == name {
                    found = Some(variant);
                }
            }
            let variant = found.expect("running attempt kind is present");
            let record = schema.declaration(variant.record.expect("kind's record"));
            assert_eq!(
                bridge.total(schema.version(), schema.spelled(record.name)),
                Some(true),
                "running attempt kind {name:?} at {}",
                schema.version()
            );
        }
    }
    assert_eq!(smith_channel::pieces::wire::FLOOR, smith_channel::pieces::wire::VERSION);
}

#[test]
fn kept_terms_exclude_fields_and_kinds_the_version_does_not_have() {
    let limits = super::fixtures::LIMITS;
    let schema = crate::schema(&limits).expect("terms");
    assert_eq!(schema.range(), Some((wire::FLOOR, wire::VERSION)));
    let old = schema.version(12).expect("floor");
    let current = schema.version(wire::VERSION).expect("current");
    assert_eq!(old.kind(0x018d), None);
    assert_eq!(current.kind(0x018d).expect("statement").largest, 612);
    assert_eq!(old.kind(0x0102).expect("old call").largest + 2, current.kind(0x0102).expect("current call").largest);
    for version in &schema.versions {
        let mut found = None;
        for text in kept::TEXTS {
            let text = parse(text.as_bytes()).expect("kept schema");
            if text.version() == version.version {
                found = Some(text);
            }
        }
        let text = found.expect("version's text");
        for (name, base, direction) in [
            (b"FromWorker".as_slice(), 0x0100_u16, skein_channel::Direction::FromInitiator),
            (b"FromEngine".as_slice(), 0x0180_u16, skein_channel::Direction::FromResponder),
        ] {
            let at = text.find(name).expect("kinds");
            let Shape::Enum { first, count } = text.declaration(at).shape else { panic!("kinds enum") };
            for tag in 0..count {
                let variant = text.variant(VariantIx::new(first.index() + tag));
                let bound = text
                    .most_bytes_within(variant.record.expect("record"), &crate::v14::LIMIT_NAMES, &limits.values())
                    .expect("bound");
                let kind = version.kind(base + u16::try_from(tag).expect("tag fits")).expect("kind");
                assert_eq!(kind.direction, direction);
                assert_eq!(u64::from(kind.largest), bound, "version {} kind {}", version.version, kind.kind);
            }
        }
    }
}
