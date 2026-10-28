//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_owner_grant(value: &jig_client::contract::OwnerGrant, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"kind");
    out.unsigned(u64::from(value.kind));
    out.key(b"pattern");
    encode_owner_pattern(&value.pattern, out, budget);
    out.object_end();
}

pub(super) fn encode_owner_hold_escalation(
    value: &jig_client::contract::OwnerHoldEscalation,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerHoldEscalation::Routing => out.string(b"routing"),
        jig_client::contract::OwnerHoldEscalation::Waiting { holder, entry, since } => {
            out.object_start();
            out.key(b"waiting");
            out.object_start();
            out.key(b"holder");
            super::encode_owner_escalation_holder(holder, out, budget);
            out.key(b"entry");
            out.unsigned(*(entry));
            out.key(b"since");
            out.unsigned((since).as_nanos());
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerHoldEscalation::Rejected { by, reason } => {
            out.object_start();
            out.key(b"rejected");
            out.object_start();
            out.key(b"by");
            out.unsigned(*(by));
            out.key(b"reason");
            super::encode_document_text(reason, out, budget);
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_kind_implication(
    value: jig_client::contract::OwnerKindImplication,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"kind");
    out.unsigned(u64::from(value.kind));
    out.key(b"implies");
    out.unsigned(u64::from(value.implies));
    out.object_end();
}

pub(super) fn encode_owner_numbers(
    value: &jig_client::contract::OwnerNumbers,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"budget");
    out.unsigned(value.budget);
    out.key(b"spent");
    out.unsigned(value.spent);
    out.key(b"spent_below");
    out.unsigned(value.spent_below);
    out.key(b"reserved");
    out.unsigned(value.reserved);
    out.object_end();
}

pub(super) fn encode_owner_part(value: &jig_client::contract::OwnerPart, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"kind");
    encode_owner_part_kind(value.kind, out, budget);
    out.key(b"key");
    encode_owner_part_key(&value.key, out, budget);
    out.object_end();
}

pub(super) fn encode_owner_part_key(
    value: &jig_client::contract::OwnerPartKey,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerPartKey::Path(field_0) => {
            out.object_start();
            out.key(b"path");
            out.array_start();
            for item_1 in field_0 {
                super::hex(item_1, out);
            }
            out.array_end();
            out.object_end();
        }
        jig_client::contract::OwnerPartKey::Number(field_0) => {
            out.object_start();
            out.key(b"number");
            out.unsigned(*(field_0));
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_part_kind(
    value: jig_client::contract::OwnerPartKind,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::OwnerPartKind::Resource => out.string(b"resource"),
        jig_client::contract::OwnerPartKind::Pool => out.string(b"pool"),
        jig_client::contract::OwnerPartKind::Result => out.string(b"result"),
        jig_client::contract::OwnerPartKind::Procedure => out.string(b"procedure"),
        jig_client::contract::OwnerPartKind::Effect => out.string(b"effect"),
        jig_client::contract::OwnerPartKind::Outbox => out.string(b"outbox"),
        jig_client::contract::OwnerPartKind::Hold => out.string(b"hold"),
    }
}

pub(super) fn encode_owner_pattern(
    value: &jig_client::contract::OwnerPattern,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"segments");
    out.array_start();
    for item_1 in &value.segments {
        super::hex(item_1, out);
    }
    out.array_end();
    out.key(b"last");
    encode_owner_pattern_end(&value.last, out, budget);
    out.object_end();
}

pub(super) fn encode_owner_pattern_end(
    value: &jig_client::contract::OwnerPatternEnd,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerPatternEnd::Exact(field_0) => {
            out.object_start();
            out.key(b"exact");
            super::hex(field_0, out);
            out.object_end();
        }
        jig_client::contract::OwnerPatternEnd::Open(field_0) => {
            out.object_start();
            out.key(b"open");
            super::hex(field_0, out);
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_person_stall_notice(
    value: &jig_client::contract::OwnerPersonStallNotice,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"message");
    out.unsigned(value.message);
    out.key(b"at");
    out.unsigned(value.at.as_nanos());
    out.key(b"activity");
    out.unsigned(value.activity);
    out.object_end();
}
