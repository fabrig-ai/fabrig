//! Project record sizes into skein's schema (protocol/channel/README.md, 4.1).
use crate::{MAGIC, VERSION, v22};
use skein_channel::{Direction, Kind as ChannelKind, Role, Schema, Term};
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
    Ask,
    Asked,
    Restarting,
    Figures,
    Usage,
    UsageRead,
    ReadUsage,
    Pause,
    Paused,
    Part,
    PartKept,
    PartBusy,
    Guides,
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

    /// Whether the engine sends this kind only where the worker has it
    /// (jig/docs/design/protocol/channel/layouts.md, 4.1).
    #[must_use]
    pub fn optional_at_worker(self) -> bool {
        match self {
            Self::Restarting | Self::Figures | Self::ReadUsage | Self::Pause | Self::PartKept | Self::PartBusy => true,
            Self::Hello
            | Self::Turn
            | Self::Call
            | Self::Answer
            | Self::Fact
            | Self::Unauthorized
            | Self::Limited
            | Self::AccountRefused
            | Self::Assignment
            | Self::Message
            | Self::Relayed
            | Self::TurnKept
            | Self::TurnBusy
            | Self::Acknowledge
            | Self::Cancel
            | Self::Grant
            | Self::Sources
            | Self::Check
            | Self::ReadList
            | Self::ListRead
            | Self::Draining
            | Self::SignOut
            | Self::Logins
            | Self::SignedOut
            | Self::Job
            | Self::JobAnswer
            | Self::Ask
            | Self::Asked
            | Self::Usage
            | Self::UsageRead
            | Self::Paused
            | Self::Part
            | Self::Guides => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KindRule {
    pub kind: u16,
    pub direction: Direction,
    pub body: Kind,
}

/// The current schema's frame kinds, in wire number order.
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
    KindRule { kind: 0x010e, direction: Direction::FromInitiator, body: Kind::Usage },
    KindRule { kind: 0x010f, direction: Direction::FromInitiator, body: Kind::UsageRead },
    KindRule { kind: 0x0110, direction: Direction::FromInitiator, body: Kind::Paused },
    KindRule { kind: 0x0111, direction: Direction::FromInitiator, body: Kind::Part },
    KindRule { kind: 0x0112, direction: Direction::FromInitiator, body: Kind::Guides },
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
    KindRule { kind: 0x018d, direction: Direction::FromResponder, body: Kind::Asked },
    KindRule { kind: 0x018e, direction: Direction::FromResponder, body: Kind::Restarting },
    KindRule { kind: 0x018f, direction: Direction::FromResponder, body: Kind::Figures },
    KindRule { kind: 0x0190, direction: Direction::FromResponder, body: Kind::ReadUsage },
    KindRule { kind: 0x0191, direction: Direction::FromResponder, body: Kind::Pause },
    KindRule { kind: 0x0192, direction: Direction::FromResponder, body: Kind::PartKept },
    KindRule { kind: 0x0193, direction: Direction::FromResponder, body: Kind::PartBusy },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Overflow {
    Body,
}

pub(crate) fn current(limits: &v22::Limits) -> Result<List<ChannelKind>, Overflow> {
    let mut kinds = List::with_capacity(u32::try_from(KINDS.len()).expect("the current kinds fit"));
    for rule in KINDS {
        let largest = crate::sizes::body(limits, rule.body).ok_or(Overflow::Body)?;
        kinds.push(ChannelKind { kind: rule.kind, direction: rule.direction, largest }).expect("one row per kind");
    }
    Ok(kinds)
}

/// The channel offers the current schema at its one version.
pub fn schema(limits: &v22::Limits) -> Result<Schema, Overflow> {
    Ok(Schema { magic: MAGIC, version: VERSION, kinds: current(limits)? })
}

/// The channel table and jig's current texts.
pub fn schema_and_spoken(limits: &v22::Limits) -> Result<(Schema, List<crate::Spoken>), Overflow> {
    Ok((schema(limits)?, crate::spoken()))
}

/// The first sending kind whose peer bound does not hold this side's largest.
#[must_use]
pub fn terms_gap(table: &List<ChannelKind>, role: Role, terms: &List<Term>) -> Option<u16> {
    let sending = match role {
        Role::Initiator => Direction::FromInitiator,
        Role::Responder => Direction::FromResponder,
    };
    for rule in table {
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
