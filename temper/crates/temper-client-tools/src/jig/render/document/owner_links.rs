//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_owner_person_wait(
    value: &jig_client::contract::OwnerPersonWait,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"since");
    out.unsigned(value.since.as_nanos());
    out.key(b"stall");
    out.unsigned(value.stall.as_nanos());
    out.key(b"deadline");
    out.unsigned(value.deadline.as_nanos());
    out.key(b"notice");
    match &value.notice {
        Some(item_1) => {
            super::encode_owner_person_stall_notice(item_1, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_owner_prior_phase(
    value: &jig_client::contract::OwnerPriorPhase,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerPriorPhase::Waiting => out.string(b"waiting"),
        jig_client::contract::OwnerPriorPhase::Active(field_0) => {
            out.object_start();
            out.key(b"active");
            super::encode_run_stage(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::OwnerPriorPhase::Closing(field_0) => {
            out.object_start();
            out.key(b"closing");
            super::encode_closing_stage(field_0, out, budget);
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_requester(
    value: &jig_client::contract::OwnerRequester,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerRequester::Task(field_0) => {
            out.object_start();
            out.key(b"task");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::OwnerRequester::Person(field_0) => {
            out.object_start();
            out.key(b"person");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::OwnerRequester::Deployment { project } => {
            out.object_start();
            out.key(b"deployment");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(*(project)));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_resource_line(
    value: &jig_client::contract::OwnerResourceLine,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"part");
    super::encode_owner_part(&value.part, out, budget);
    out.key(b"kind");
    out.unsigned(u64::from(value.kind));
    out.key(b"taken");
    out.boolean(value.taken);
    out.key(b"holder");
    match &value.holder {
        Some(item_1) => {
            out.unsigned(*(item_1));
        }
        None => out.null(),
    }
    out.key(b"writer");
    match &value.writer {
        Some(item_2) => {
            encode_owner_resource_writer(item_2, out, budget);
        }
        None => out.null(),
    }
    out.key(b"writer_lost");
    out.boolean(value.writer_lost);
    out.object_end();
}

pub(super) fn encode_owner_resource_scope(
    value: &jig_client::contract::OwnerResourceScope,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(value.connector));
    out.key(b"pattern");
    super::encode_owner_pattern(&value.pattern, out, budget);
    out.object_end();
}

pub(super) fn encode_owner_resource_writer(
    value: &jig_client::contract::OwnerResourceWriter,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerResourceWriter::Run { task, attempt } => {
            out.object_start();
            out.key(b"run");
            out.object_start();
            out.key(b"task");
            out.unsigned(*(task));
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerResourceWriter::Effect { entry } => {
            out.object_start();
            out.key(b"effect");
            out.object_start();
            out.key(b"entry");
            out.unsigned(*(entry));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_task_link(
    value: &jig_client::contract::OwnerTaskLink,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerTaskLink::Live(field_0) => {
            out.object_start();
            out.key(b"live");
            super::encode_chip(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::OwnerTaskLink::Retained { chip, at } => {
            out.object_start();
            out.key(b"retained");
            out.object_start();
            out.key(b"chip");
            super::encode_chip(chip, out, budget);
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
        jig_client::contract::OwnerTaskLink::Stored { number, status } => {
            out.object_start();
            out.key(b"stored");
            out.object_start();
            out.key(b"number");
            out.unsigned(*(number));
            out.key(b"status");
            match status {
                Some(item_2) => {
                    encode_owner_task_status(*(item_2), out, budget);
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_task_status(
    value: jig_client::contract::OwnerTaskStatus,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::OwnerTaskStatus::Done => out.string(b"done"),
        jig_client::contract::OwnerTaskStatus::Failed => out.string(b"failed"),
        jig_client::contract::OwnerTaskStatus::Cancelled => out.string(b"cancelled"),
    }
}
