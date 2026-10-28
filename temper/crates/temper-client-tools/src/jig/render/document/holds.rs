//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_funder(value: &jig_client::contract::Funder, out: &mut Encoder, _budget: &mut super::Cut) {
    match value {
        jig_client::contract::Funder::Task(field_0) => {
            out.object_start();
            out.key(b"task");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::Funder::Pool { project, person, period } => {
            out.object_start();
            out.key(b"pool");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(*(project)));
            out.key(b"person");
            out.unsigned(*(person));
            out.key(b"period");
            out.unsigned(*(period));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Funder::Recurring { project, task, period } => {
            out.object_start();
            out.key(b"recurring");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(*(project)));
            out.key(b"task");
            out.unsigned(*(task));
            out.key(b"period");
            out.unsigned(*(period));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Funder::Period { project, period } => {
            out.object_start();
            out.key(b"period");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(*(project)));
            out.key(b"period");
            out.unsigned(*(period));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_grant(value: &jig_client::contract::Grant, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"kind");
    out.unsigned(u64::from(value.kind));
    out.key(b"pattern");
    super::encode_owner_pattern(&value.pattern, out, budget);
    out.object_end();
}

pub(super) fn encode_held_role(value: jig_client::contract::HeldRole, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::HeldRole::Owner => out.string(b"owner"),
        jig_client::contract::HeldRole::Maintainer => out.string(b"maintainer"),
        jig_client::contract::HeldRole::Member => out.string(b"member"),
        jig_client::contract::HeldRole::Observer => out.string(b"observer"),
        jig_client::contract::HeldRole::Policy { role } => {
            out.object_start();
            out.key(b"policy");
            out.object_start();
            out.key(b"role");
            out.unsigned(u64::from(*(role)));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_held_runtime(
    value: &jig_client::contract::HeldRuntime,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::HeldRuntime::Waiting => out.string(b"waiting"),
        jig_client::contract::HeldRuntime::Active(field_0) => {
            out.object_start();
            out.key(b"active");
            super::encode_active_run(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::HeldRuntime::Closing(field_0) => {
            out.object_start();
            out.key(b"closing");
            super::encode_closing(field_0, out, budget);
            out.object_end();
        }
    }
}

pub(super) fn encode_hold(value: &jig_client::contract::Hold, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"revision");
    super::encode_revision(value.revision, out, budget);
    out.key(b"reason");
    super::encode_hold_reason(&value.reason, out, budget);
    out.key(b"before");
    super::encode_phase(value.before, out, budget);
    out.key(b"since");
    out.unsigned(value.since.as_nanos());
    out.key(b"result");
    match &value.result {
        Some(item_1) => {
            super::encode_result_value(item_1, out, budget);
        }
        None => out.null(),
    }
    out.key(b"spend");
    out.array_start();
    for item_2 in &value.spend {
        super::encode_spend_line(item_2, out, budget);
    }
    out.array_end();
    out.object_end();
}
