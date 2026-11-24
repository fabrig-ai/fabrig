//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use crate::Limits;
use crate::generated::client;
use crate::translate::text_problem;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use skein_lib::bytes;

pub(crate) fn person_address(value: &w::PersonAddress) -> c::PersonAddress {
    client::person_address_to_domain(value.clone())
}

pub(crate) fn recipient(value: &w::Recipient) -> c::Recipient {
    client::recipient_to_domain(value.clone())
}

pub(crate) fn actor(value: &w::Actor) -> c::Actor {
    client::actor_to_domain(value.clone())
}

pub(crate) fn party_kind(value: &w::PartyKind) -> c::PartyKind {
    client::party_kind_to_domain(value.clone())
}

pub(crate) fn party(value: &w::Party, limits: &Limits) -> Result<c::Party, w::Problem> {
    Ok(c::Party {
        number: value.number,
        kind: party_kind(&value.kind),
        name: match c::Text::new(bytes::copy_of(&value.name), limits.client().text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
    })
}

pub(crate) fn decision(value: &w::Decision, limits: &Limits) -> Result<c::Decision, w::Problem> {
    Ok(match value {
        w::Decision::Accepted => c::Decision::Accepted,
        w::Decision::Rejected(row) => c::Decision::Rejected {
            reason: match c::Text::new(bytes::copy_of(&row.reason), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::Decision::PassedUp => c::Decision::PassedUp,
        w::Decision::Released(row) => {
            let opt0 = match &row.again {
                Some(item) => Some(*item),
                None => None,
            };
            c::Decision::Released { again: opt0 }
        }
        w::Decision::LeftHeld(row) => c::Decision::LeftHeld {
            reason: match c::Text::new(bytes::copy_of(&row.reason), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
    })
}

pub(crate) fn person_wait(value: &w::PersonWait) -> c::PersonWait {
    client::person_wait_to_domain(value.clone())
}

pub(crate) fn escalation_holder(value: &w::EscalationHolder) -> c::EscalationHolder {
    client::escalation_holder_to_domain(value.clone())
}

pub(crate) fn requester(value: &w::Requester) -> c::Requester {
    client::requester_to_domain(value.clone())
}
