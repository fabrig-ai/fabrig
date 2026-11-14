//! Project record sizes into skein's schema (protocol/channel/README.md, 4.1).
use crate::{MAGIC, VERSION, v14};
use skein_channel::{Direction, Kind as ChannelKind, Role, Schema, Term, Version};
use skein_lib::List;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Hello,
    Turn,
    Call,
    Answer,
    Fact,
    Unauthorized,
    Limited,
    AccountRefused,
    Assignment,
    Message,
    Relayed,
    TurnKept,
    TurnBusy,
    Acknowledge,
    Cancel,
    Grant,
    Sources,
    Check,
    ReadList,
    ListRead,
    Draining,
    SignOut,
    Logins,
    SignedOut,
    Job,
    JobAnswer,
    Wires,
    Ask,
    Asked,
}

impl Kind {
    /// The record's allocated worker-channel number.
    #[must_use]
    pub fn number(self) -> u16 {
        for rule in KINDS {
            if rule.body == self {
                return rule.kind;
            }
        }
        unreachable!("every worker record has one kind rule")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KindRule {
    pub kind: u16,
    pub direction: Direction,
    pub body: Kind,
}

/// Every kind is required, in wire number order.
#[rustfmt::skip]
pub const KINDS: &[KindRule] = &[
    KindRule { kind: 0x0100, direction: Direction::FromInitiator, body: Kind::Hello },
    KindRule { kind: 0x0101, direction: Direction::FromInitiator, body: Kind::Turn },
    KindRule { kind: 0x0102, direction: Direction::FromInitiator, body: Kind::Call },
    KindRule { kind: 0x0103, direction: Direction::FromInitiator, body: Kind::Answer },
    KindRule { kind: 0x0104, direction: Direction::FromInitiator, body: Kind::Fact },
    KindRule { kind: 0x0105, direction: Direction::FromInitiator, body: Kind::Unauthorized },
    KindRule { kind: 0x0106, direction: Direction::FromInitiator, body: Kind::Limited },
    KindRule { kind: 0x0107, direction: Direction::FromInitiator, body: Kind::AccountRefused },
    KindRule { kind: 0x0108, direction: Direction::FromInitiator, body: Kind::Logins },
    KindRule { kind: 0x0109, direction: Direction::FromInitiator, body: Kind::SignedOut },
    KindRule { kind: 0x010a, direction: Direction::FromInitiator, body: Kind::ListRead },
    KindRule { kind: 0x010b, direction: Direction::FromInitiator, body: Kind::Draining },
    KindRule { kind: 0x010c, direction: Direction::FromInitiator, body: Kind::JobAnswer },
    KindRule { kind: 0x010d, direction: Direction::FromInitiator, body: Kind::Ask },
    KindRule { kind: 0x0180, direction: Direction::FromResponder, body: Kind::Assignment },
    KindRule { kind: 0x0181, direction: Direction::FromResponder, body: Kind::Message },
    KindRule { kind: 0x0182, direction: Direction::FromResponder, body: Kind::Relayed },
    KindRule { kind: 0x0183, direction: Direction::FromResponder, body: Kind::TurnKept },
    KindRule { kind: 0x0184, direction: Direction::FromResponder, body: Kind::TurnBusy },
    KindRule { kind: 0x0185, direction: Direction::FromResponder, body: Kind::Acknowledge },
    KindRule { kind: 0x0186, direction: Direction::FromResponder, body: Kind::Cancel },
    KindRule { kind: 0x0187, direction: Direction::FromResponder, body: Kind::Grant },
    KindRule { kind: 0x0188, direction: Direction::FromResponder, body: Kind::Sources },
    KindRule { kind: 0x0189, direction: Direction::FromResponder, body: Kind::Check },
    KindRule { kind: 0x018a, direction: Direction::FromResponder, body: Kind::SignOut },
    KindRule { kind: 0x018b, direction: Direction::FromResponder, body: Kind::Job },
    KindRule { kind: 0x018c, direction: Direction::FromResponder, body: Kind::ReadList },
    KindRule { kind: 0x018d, direction: Direction::FromResponder, body: Kind::Wires },
    KindRule { kind: 0x018e, direction: Direction::FromResponder, body: Kind::Asked },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Overflow {
    Body,
    Frame,
}

pub fn schema(limits: &v14::Limits) -> Result<Schema, Overflow> {
    use skein_codec::schema::{Shape, VariantIx};
    let mut versions = List::uncapped();
    for text in crate::patches::kept::jig_worker_wire::TEXTS {
        let kept = skein_codec::schema::parse(text.as_bytes()).expect("valid kept worker schema");
        let version = kept.version();
        if version < crate::wire::FLOOR {
            continue;
        }
        let mut kinds = List::uncapped();
        for (name, base, direction) in [
            (b"FromWorker".as_slice(), 0x0100_u16, Direction::FromInitiator),
            (b"FromEngine".as_slice(), 0x0180_u16, Direction::FromResponder),
        ] {
            let at = kept.find(name).expect("kept kinds enumeration");
            let Shape::Enum { first, count } = kept.declaration(at).shape else {
                unreachable!("kinds are enumerations")
            };
            for tag in 0..count {
                let variant = kept.variant(VariantIx::new(first.index().checked_add(tag).expect("bounded variants")));
                let kind = base.checked_add(u16::try_from(tag).expect("variant tag fits")).expect("kind fits");
                let largest = if version == VERSION {
                    let rule = current_kind(kind);
                    crate::sizes::body(limits, rule)
                } else {
                    let record = variant.record.expect("each kind holds a record");
                    let bound =
                        kept.most_bytes_within(record, &v14::LIMIT_NAMES, &limits.values()).ok_or(Overflow::Body)?;
                    let Ok(bound) = u32::try_from(bound) else { return Err(Overflow::Frame) };
                    Some(bound)
                }
                .ok_or(Overflow::Body)?;
                kinds.push(ChannelKind { kind, direction, largest }).expect("uncapped kinds");
            }
        }
        versions.push(Version { version, kinds }).expect("uncapped versions");
    }
    Ok(Schema { magic: MAGIC, versions })
}

/// The channel offers the range stated for jig's own wire.
#[must_use]
pub fn schema_for(limits: &v14::Limits, wires: &List<crate::Spoken>) -> Option<Schema> {
    let mut stated = None;
    for wire in wires {
        if wire.name == crate::wire::FAMILY {
            if stated.is_some() || wire.layouts.is_empty() {
                return None;
            }
            stated = Some(wire);
        }
    }
    let stated = stated?;
    let full = schema(limits).ok()?;
    full.version(stated.lowest)?;
    full.version(stated.highest())?;
    let mut versions = List::uncapped();
    for version in &full.versions {
        if version.version >= stated.lowest && version.version <= stated.highest() {
            versions.push(version.clone()).expect("uncapped versions");
        }
    }
    Some(Schema { magic: MAGIC, versions })
}

fn current_kind(number: u16) -> Kind {
    for rule in KINDS {
        if rule.kind == number {
            return rule.body;
        }
    }
    unreachable!("every current variant has a kind rule")
}

/// The first sending kind whose peer bound does not hold this side's largest.
#[must_use]
pub fn terms_gap(schema: &Schema, version: u16, role: Role, terms: &List<Term>) -> Option<u16> {
    let sending = match role {
        Role::Initiator => Direction::FromInitiator,
        Role::Responder => Direction::FromResponder,
    };
    let table = schema.version(version)?;
    for rule in &table.kinds {
        if rule.direction == sending {
            let mut accepted = false;
            for term in terms {
                if term.kind == rule.kind && term.largest >= rule.largest {
                    accepted = true;
                }
            }
            if !accepted {
                return Some(rule.kind);
            }
        }
    }
    None
}
