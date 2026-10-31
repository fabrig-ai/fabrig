//! Smith's one-version body table. It keeps no state and knows no domain;
//! [`schema`] projects the generated body bounds into Skein's channel table.
//! Contract: `protocol/channel.md`, sections 2-4.

use skein_channel::{Direction, Kind, Role, Schema, Term, Version};
use skein_lib::List;

use crate::{kind, v5};

/// A version-five body's number, sender, admission obligation and codec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(
    clippy::partial_pub_fields,
    reason = "the codec selector is internal; number, direction and obligation are public"
)]
pub struct KindRule {
    pub kind: u16,
    pub direction: Direction,
    pub required: bool,
    body: Body,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Body {
    Start,
    Message,
    MessageRefused,
    HostAnswer,
    Acknowledge,
    GrantRefresh,
    Cancel,
    Admitted,
    Call,
    Withdraw,
    Turn,
    Waiting,
    Long,
    LongDone,
    Unauthorized,
    Limited,
    Fact,
    Answer,
    Live,
    AccountRefused,
}

/// Why a body maximum cannot be represented in the channel's frame length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    /// Generated bound arithmetic exceeded its checked range.
    Body,
    /// The body exceeds a frame's `u32` length.
    Frame,
}

/// The 20 kinds required by channel version five, in wire number order.
pub const V5: &[KindRule] = &[
    KindRule { kind: kind::START, direction: Direction::FromInitiator, required: true, body: Body::Start },
    KindRule { kind: kind::MESSAGE, direction: Direction::FromInitiator, required: true, body: Body::Message },
    KindRule { kind: kind::HOST_ANSWER, direction: Direction::FromInitiator, required: true, body: Body::HostAnswer },
    KindRule { kind: kind::ACKNOWLEDGE, direction: Direction::FromInitiator, required: true, body: Body::Acknowledge },
    KindRule {
        kind: kind::GRANT_REFRESH,
        direction: Direction::FromInitiator,
        required: true,
        body: Body::GrantRefresh,
    },
    KindRule { kind: kind::CANCEL, direction: Direction::FromInitiator, required: true, body: Body::Cancel },
    KindRule { kind: kind::ADMITTED, direction: Direction::FromResponder, required: true, body: Body::Admitted },
    KindRule { kind: kind::CALL, direction: Direction::FromResponder, required: true, body: Body::Call },
    KindRule { kind: kind::WITHDRAW, direction: Direction::FromResponder, required: true, body: Body::Withdraw },
    KindRule { kind: kind::TURN, direction: Direction::FromResponder, required: true, body: Body::Turn },
    KindRule { kind: kind::WAITING, direction: Direction::FromResponder, required: true, body: Body::Waiting },
    KindRule { kind: kind::LONG, direction: Direction::FromResponder, required: true, body: Body::Long },
    KindRule { kind: kind::LONG_DONE, direction: Direction::FromResponder, required: true, body: Body::LongDone },
    KindRule {
        kind: kind::UNAUTHORIZED,
        direction: Direction::FromResponder,
        required: true,
        body: Body::Unauthorized,
    },
    KindRule { kind: kind::LIMITED, direction: Direction::FromResponder, required: true, body: Body::Limited },
    KindRule { kind: kind::FACT, direction: Direction::FromResponder, required: true, body: Body::Fact },
    KindRule { kind: kind::ANSWER, direction: Direction::FromResponder, required: true, body: Body::Answer },
    KindRule {
        kind: kind::MESSAGE_REFUSED,
        direction: Direction::FromResponder,
        required: true,
        body: Body::MessageRefused,
    },
    KindRule { kind: kind::LIVE, direction: Direction::FromResponder, required: true, body: Body::Live },
    KindRule {
        kind: kind::ACCOUNT_REFUSED,
        direction: Direction::FromResponder,
        required: true,
        body: Body::AccountRefused,
    },
];

/// Build the version-five channel schema from receiving codec limits.
pub fn schema(limits: &v5::Limits) -> Result<Schema, Overflow> {
    let mut kinds = List::with_capacity(20);
    for rule in V5 {
        let largest = match rule.body {
            Body::Start => v5::Start::encoded_bound(limits),
            Body::Message => v5::Message::encoded_bound(limits),
            Body::MessageRefused => v5::MessageRefused::encoded_bound(limits),
            Body::HostAnswer => v5::HostAnswer::encoded_bound(limits),
            Body::Acknowledge => v5::Acknowledge::encoded_bound(limits),
            Body::GrantRefresh => v5::GrantRefresh::encoded_bound(limits),
            Body::Cancel => v5::Cancel::encoded_bound(limits),
            Body::Admitted => v5::Admitted::encoded_bound(limits),
            Body::Call => v5::Call::encoded_bound(limits),
            Body::Withdraw => v5::Withdraw::encoded_bound(limits),
            Body::Turn => v5::Turn::encoded_bound(limits),
            Body::Waiting => v5::Waiting::encoded_bound(limits),
            Body::Long => v5::Long::encoded_bound(limits),
            Body::LongDone => v5::LongDone::encoded_bound(limits),
            Body::Unauthorized => v5::Unauthorized::encoded_bound(limits),
            Body::Limited => v5::Limited::encoded_bound(limits),
            Body::Fact => v5::Fact::encoded_bound(limits),
            Body::Answer => v5::Answer::encoded_bound(limits),
            Body::AccountRefused => v5::AccountRefused::encoded_bound(limits),
            Body::Live => v5::Live::encoded_bound(limits),
        };
        let largest = largest.ok_or(Overflow::Body)?;
        let Ok(largest) = u32::try_from(largest) else {
            return Err(Overflow::Frame);
        };
        kinds.push(Kind { kind: rule.kind, direction: rule.direction, largest }).expect("V5 has exactly 20 kinds");
    }
    let mut versions = List::with_capacity(1);
    versions.push(Version { version: kind::VERSION, kinds }).expect("one version has one slot");
    Ok(Schema { magic: *b"smth", versions })
}

/// Find the first required kind that the peer cannot receive at our sending bound.
#[must_use]
pub fn peer_terms_gap(schema: &Schema, role: Role, version: u16, terms: &List<Term>) -> Option<u16> {
    required_terms_gap(schema, role, version, terms, V5)
}

fn required_terms_gap(
    schema: &Schema,
    role: Role,
    version: u16,
    terms: &List<Term>,
    rules: &[KindRule],
) -> Option<u16> {
    let sending = match role {
        Role::Initiator => Direction::FromInitiator,
        Role::Responder => Direction::FromResponder,
    };
    let table = schema.version(version)?;
    for rule in rules {
        if rule.direction == sending && rule.required {
            let own = table.kind(rule.kind)?;
            let mut accepted = false;
            for term in terms {
                if term.kind == rule.kind && term.largest >= own.largest {
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

impl KindRule {
    /// The wire format version this codec writes and reads.
    pub const FORMAT_VERSION: u64 = 5;
}

#[cfg(test)]
mod optional_tests {
    use super::{Body, KindRule, required_terms_gap};
    use crate::kind;
    use skein_channel::{Direction, Kind, Role, Schema, Term, Version};
    use skein_lib::List;

    #[test]
    fn an_older_peer_may_leave_a_test_only_optional_kind_out() {
        let rules = [
            KindRule { kind: 0x0100, direction: Direction::FromInitiator, required: true, body: Body::Start },
            KindRule { kind: 0x0101, direction: Direction::FromInitiator, required: false, body: Body::Message },
            KindRule { kind: 0x0102, direction: Direction::FromResponder, required: true, body: Body::Answer },
            KindRule { kind: 0x0103, direction: Direction::FromResponder, required: false, body: Body::Fact },
        ];
        let mut kinds = List::with_capacity(4);
        for rule in rules {
            kinds.push(Kind { kind: rule.kind, direction: rule.direction, largest: 32 }).expect("four test kinds");
        }
        let mut versions = List::with_capacity(1);
        versions.push(Version { version: kind::VERSION, kinds }).expect("one test version");
        let schema = Schema { magic: *b"smth", versions };
        let mut host_terms = List::with_capacity(1);
        host_terms.push(Term { kind: 0x0100, largest: 32 }).expect("one required kind");
        assert_eq!(required_terms_gap(&schema, Role::Initiator, 5, &host_terms, &rules), None);
        assert_eq!(required_terms_gap(&schema, Role::Initiator, 5, &List::with_capacity(0), &rules), Some(0x0100));
        let mut agent_terms = List::with_capacity(1);
        agent_terms.push(Term { kind: 0x0102, largest: 32 }).expect("one required kind");
        assert_eq!(required_terms_gap(&schema, Role::Responder, 5, &agent_terms, &rules), None);
        assert_eq!(required_terms_gap(&schema, Role::Responder, 5, &List::with_capacity(0), &rules), Some(0x0102));
    }
}
