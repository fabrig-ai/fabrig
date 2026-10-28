//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_part_scope(value: jig_client::contract::PartScope, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::PartScope::Project(field_0) => {
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(*(field_0)));
            out.object_end();
        }
        jig_client::contract::PartScope::Deployment => out.string(b"deployment"),
    }
}

pub(super) fn encode_party(value: &jig_client::contract::Party, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"number");
    out.unsigned(value.number);
    out.key(b"kind");
    encode_party_kind(value.kind, out, budget);
    out.key(b"name");
    budget.words(value.name.bytes(), out);
    out.object_end();
}

pub(super) fn encode_party_kind(value: jig_client::contract::PartyKind, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::PartyKind::Person => out.string(b"person"),
        jig_client::contract::PartyKind::Service => out.string(b"service"),
    }
}

pub(super) fn encode_passed(value: &jig_client::contract::Passed, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"by");
    super::encode_actor(&value.by, out, budget);
    out.key(b"at");
    out.unsigned(value.at.as_nanos());
    out.key(b"reason");
    budget.words(value.reason.bytes(), out);
    out.key(b"lacking");
    match &value.lacking {
        Some(item_1) => {
            super::encode_lacking(item_1, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_person_address(
    value: &jig_client::contract::PersonAddress,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::PersonAddress::Party(field_0) => {
            out.object_start();
            out.key(b"party");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::PersonAddress::Role(field_0) => {
            out.object_start();
            out.key(b"role");
            out.unsigned(u64::from(*(field_0)));
            out.object_end();
        }
    }
}

pub(super) fn encode_phase(value: jig_client::contract::Phase, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::Phase::Waiting => out.string(b"waiting"),
        jig_client::contract::Phase::Idle => out.string(b"idle"),
        jig_client::contract::Phase::Due => out.string(b"due"),
        jig_client::contract::Phase::Preparing => out.string(b"preparing"),
        jig_client::contract::Phase::Claimed => out.string(b"claimed"),
        jig_client::contract::Phase::Running => out.string(b"running"),
        jig_client::contract::Phase::BackingOff => out.string(b"backing_off"),
        jig_client::contract::Phase::Active => out.string(b"active"),
        jig_client::contract::Phase::Closing => out.string(b"closing"),
        jig_client::contract::Phase::Held => out.string(b"held"),
        jig_client::contract::Phase::Ended => out.string(b"ended"),
        jig_client::contract::Phase::Done => out.string(b"done"),
        jig_client::contract::Phase::Failed => out.string(b"failed"),
        jig_client::contract::Phase::Cancelled => out.string(b"cancelled"),
    }
}

pub(super) fn encode_phase_counts(
    value: &jig_client::contract::PhaseCounts,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"ended");
    out.unsigned(u64::from(value.ended));
    out.key(b"waiting");
    out.unsigned(u64::from(value.waiting));
    out.key(b"running");
    out.unsigned(u64::from(value.running));
    out.key(b"held");
    out.unsigned(u64::from(value.held));
    out.key(b"closing");
    out.unsigned(u64::from(value.closing));
    out.key(b"done");
    out.unsigned(u64::from(value.done));
    out.key(b"failed");
    out.unsigned(u64::from(value.failed));
    out.key(b"cancelled");
    out.unsigned(u64::from(value.cancelled));
    out.object_end();
}

pub(super) fn encode_pool_ledger(value: &jig_client::contract::PoolLedger, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"source");
    super::encode_funder(&value.source, out, budget);
    out.key(b"parent");
    match &value.parent {
        Some(item_1) => {
            super::encode_funder(item_1, out, budget);
        }
        None => out.null(),
    }
    out.key(b"numbers");
    super::encode_owner_numbers(&value.numbers, out, budget);
    out.key(b"left");
    out.unsigned(value.left);
    out.key(b"closed");
    out.boolean(value.closed);
    out.key(b"began");
    match &value.began {
        Some(item_2) => {
            out.unsigned((item_2).as_nanos());
        }
        None => out.null(),
    }
    out.key(b"next");
    match &value.next {
        Some(item_3) => {
            out.unsigned((item_3).as_nanos());
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_proposal_key(
    value: &jig_client::contract::ProposalKey,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"proposer");
    super::encode_proposer(&value.proposer, out, budget);
    out.key(b"number");
    out.unsigned(value.number);
    out.object_end();
}
