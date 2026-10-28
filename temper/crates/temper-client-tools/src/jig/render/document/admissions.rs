//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_act_kind(value: jig_client::contract::ActKind, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::ActKind::Message => out.string(b"message"),
        jig_client::contract::ActKind::Answer => out.string(b"answer"),
        jig_client::contract::ActKind::Take => out.string(b"take"),
        jig_client::contract::ActKind::HandBack => out.string(b"hand_back"),
        jig_client::contract::ActKind::Accept => out.string(b"accept"),
        jig_client::contract::ActKind::Reject => out.string(b"reject"),
        jig_client::contract::ActKind::PassUp => out.string(b"pass_up"),
        jig_client::contract::ActKind::Release => out.string(b"release"),
        jig_client::contract::ActKind::LeaveHeld => out.string(b"leave_held"),
        jig_client::contract::ActKind::Amend => out.string(b"amend"),
        jig_client::contract::ActKind::Stop => out.string(b"stop"),
        jig_client::contract::ActKind::Cancel => out.string(b"cancel"),
        jig_client::contract::ActKind::Move => out.string(b"move"),
        jig_client::contract::ActKind::Reorder => out.string(b"reorder"),
        jig_client::contract::ActKind::Seen => out.string(b"seen"),
        jig_client::contract::ActKind::EditNote => out.string(b"edit_note"),
        jig_client::contract::ActKind::DeleteNote => out.string(b"delete_note"),
        jig_client::contract::ActKind::Adopt => out.string(b"adopt"),
        jig_client::contract::ActKind::ChangePolicy => out.string(b"change_policy"),
        jig_client::contract::ActKind::SetRoles => out.string(b"set_roles"),
        jig_client::contract::ActKind::MakeService => out.string(b"make_service"),
        jig_client::contract::ActKind::SetPool => out.string(b"set_pool"),
        jig_client::contract::ActKind::Rename => out.string(b"rename"),
        jig_client::contract::ActKind::Withdraw => out.string(b"withdraw"),
    }
}

pub(super) fn encode_active_run(value: &jig_client::contract::ActiveRun, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"stage");
    super::encode_run_stage(&value.stage, out, budget);
    out.key(b"phase_since");
    out.unsigned(value.phase_since.as_nanos());
    out.key(b"attempt");
    match &value.attempt {
        Some(item_1) => {
            out.unsigned(*(item_1));
        }
        None => out.null(),
    }
    out.key(b"placement");
    match &value.placement {
        Some(item_2) => {
            out.unsigned((item_2).as_nanos());
        }
        None => out.null(),
    }
    out.key(b"retry");
    match &value.retry {
        Some(item_3) => {
            super::encode_retry_notice(item_3, out, budget);
        }
        None => out.null(),
    }
    out.key(b"observation");
    match &value.observation {
        Some(item_4) => {
            super::encode_run_observation(item_4, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_actor(value: &jig_client::contract::Actor, out: &mut Encoder, _budget: &mut super::Cut) {
    match value {
        jig_client::contract::Actor::Party(field_0) => {
            out.object_start();
            out.key(b"party");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::Actor::Task(field_0) => {
            out.object_start();
            out.key(b"task");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::Actor::Deployment => out.string(b"deployment"),
        jig_client::contract::Actor::ProjectDeployment { project } => {
            out.object_start();
            out.key(b"project_deployment");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(*(project)));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_admission(value: &jig_client::contract::Admission, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::Admission::Allowed => out.string(b"allowed"),
        jig_client::contract::Admission::Proposed { to } => {
            out.object_start();
            out.key(b"proposed");
            out.object_start();
            out.key(b"to");
            super::encode_recipient(to, out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Admission::Bounded { within, beyond } => {
            out.object_start();
            out.key(b"bounded");
            out.object_start();
            out.key(b"within");
            super::encode_bounds(within, out, budget);
            out.key(b"beyond");
            super::encode_beyond(beyond, out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Admission::Refused(field_0) => {
            out.object_start();
            out.key(b"refused");
            super::encode_refusal(field_0, out, budget);
            out.object_end();
        }
    }
}
