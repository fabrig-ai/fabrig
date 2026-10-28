//! Integer JSON for supplied read facts.
use skein_json::writer::Encoder;
pub(super) fn encode_activity(value: &jig_client::contract::Activity, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::Activity::Owner { source, decoded } => {
            encode_activity_owner(source, decoded.as_ref(), out, budget);
        }
        jig_client::contract::Activity::Words { by, message, words } => {
            out.object_start();
            out.key(b"words");
            out.object_start();
            out.key(b"by");
            super::encode_party(by, out, budget);
            out.key(b"message");
            out.unsigned(*(message));
            out.key(b"words");
            budget.words((words).bytes(), out);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::RunBegan { attempt, how, why } => {
            out.object_start();
            out.key(b"run_began");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.key(b"how");
            super::encode_run_beginning(*how, out, budget);
            out.key(b"why");
            budget.words((why).bytes(), out);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::RunEnded { attempt, how, spent } => {
            out.object_start();
            out.key(b"run_ended");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.key(b"how");
            super::encode_run_ending(how, out, budget);
            out.key(b"spent");
            out.unsigned(*(spent));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Turn { name, turn } => {
            if let Some(window) = &mut budget.turn {
                window.reset(*name);
            }
            out.object_start();
            out.key(b"turn");
            out.object_start();
            out.key(b"name");
            super::encode_turn_name(name, out, budget);
            out.key(b"turn");
            super::encode_turn_read(turn, out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Result { result } => {
            out.object_start();
            out.key(b"result");
            out.object_start();
            out.key(b"result");
            super::encode_result(result, out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::News { topic, summary, part } => {
            out.object_start();
            out.key(b"news");
            out.object_start();
            out.key(b"topic");
            out.unsigned(u64::from(*(topic)));
            out.key(b"summary");
            budget.words((summary).bytes(), out);
            out.key(b"part");
            match part {
                Some(item_2) => {
                    super::encode_part(item_2, out, budget);
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Activity::Decision { object, by, how } => {
            out.object_start();
            out.key(b"decision");
            out.object_start();
            out.key(b"object");
            super::encode_object_key(object, out, budget);
            out.key(b"by");
            super::encode_actor(by, out, budget);
            out.key(b"how");
            super::encode_decision(how, out, budget);
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_activity_line(
    value: &jig_client::contract::ActivityLine,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"position");
    out.unsigned(value.position);
    out.key(b"at");
    out.unsigned(value.at.as_nanos());
    out.key(b"event");
    encode_activity(&value.event, out, budget);
    out.object_end();
}

pub(super) fn encode_activity_page(
    value: &jig_client::contract::ActivityPage,
    rows: u32,
    read_on: Option<&[u8]>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"cut");
    out.boolean(value.cut);
    out.key(b"rows");
    out.array_start();
    let first = value.rows.len().saturating_sub(rows);
    for item_1 in value.rows.as_slice().iter().skip(usize::try_from(first).expect("rows fit")) {
        encode_activity_line(item_1, out, budget);
    }
    out.array_end();
    out.key(b"before");
    match &value.before {
        Some(item_2) => {
            super::encode_cursor(item_2, out, budget);
        }
        None => out.null(),
    }
    out.key(b"after");
    match &value.after {
        Some(item_3) => {
            super::encode_cursor(item_3, out, budget);
        }
        None => out.null(),
    }
    out.key(b"read_on");
    match read_on {
        Some(call) => out.string(call),
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_archived_turn(
    value: &jig_client::contract::ArchivedTurn,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"task");
    out.unsigned(value.task);
    out.key(b"name");
    super::encode_turn_name(&value.name, out, budget);
    out.key(b"cumulative");
    out.unsigned(value.cumulative);
    out.key(b"read");
    match &value.read {
        Some(item_1) => {
            out.unsigned(*(item_1));
        }
        None => out.null(),
    }
    out.key(b"at");
    out.unsigned(value.at.as_nanos());
    out.key(b"turn");
    if let Some(window) = &mut budget.turn {
        window.reset(value.name);
    }
    super::encode_turn_read(&value.turn, out, budget);
    out.object_end();
}

fn encode_activity_owner(
    source: &jig_client::contract::OwnerActivityLine,
    decoded: Option<&jig_client::contract::ArchivedTurn>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"owner");
    out.object_start();
    out.key(b"source");
    super::encode_owner_activity_line(source, out, budget);
    out.key(b"decoded");
    match decoded {
        Some(item_1) => {
            encode_archived_turn(item_1, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}
