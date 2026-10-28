//! Integer JSON for supplied read facts.
use skein_json::writer::Encoder;
pub(super) fn encode_inbox_entry(value: &jig_client::contract::InboxEntry, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"reply");
    match &value.reply {
        Some(item_1) => {
            super::encode_task_line(item_1, out, budget);
        }
        None => out.null(),
    }
    out.key(b"lineage");
    out.array_start();
    for item_2 in &value.lineage {
        super::encode_chip(item_2, out, budget);
    }
    out.array_end();
    out.key(b"goal");
    match &value.goal {
        Some(item_3) => {
            out.unsigned(*(item_3));
        }
        None => out.null(),
    }
    out.key(b"recipient");
    match &value.recipient {
        Some(item_4) => {
            super::encode_recipient(item_4, out, budget);
        }
        None => out.null(),
    }
    out.key(b"stall");
    match &value.stall {
        Some(item_5) => {
            out.unsigned((item_5).as_nanos());
        }
        None => out.null(),
    }
    out.key(b"object");
    super::encode_object_key(&value.object, out, budget);
    out.key(b"project");
    out.unsigned(u64::from(value.project));
    out.key(b"group");
    encode_inbox_group(value.group, out, budget);
    out.key(b"position");
    out.unsigned(value.position);
    out.key(b"at");
    out.unsigned(value.at.as_nanos());
    out.object_end();
}

pub(super) fn encode_inbox_group(value: jig_client::contract::InboxGroup, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::InboxGroup::Questions => out.string(b"questions"),
        jig_client::contract::InboxGroup::Proposals => out.string(b"proposals"),
        jig_client::contract::InboxGroup::Escalations => out.string(b"escalations"),
        jig_client::contract::InboxGroup::PersonTasks => out.string(b"person_tasks"),
        jig_client::contract::InboxGroup::Results => out.string(b"results"),
        jig_client::contract::InboxGroup::Replies => out.string(b"replies"),
    }
}

pub(super) fn encode_inbox_page(value: &jig_client::contract::InboxPage, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"cut");
    out.boolean(value.cut);
    out.key(b"rows");
    out.array_start();
    for item_1 in &value.rows {
        encode_inbox_entry(item_1, out, budget);
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
    out.object_end();
}

pub(super) fn encode_owner_inbox_cursor(
    value: &jig_client::contract::OwnerInboxCursor,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerInboxCursor::Number { number } => {
            out.object_start();
            out.key(b"number");
            out.object_start();
            out.key(b"number");
            out.unsigned(*(number));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerInboxCursor::Inbox { task, object, at } => {
            out.object_start();
            out.key(b"inbox");
            out.object_start();
            out.key(b"task");
            out.unsigned(*(task));
            out.key(b"object");
            encode_owner_inbox_object(object, out, budget);
            out.key(b"at");
            match at {
                Some(item_1) => {
                    out.unsigned((item_1).as_nanos());
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_inbox_entry(
    value: &jig_client::contract::OwnerInboxEntry,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"reply");
    match &value.reply {
        Some(item_1) => {
            super::encode_task_line(item_1, out, budget);
        }
        None => out.null(),
    }
    out.key(b"task");
    super::encode_owner_task_link(&value.task, out, budget);
    out.key(b"project");
    match &value.project {
        Some(item_2) => {
            out.unsigned(u64::from(*(item_2)));
        }
        None => out.null(),
    }
    out.key(b"object");
    encode_owner_inbox_object(&value.object, out, budget);
    out.key(b"activity");
    match &value.activity {
        Some(item_3) => {
            out.unsigned(*(item_3));
        }
        None => out.null(),
    }
    out.key(b"at");
    match &value.at {
        Some(item_4) => {
            out.unsigned((item_4).as_nanos());
        }
        None => out.null(),
    }
    out.key(b"unread");
    match &value.unread {
        Some(item_5) => {
            out.boolean(*(item_5));
        }
        None => out.null(),
    }
    if !budget.subject {
        out.key(b"cursor");
        encode_owner_inbox_cursor(&value.cursor, out, budget);
    }
    out.object_end();
}

pub(super) fn encode_owner_inbox_object(
    value: &jig_client::contract::OwnerInboxObject,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerInboxObject::Question { message } => {
            out.object_start();
            out.key(b"question");
            out.object_start();
            out.key(b"message");
            out.unsigned(*(message));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerInboxObject::Proposal { proposer, number } => {
            out.object_start();
            out.key(b"proposal");
            out.object_start();
            out.key(b"proposer");
            super::encode_proposer(proposer, out, budget);
            out.key(b"number");
            out.unsigned(*(number));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerInboxObject::Escalation { revision } => {
            out.object_start();
            out.key(b"escalation");
            out.object_start();
            out.key(b"revision");
            out.unsigned(*(revision));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerInboxObject::PersonTask => out.string(b"person_task"),
        jig_client::contract::OwnerInboxObject::Reply { message } => {
            out.object_start();
            out.key(b"reply");
            out.object_start();
            out.key(b"message");
            out.unsigned(*(message));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerInboxObject::Result { position } => {
            out.object_start();
            out.key(b"result");
            out.object_start();
            out.key(b"position");
            out.unsigned(*(position));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_inbox_page(
    value: &jig_client::contract::OwnerInboxPage,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"selection");
    match &value.selection {
        Some(item_1) => {
            out.array_start();
            for item_2 in item_1 {
                encode_owner_inbox_selection(item_2, out, budget);
            }
            out.array_end();
        }
        None => out.null(),
    }
    out.key(b"cut");
    out.boolean(value.cut);
    out.key(b"rows");
    out.array_start();
    for item_3 in &value.rows {
        encode_owner_inbox_entry(item_3, out, budget);
    }
    out.array_end();
    out.key(b"before");
    match &value.before {
        Some(item_4) => {
            super::encode_cursor(item_4, out, budget);
        }
        None => out.null(),
    }
    out.key(b"after");
    match &value.after {
        Some(item_5) => {
            super::encode_cursor(item_5, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_owner_inbox_selection(
    value: &jig_client::contract::OwnerInboxSelection,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    if !budget.subject {
        out.key(b"cursor");
        encode_owner_inbox_cursor(&value.cursor, out, budget);
    }
    out.key(b"goals");
    out.array_start();
    for item_1 in &value.goals {
        out.unsigned(*(item_1));
    }
    out.array_end();
    out.object_end();
}
