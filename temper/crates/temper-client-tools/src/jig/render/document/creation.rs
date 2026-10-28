//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_count(value: jig_client::contract::Count, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    out.object_start();
    out.key(b"value");
    out.unsigned(u64::from(value.value));
    out.key(b"limited");
    out.boolean(value.limited);
    out.object_end();
}

pub(super) fn encode_create_bounds(
    value: &jig_client::contract::CreateBounds,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"authority");
    super::encode_owner_authority_document(&value.authority, out, budget);
    out.key(b"pool_left");
    out.unsigned(value.pool_left);
    out.key(b"charters");
    out.array_start();
    for item_1 in &value.charters {
        out.unsigned(u64::from(*(item_1)));
    }
    out.array_end();
    out.key(b"procedures");
    out.array_start();
    for item_2 in &value.procedures {
        encode_create_procedure(*(item_2), out, budget);
    }
    out.array_end();
    out.key(b"latest");
    match &value.latest {
        Some(item_3) => {
            out.unsigned((item_3).as_nanos());
        }
        None => out.null(),
    }
    out.key(b"budget");
    out.unsigned(value.budget);
    out.key(b"maximum_budget");
    out.unsigned(value.maximum_budget);
    out.key(b"maximum_deadline");
    match &value.maximum_deadline {
        Some(item_4) => {
            out.unsigned((item_4).as_nanos());
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_create_procedure(
    value: jig_client::contract::CreateProcedure,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"code");
    out.unsigned(u64::from(value.code));
    out.object_end();
}

pub(super) fn encode_creating(value: &jig_client::contract::Creating, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"within");
    encode_create_bounds(&value.within, out, budget);
    out.key(b"beyond");
    super::encode_beyond(&value.beyond, out, budget);
    out.object_end();
}

pub(super) fn encode_current_hold(
    value: &jig_client::contract::CurrentHold,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::CurrentHold::Document(field_0) => {
            out.object_start();
            out.key(b"document");
            super::encode_hold_reason(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::CurrentHold::Reason(field_0) => {
            out.object_start();
            out.key(b"reason");
            super::encode_hold_category(field_0, out, budget);
            out.object_end();
        }
    }
}

pub(super) fn encode_decision(value: &jig_client::contract::Decision, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::Decision::Accepted => out.string(b"accepted"),
        jig_client::contract::Decision::Rejected { reason } => {
            out.object_start();
            out.key(b"rejected");
            out.object_start();
            out.key(b"reason");
            budget.words((reason).bytes(), out);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Decision::PassedUp => out.string(b"passed_up"),
        jig_client::contract::Decision::Released { again } => {
            out.object_start();
            out.key(b"released");
            out.object_start();
            out.key(b"again");
            match again {
                Some(item_1) => {
                    out.boolean(*(item_1));
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Decision::LeftHeld { reason } => {
            out.object_start();
            out.key(b"left_held");
            out.object_start();
            out.key(b"reason");
            budget.words((reason).bytes(), out);
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_document_specification(
    value: &jig_client::contract::DocumentSpecification,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"words");
    encode_document_text(&value.words, out, budget);
    out.key(b"parameters");
    out.array_start();
    for item_1 in &value.parameters {
        super::encode_parameter_document(item_1, out, budget);
    }
    out.array_end();
    out.key(b"inputs");
    out.array_start();
    for item_2 in &value.inputs {
        out.unsigned(*(item_2));
    }
    out.array_end();
    out.object_end();
}

pub(super) fn encode_document_text(
    value: &jig_client::contract::DocumentText,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"bytes");
    budget.words(value.bytes.bytes(), out);
    out.key(b"cut");
    out.boolean(value.cut);
    out.object_end();
}

pub(super) fn encode_effect(value: &jig_client::contract::Effect, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::Effect::Release(field_0) => {
            out.object_start();
            out.key(b"release");
            super::encode_release_effect(*(field_0), out, budget);
            out.object_end();
        }
        jig_client::contract::Effect::Cancel(field_0) => {
            out.object_start();
            out.key(b"cancel");
            super::encode_impact(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::Effect::Move { reservation } => {
            out.object_start();
            out.key(b"move");
            out.object_start();
            out.key(b"reservation");
            out.unsigned(*(reservation));
            out.object_end();
            out.object_end();
        }
    }
}
