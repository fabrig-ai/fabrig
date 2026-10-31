//! Integer JSON for supplied read facts.
use skein_json::writer::Encoder;
pub(super) fn encode_activity(value: &jig_client::contract::ActivityLine, out: &mut Encoder, budget: &mut super::Cut) {
    encode_activity_owner(&value.source, value.decoded.as_ref(), out, budget);
}

pub(super) fn encode_activity_line(
    value: &jig_client::contract::ActivityLine,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"position");
    out.unsigned(value.source.position);
    out.key(b"at");
    out.unsigned(value.source.at.as_nanos());
    out.key(b"event");
    encode_activity(value, out, budget);
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
