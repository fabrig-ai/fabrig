//! Integer JSON for supplied read facts.
use skein_json::writer::Encoder;
pub(super) fn encode_conversation_read(
    value: &jig_client::contract::ConversationRead,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"seen");
    out.unsigned(value.seen);
    out.key(b"unread");
    super::encode_count(value.unread, out, budget);
    out.object_end();
}

pub(super) fn encode_cursor(value: &jig_client::contract::Cursor, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"value");
    super::hex(value.value.bytes(), out);
    out.object_end();
}

pub(super) fn encode_ended_group(value: &jig_client::contract::EndedGroup, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::EndedGroup::Rows { rows, after, cut } => {
            out.object_start();
            out.key(b"rows");
            out.object_start();
            out.key(b"rows");
            out.array_start();
            for item_1 in rows {
                encode_task_line(item_1, out, budget);
            }
            out.array_end();
            out.key(b"after");
            match after {
                Some(item_2) => {
                    encode_cursor(item_2, out, budget);
                }
                None => out.null(),
            }
            out.key(b"cut");
            out.boolean(*(cut));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::EndedGroup::None => out.string(b"none"),
    }
}

pub(super) fn encode_owner_phase_counts(
    value: &jig_client::contract::OwnerPhaseCounts,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"waiting");
    out.unsigned(u64::from(value.waiting));
    out.key(b"idle");
    out.unsigned(u64::from(value.idle));
    out.key(b"starting");
    out.unsigned(u64::from(value.starting));
    out.key(b"running");
    out.unsigned(u64::from(value.running));
    out.key(b"backing_off");
    out.unsigned(u64::from(value.backing_off));
    out.key(b"closing");
    out.unsigned(u64::from(value.closing));
    out.key(b"held");
    out.unsigned(u64::from(value.held));
    out.key(b"done");
    out.unsigned(u64::from(value.done));
    out.key(b"failed");
    out.unsigned(u64::from(value.failed));
    out.key(b"cancelled");
    out.unsigned(u64::from(value.cancelled));
    out.object_end();
}

pub(super) fn encode_task_line(value: &jig_client::contract::TaskLine, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"chip");
    super::encode_chip(&value.chip, out, budget);
    out.key(b"project");
    out.unsigned(u64::from(value.project));
    out.key(b"requester");
    super::encode_owner_requester(&value.requester, out, budget);
    out.key(b"tracked");
    match &value.tracked {
        Some(item_1) => {
            out.unsigned(u64::from(*(item_1)));
        }
        None => out.null(),
    }
    out.key(b"label");
    out.unsigned(u64::from(value.label));
    out.key(b"flags");
    super::encode_task_flags(value.flags, out, budget);
    out.key(b"changed");
    out.unsigned(value.changed.as_nanos());
    out.key(b"last_activity");
    out.unsigned(value.last_activity.as_nanos());
    out.key(b"ended_at");
    match &value.ended_at {
        Some(item_2) => {
            out.unsigned((item_2).as_nanos());
        }
        None => out.null(),
    }
    out.key(b"budget");
    super::encode_owner_budget(&value.budget, out, budget);
    out.key(b"waits_on");
    super::encode_waits_on(&value.waits_on, out, budget);
    out.key(b"phases");
    encode_owner_phase_counts(&value.phases, out, budget);
    out.key(b"held_below");
    out.unsigned(u64::from(value.held_below));
    out.key(b"conversation");
    match &value.conversation {
        Some(item_3) => {
            encode_conversation_read(item_3, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_task_page(
    value: &jig_client::contract::TaskPage,
    ended: bool,
    read_on: Option<&[u8]>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"live");
    out.array_start();
    if !ended {
        for item_1 in &value.live {
            encode_task_line(item_1, out, budget);
        }
    }
    out.array_end();
    out.key(b"live_after");
    match if ended { None } else { value.live_after.as_ref() } {
        Some(item_2) => {
            encode_cursor(item_2, out, budget);
        }
        None => out.null(),
    }
    out.key(b"ended");
    encode_ended_group(if ended { &value.ended } else { &jig_client::contract::EndedGroup::None }, out, budget);
    out.key(b"read_on");
    match read_on {
        Some(call) => out.string(call),
        None => out.null(),
    }
    out.object_end();
}
