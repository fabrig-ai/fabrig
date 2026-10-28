//! Integer JSON for supplied read facts.
use skein_json::writer::Encoder;
pub(super) fn encode_object_key(value: &jig_client::contract::ObjectKey, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::ObjectKey::Task(field_0) => {
            out.object_start();
            out.key(b"task");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::ObjectKey::Proposal(field_0) => {
            out.object_start();
            out.key(b"proposal");
            super::encode_proposal_key(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::ObjectKey::Escalation(field_0) => {
            out.object_start();
            out.key(b"escalation");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::ObjectKey::Question { task, message } => {
            out.object_start();
            out.key(b"question");
            out.object_start();
            out.key(b"task");
            out.unsigned(*(task));
            out.key(b"message");
            out.unsigned(*(message));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::ObjectKey::PersonTask(field_0) => {
            out.object_start();
            out.key(b"person_task");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::ObjectKey::Result(field_0) => {
            out.object_start();
            out.key(b"result");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::ObjectKey::Note { project, note } => {
            out.object_start();
            out.key(b"note");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(*(project)));
            out.key(b"note");
            out.unsigned(*(note));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_tree(value: &jig_client::contract::OwnerTree, out: &mut Encoder, budget: &mut super::Cut) {
    let digest = budget.digest.take();
    out.object_start();
    out.key(b"root");
    super::encode_chip(&value.root, out, budget);
    out.key(b"rows");
    out.array_start();
    for item_1 in &value.rows {
        encode_owner_tree_row(item_1, out, budget);
    }
    out.array_end();
    out.key(b"folded");
    out.array_start();
    for item_2 in &value.folded {
        encode_owner_tree_fold(item_2, out, budget);
    }
    out.array_end();
    out.key(b"held_below");
    out.unsigned(u64::from(value.held_below));
    if budget.subject {
        out.key(b"more");
        out.boolean(value.next.is_some());
    } else {
        out.key(b"next");
        match &value.next {
            Some(item_3) => super::encode_cursor(item_3, out, budget),
            None => out.null(),
        }
    }
    super::subject_end(digest, out);
}

pub(super) fn encode_owner_tree_fold(
    value: &jig_client::contract::OwnerTreeFold,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"parent");
    out.unsigned(value.parent);
    out.key(b"phases");
    super::encode_owner_phase_counts(&value.phases, out, budget);
    out.key(b"held_below");
    out.unsigned(u64::from(value.held_below));
    out.object_end();
}

pub(super) fn encode_owner_tree_row(
    value: &jig_client::contract::OwnerTreeRow,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"chip");
    super::encode_chip(&value.chip, out, budget);
    out.key(b"parent");
    out.unsigned(value.parent);
    out.key(b"depth");
    out.unsigned(u64::from(value.depth));
    out.key(b"proposals");
    out.array_start();
    for item_1 in &value.proposals {
        super::encode_proposal_key(item_1, out, budget);
    }
    out.array_end();
    out.key(b"spent");
    out.unsigned(value.spent);
    out.key(b"dependencies");
    out.array_start();
    for item_2 in &value.dependencies {
        super::encode_owner_task_link(item_2, out, budget);
    }
    out.array_end();
    out.key(b"cut");
    out.boolean(value.cut);
    out.object_end();
}

pub(super) fn encode_run_beginning(
    value: jig_client::contract::RunBeginning,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::RunBeginning::Resumed => out.string(b"resumed"),
        jig_client::contract::RunBeginning::Fresh { tail } => {
            out.object_start();
            out.key(b"fresh");
            out.object_start();
            out.key(b"tail");
            out.unsigned(u64::from(*(tail)));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_run_ending(value: &jig_client::contract::RunEnding, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::RunEnding::Parked => out.string(b"parked"),
        jig_client::contract::RunEnding::Finished => out.string(b"finished"),
        jig_client::contract::RunEnding::Cancelled => out.string(b"cancelled"),
        jig_client::contract::RunEnding::Failed { class, reason } => {
            out.object_start();
            out.key(b"failed");
            out.object_start();
            out.key(b"class");
            super::encode_failure_class(*(class), out, budget);
            out.key(b"reason");
            budget.words((reason).bytes(), out);
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_run_state(value: &jig_client::contract::RunState, out: &mut Encoder, budget: &mut super::Cut) {
    let digest = budget.digest.take();
    out.object_start();
    out.key(b"task");
    out.unsigned(value.task);
    out.key(b"attempt");
    match &value.attempt {
        Some(item_1) => {
            out.unsigned(*(item_1));
        }
        None => out.null(),
    }
    out.key(b"committed_turn");
    out.unsigned(u64::from(value.committed_turn));
    out.key(b"writing");
    match &value.writing {
        Some(item_2) => {
            encode_run_text(item_2, out, budget);
        }
        None => out.null(),
    }
    out.key(b"call");
    match &value.call {
        Some(item_3) => {
            super::encode_current_call(item_3, out, budget);
        }
        None => out.null(),
    }
    super::subject_end(digest, out);
}

pub(super) fn encode_run_text(value: &jig_client::contract::RunText, out: &mut Encoder, budget: &mut super::Cut) {
    let digest = budget.digest.take();
    out.object_start();
    out.key(b"attempt");
    out.unsigned(value.attempt);
    out.key(b"began");
    match &value.began {
        Some(item_1) => {
            out.unsigned((item_1).as_nanos());
        }
        None => out.null(),
    }
    out.key(b"turn");
    super::encode_provisional_turn_name(&value.turn, out, budget);
    out.key(b"from");
    out.unsigned(u64::from(value.from));
    out.key(b"through");
    out.unsigned(u64::from(value.through));
    out.key(b"text");
    budget.words(value.text.bytes(), out);
    out.key(b"cut");
    out.boolean(value.cut);
    out.key(b"call");
    match &value.call {
        Some(item_2) => {
            super::encode_live_call(item_2, out, budget);
        }
        None => out.null(),
    }
    super::subject_end(digest, out);
}

pub(super) fn encode_tree(value: &jig_client::contract::Tree, out: &mut Encoder, budget: &mut super::Cut) {
    let digest = budget.digest.take();
    out.object_start();
    out.key(b"task");
    out.unsigned(value.task);
    out.key(b"nodes");
    out.array_start();
    for item_1 in &value.nodes {
        encode_tree_node(item_1, out, budget);
    }
    out.array_end();
    out.key(b"folded");
    super::encode_phase_counts(&value.folded, out, budget);
    out.key(b"held_below");
    out.unsigned(u64::from(value.held_below));
    super::subject_end(digest, out);
}

pub(super) fn encode_tree_node(value: &jig_client::contract::TreeNode, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"chip");
    super::encode_chip(&value.chip, out, budget);
    out.key(b"parent");
    match &value.parent {
        Some(item_1) => {
            out.unsigned(*(item_1));
        }
        None => out.null(),
    }
    out.key(b"dependencies");
    out.array_start();
    for item_2 in &value.dependencies {
        out.unsigned(*(item_2));
    }
    out.array_end();
    out.key(b"proposals");
    out.array_start();
    for item_3 in &value.proposals {
        super::encode_proposal_key(item_3, out, budget);
    }
    out.array_end();
    out.key(b"spend");
    out.unsigned(value.spend);
    out.key(b"folded");
    super::encode_phase_counts(&value.folded, out, budget);
    out.key(b"held_below");
    out.unsigned(u64::from(value.held_below));
    out.object_end();
}
