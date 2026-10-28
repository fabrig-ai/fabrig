//! Integer JSON for supplied read facts.
use skein_json::writer::Encoder;
pub(super) fn encode_block(value: &jig_client::contract::Block, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::Block::Text(field_0) => {
            out.object_start();
            out.key(b"text");
            budget.part((field_0).bytes(), out);
            out.object_end();
        }
        jig_client::contract::Block::Opaque { dialect, size } => {
            out.object_start();
            out.key(b"opaque");
            out.object_start();
            out.key(b"dialect");
            budget.words((dialect).bytes(), out);
            out.key(b"size");
            out.unsigned(*(size));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Block::Call(field_0) => {
            out.object_start();
            out.key(b"call");
            encode_call(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::Block::Result { call, summary, failed } => {
            out.object_start();
            out.key(b"result");
            out.object_start();
            out.key(b"call");
            out.unsigned(*(call));
            out.key(b"summary");
            budget.part((summary).bytes(), out);
            out.key(b"failed");
            out.boolean(*(failed));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_call(value: &jig_client::contract::Call, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"number");
    out.unsigned(value.number);
    out.key(b"tool");
    out.string(value.tool.bytes());
    out.key(b"input");
    budget.part(value.input.bytes(), out);
    out.key(b"outcome");
    match &value.outcome {
        Some(item_1) => {
            budget.part((item_1).bytes(), out);
        }
        None => out.null(),
    }
    out.key(b"failed");
    out.boolean(value.failed);
    out.key(b"objects");
    out.array_start();
    for item_2 in &value.objects {
        super::encode_object_key(item_2, out, budget);
    }
    out.array_end();
    out.key(b"parts");
    out.array_start();
    for item_3 in &value.parts {
        super::encode_part(item_3, out, budget);
    }
    out.array_end();
    out.object_end();
}

pub(super) fn encode_current_call(
    value: &jig_client::contract::CurrentCall,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"identity");
    super::encode_call_identity(&value.identity, out, budget);
    out.key(b"tool");
    out.string(value.tool.bytes());
    out.key(b"input");
    budget.words(value.input.bytes(), out);
    out.key(b"cut");
    out.boolean(value.cut);
    out.key(b"began");
    match &value.began {
        Some(item_1) => {
            out.unsigned((item_1).as_nanos());
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_live_call(value: &jig_client::contract::LiveCall, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::LiveCall::Decoded(field_0) => {
            out.object_start();
            out.key(b"decoded");
            encode_call(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::LiveCall::Raw(field_0) => {
            out.object_start();
            out.key(b"raw");
            encode_current_call(field_0, out, budget);
            out.object_end();
        }
    }
}

pub(super) fn encode_message(value: &jig_client::contract::Message, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"speaker");
    encode_speaker(value.speaker, out, budget);
    out.key(b"blocks");
    out.array_start();
    for item_1 in &value.blocks {
        encode_block(item_1, out, budget);
    }
    out.array_end();
    out.object_end();
}

pub(super) fn encode_provisional_turn_name(
    value: &jig_client::contract::ProvisionalTurnName,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"attempt");
    out.unsigned(value.attempt);
    out.key(b"number");
    out.unsigned(u64::from(value.number));
    out.key(b"retry");
    out.unsigned(u64::from(value.retry));
    out.object_end();
}

pub(super) fn encode_speaker(value: jig_client::contract::Speaker, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::Speaker::Person => out.string(b"person"),
        jig_client::contract::Speaker::Assistant => out.string(b"assistant"),
        jig_client::contract::Speaker::Tool => out.string(b"tool"),
        jig_client::contract::Speaker::System => out.string(b"system"),
    }
}

pub(super) fn encode_turn(value: &jig_client::contract::Turn, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"messages");
    out.array_start();
    for item_1 in &value.messages {
        encode_message(item_1, out, budget);
    }
    out.array_end();
    out.key(b"used");
    encode_usage(&value.used, out, budget);
    out.key(b"spent");
    out.unsigned(value.spent);
    out.key(b"read_through");
    match &value.read_through {
        Some(item_2) => {
            out.unsigned(*(item_2));
        }
        None => out.null(),
    }
    out.key(b"cut");
    out.boolean(value.cut);
    out.object_end();
}

pub(super) fn encode_turn_name(value: &jig_client::contract::TurnName, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"attempt");
    out.unsigned(value.attempt);
    out.key(b"number");
    out.unsigned(u64::from(value.number));
    out.object_end();
}

pub(super) fn encode_turn_problem(
    value: &jig_client::contract::TurnProblem,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::TurnProblem::Version { found } => {
            out.object_start();
            out.key(b"version");
            out.object_start();
            out.key(b"found");
            out.unsigned(u64::from(*(found)));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::TurnProblem::Malformed => out.string(b"malformed"),
        jig_client::contract::TurnProblem::TooLarge { bytes } => {
            out.object_start();
            out.key(b"too_large");
            out.object_start();
            out.key(b"bytes");
            out.unsigned(*(bytes));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_turn_read(value: &jig_client::contract::TurnRead, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::TurnRead::Decoded(field_0) => {
            out.object_start();
            out.key(b"decoded");
            encode_turn(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::TurnRead::Undecodable { why } => {
            out.object_start();
            out.key(b"undecodable");
            out.object_start();
            out.key(b"why");
            encode_turn_problem(why, out, budget);
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_usage(value: &jig_client::contract::Usage, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"input");
    out.unsigned(value.input);
    out.key(b"output");
    out.unsigned(value.output);
    out.key(b"cached");
    out.unsigned(value.cached);
    out.key(b"cache_written");
    out.unsigned(value.cache_written);
    out.object_end();
}
