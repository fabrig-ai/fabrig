//! The schema owns numbers, directions and records (protocol/channel/wires.md, 4).
use crate::{KINDS, Kind};
use skein_channel::Direction;
use skein_codec::schema::{Shape, VariantIx};

#[test]
fn terms_use_the_agreed_versions_own_table() {
    use skein_channel::{Kind as ChannelKind, Role, Schema, Term, Version};
    use skein_lib::List;
    let mut versions = List::with_capacity(2);
    for (version, largest) in [(7, 3), (8, 5)] {
        let mut kinds = List::with_capacity(1);
        kinds.push(ChannelKind { kind: 0x0123, direction: Direction::FromInitiator, largest }).expect("one kind");
        versions.push(Version { version, kinds }).expect("two versions");
    }
    let schema = Schema { magic: crate::MAGIC, versions };
    let mut terms = List::with_capacity(1);
    terms.push(Term { kind: 0x0123, largest: 3 }).expect("one term");
    assert_eq!(crate::terms_gap(&schema, 7, Role::Initiator, &terms), None);
    assert_eq!(crate::terms_gap(&schema, 8, Role::Initiator, &terms), Some(0x0123));
}

fn record(kind: Kind) -> &'static [u8] {
    match kind {
        Kind::Ask => b"Ask",
        Kind::Asked => b"Asked",
        Kind::Wires => b"Agreed",
        Kind::Hello => b"Hello",
        Kind::Turn => b"Turn",
        Kind::Call => b"Call",
        Kind::Answer => b"Answer",
        Kind::Fact => b"Fact",
        Kind::Unauthorized => b"Unauthorized",
        Kind::Limited => b"Limited",
        Kind::AccountRefused => b"AccountRefused",
        Kind::Logins => b"Logins",
        Kind::SignedOut => b"SignedOut",
        Kind::ListRead => b"ListRead",
        Kind::Draining => b"Draining",
        Kind::JobAnswer => b"JobAnswer",
        Kind::Assignment => b"Assignment",
        Kind::Message => b"Inbound",
        Kind::Relayed => b"Relayed",
        Kind::TurnKept => b"TurnKept",
        Kind::TurnBusy => b"TurnBusy",
        Kind::Acknowledge => b"Acknowledge",
        Kind::Cancel => b"Cancel",
        Kind::Grant => b"Granted",
        Kind::Sources => b"Sources",
        Kind::Check => b"Check",
        Kind::SignOut => b"SignOut",
        Kind::Job => b"Job",
        Kind::ReadList => b"ReadList",
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the kinds test reads the current wire schema")]
fn kinds_follow_every_schema_variant() {
    let schema = skein_codec::schema::parse(include_str!("../../schema/worker-wire-v14.schema").as_bytes())
        .expect("current schema");
    let mut seen = 0;
    for (name, base, direction) in [
        (b"FromWorker".as_slice(), 0x0100_u16, Direction::FromInitiator),
        (b"FromEngine".as_slice(), 0x0180_u16, Direction::FromResponder),
    ] {
        let at = schema.find(name).expect("kinds enumeration");
        let Shape::Enum { first, count } = schema.declaration(at).shape else { panic!("kinds are an enumeration") };
        for tag in 0..count {
            let variant = schema.variant(VariantIx::new(first.index().checked_add(tag).expect("bounded variants")));
            let number = base.checked_add(u16::try_from(tag).expect("variant tag fits")).expect("kind fits");
            let mut rule = None;
            for candidate in KINDS {
                if candidate.kind == number {
                    assert!(rule.is_none(), "one rule per kind");
                    rule = Some(candidate);
                }
            }
            let rule = rule.expect("each variant has a rule");
            assert_eq!(rule.direction, direction, "{number:#06x}");
            let declaration = schema.declaration(variant.record.expect("each kind holds a record"));
            assert_eq!(schema.spelled(declaration.name), record(rule.body), "{number:#06x}");
            seen += 1;
        }
    }
    assert_eq!(seen, KINDS.len(), "every rule has a variant");
}
