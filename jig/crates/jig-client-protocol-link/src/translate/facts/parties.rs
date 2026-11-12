//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use crate::Limits;
use crate::translate::text_problem;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v20 as w;
use skein_lib::{List, Wall, bytes};

pub(crate) fn person_address(value: &w::PersonAddress) -> c::PersonAddress {
    match value {
        w::PersonAddress::Party(row) => c::PersonAddress::Party(row.value()),
        w::PersonAddress::Role(row) => c::PersonAddress::Role(row.value()),
    }
}

pub(crate) fn proposer(value: &w::Proposer) -> c::Proposer {
    match value {
        w::Proposer::Task(row) => c::Proposer::Task(row.value()),
        w::Proposer::Party(row) => c::Proposer::Party(row.value()),
    }
}

pub(crate) fn recipient(value: &w::Recipient) -> c::Recipient {
    match value {
        w::Recipient::Task(row) => c::Recipient::Task(row.value()),
        w::Recipient::Party(row) => c::Recipient::Party(row.value()),
        w::Recipient::Role(row) => c::Recipient::Role { project: row.project(), role: row.role() },
        w::Recipient::Roles(row) => {
            let mut list0 = List::with_capacity(row.roles().len());
            for item in row.roles() {
                list0.push(*item).expect("reserved supplied list");
            }
            c::Recipient::Roles { project: row.project(), roles: list0 }
        }
    }
}

pub(crate) fn actor(value: &w::Actor) -> c::Actor {
    match value {
        w::Actor::Party(row) => c::Actor::Party(row.value()),
        w::Actor::Task(row) => c::Actor::Task(row.value()),
        w::Actor::Deployment => c::Actor::Deployment,
        w::Actor::ProjectDeployment(row) => c::Actor::ProjectDeployment { project: row.project() },
    }
}

pub(crate) fn party_kind(value: &w::PartyKind) -> c::PartyKind {
    match value {
        w::PartyKind::Person => c::PartyKind::Person,
        w::PartyKind::Service => c::PartyKind::Service,
    }
}

pub(crate) fn party(value: &w::Party, limits: &Limits) -> Result<c::Party, w::Problem> {
    Ok(c::Party {
        number: value.number(),
        kind: party_kind(value.kind()),
        name: match c::Text::new(bytes::copy_of(value.name()), limits.client().text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
    })
}

pub(crate) fn decision(value: &w::Decision, limits: &Limits) -> Result<c::Decision, w::Problem> {
    Ok(match value {
        w::Decision::Accepted => c::Decision::Accepted,
        w::Decision::Rejected(row) => c::Decision::Rejected {
            reason: match c::Text::new(bytes::copy_of(row.reason()), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::Decision::PassedUp => c::Decision::PassedUp,
        w::Decision::Released(row) => {
            let opt0 = match row.again() {
                Some(item) => Some(*item),
                None => None,
            };
            c::Decision::Released { again: opt0 }
        }
        w::Decision::LeftHeld(row) => c::Decision::LeftHeld {
            reason: match c::Text::new(bytes::copy_of(row.reason()), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
    })
}

pub(crate) fn person_stall_notice(value: &w::PersonStallNotice) -> c::PersonStallNotice {
    c::PersonStallNotice { message: value.message(), at: Wall::from_nanos(value.at()), activity: value.activity() }
}

pub(crate) fn person_wait(value: &w::PersonWait) -> c::PersonWait {
    let opt0 = match &value.notice() {
        Some(item) => Some(person_stall_notice(item)),
        None => None,
    };
    c::PersonWait {
        since: Wall::from_nanos(value.since()),
        stall: value.stall(),
        deadline: Wall::from_nanos(value.deadline()),
        notice: opt0,
    }
}

pub(crate) fn escalation_holder(value: &w::EscalationHolder) -> c::EscalationHolder {
    match value {
        w::EscalationHolder::Task(row) => c::EscalationHolder::Task(row.value()),
        w::EscalationHolder::Person(row) => c::EscalationHolder::Person(row.value()),
        w::EscalationHolder::Role(row) => c::EscalationHolder::Role { project: row.project(), role: row.role() },
    }
}

pub(crate) fn requester(value: &w::Requester) -> c::Requester {
    match value {
        w::Requester::Task(row) => c::Requester::Task(row.value()),
        w::Requester::Person(row) => c::Requester::Person(row.value()),
        w::Requester::Deployment(row) => c::Requester::Deployment { project: row.project() },
    }
}
