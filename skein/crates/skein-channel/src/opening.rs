//! Opening policy that is mechanical rather than the owner's decision
//! (net/channel.md, sections 5.1–5.3). The machine checks magic, phase
//! and terms; its owner alone judges credentials and acceptable limits.
use alloc::boxed::Box;
use skein_lib::{List, Writer};

use super::schema::KindTableError;
use super::{Direction, Limits, Role, Schema, Term};

/// Limits refusal for a responder's invalid replacement table (net/channel.md, 5.2).
pub(super) fn table_words(error: KindTableError) -> Box<[u8]> {
    let prefix = b"the responder's table: ";
    let why: &[u8] = match error {
        KindTableError::TooManyKinds => b"too many kinds",
        KindTableError::ReservedKind => b"reserved kind",
        KindTableError::BodyCapacity => b"body capacity is not representable",
        KindTableError::OutputCapacity => b"sent frame exceeds the output cap",
        KindTableError::DuplicateKind => b"duplicate kind",
    };
    let length = prefix.len().checked_add(why.len()).expect("fixed table refusal text fits");
    let mut out = Writer::new(length);
    out.put(prefix).expect("measured table refusal");
    out.put(why).expect("measured table refusal");
    out.finish()
}
/// Builds the terms for kinds this side receives.
pub(super) fn local_terms(schema: &Schema, role: Role, limits: &Limits) -> List<Term> {
    let mut terms = List::with_capacity(limits.kinds);
    for kind in &schema.kinds {
        let receive = match role {
            Role::Initiator => match kind.direction {
                Direction::FromInitiator => false,
                Direction::FromResponder => true,
            },
            Role::Responder => match kind.direction {
                Direction::FromInitiator => true,
                Direction::FromResponder => false,
            },
        };
        if receive {
            terms.push(Term { kind: kind.kind, largest: kind.largest }).expect("schema kinds fit limits");
        }
    }
    terms
}

/// Checks that peer terms name only distinct kinds this side may send.
#[expect(clippy::manual_let_else, reason = "step code uses an exhaustive match instead of let-else")]
pub(super) fn check_terms(schema: &Schema, role: Role, terms: &List<Term>) -> bool {
    for (index, term) in terms.iter().enumerate() {
        let kind = match schema.kind(term.kind) {
            Some(kind) => kind,
            None => return false,
        };
        let send = match role {
            Role::Initiator => match kind.direction {
                Direction::FromInitiator => true,
                Direction::FromResponder => false,
            },
            Role::Responder => match kind.direction {
                Direction::FromInitiator => false,
                Direction::FromResponder => true,
            },
        };
        if !send {
            return false;
        }
        for earlier in terms.iter().take(index) {
            if earlier.kind == term.kind {
                return false;
            }
        }
    }
    true
}
