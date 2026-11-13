//! Project record sizes into skein's schema (protocol/channel/README.md, 4.1).
use crate::{MAGIC, VERSION, v12};
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
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Overflow {
    Body,
    Frame,
}

pub fn schema(limits: &v12::Limits) -> Result<Schema, Overflow> {
    let mut kinds = List::with_capacity(u32::try_from(KINDS.len()).expect("kind count fits"));
    for rule in KINDS {
        let largest = match rule.body {
            Kind::Hello => v12::Hello::encoded_bound(limits),
            Kind::Turn => v12::Turn::encoded_bound(limits),
            Kind::Call => v12::Call::encoded_bound(limits),
            Kind::Answer => v12::Answer::encoded_bound(limits),
            Kind::Fact => v12::Fact::encoded_bound(limits),
            Kind::Unauthorized => v12::Unauthorized::encoded_bound(limits),
            Kind::Limited => v12::Limited::encoded_bound(limits),
            Kind::AccountRefused => v12::AccountRefused::encoded_bound(limits),
            Kind::Assignment => v12::Assignment::encoded_bound(limits),
            Kind::Message => v12::Inbound::encoded_bound(limits),
            Kind::Relayed => v12::Relayed::encoded_bound(limits),
            Kind::TurnKept => v12::TurnKept::encoded_bound(limits),
            Kind::TurnBusy => v12::TurnBusy::encoded_bound(limits),
            Kind::Acknowledge => v12::Acknowledge::encoded_bound(limits),
            Kind::Cancel => v12::Cancel::encoded_bound(limits),
            Kind::Sources => v12::Sources::encoded_bound(limits),
            Kind::ReadList => v12::ReadList::encoded_bound(limits),
            Kind::Draining => v12::Draining::encoded_bound(limits),
            Kind::ListRead => v12::ListRead::encoded_bound(limits),
            Kind::Check => v12::Check::encoded_bound(limits),
            Kind::SignOut => v12::SignOut::encoded_bound(limits),
            Kind::Logins => v12::Logins::encoded_bound(limits),
            Kind::SignedOut => v12::SignedOut::encoded_bound(limits),
            Kind::Job => v12::Job::encoded_bound(limits),
            Kind::JobAnswer => v12::JobAnswer::encoded_bound(limits),
            Kind::Grant => v12::Granted::encoded_bound(limits),
        };
        let largest = largest.ok_or(Overflow::Body)?;
        let Ok(largest) = u32::try_from(largest) else { return Err(Overflow::Frame) };
        kinds.push(ChannelKind { kind: rule.kind, direction: rule.direction, largest }).expect("one slot per kind");
    }
    let mut versions = List::with_capacity(1);
    versions.push(Version { version: VERSION, kinds }).expect("one version has one slot");
    Ok(Schema { magic: MAGIC, versions })
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
