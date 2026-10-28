//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_owner_chip_phase(
    value: &jig_client::contract::OwnerChipPhase,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerChipPhase::Waiting { dependencies, more, holds } => {
            out.object_start();
            out.key(b"waiting");
            out.object_start();
            out.key(b"dependencies");
            out.array_start();
            for item_1 in dependencies {
                out.unsigned(*(item_1));
            }
            out.array_end();
            out.key(b"more");
            out.boolean(*(more));
            out.key(b"holds");
            out.boolean(*(holds));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerChipPhase::Active(field_0) => {
            out.object_start();
            out.key(b"active");
            super::encode_run_stage(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::OwnerChipPhase::Person { address, taken_by, since } => {
            out.object_start();
            out.key(b"person");
            out.object_start();
            out.key(b"address");
            super::encode_person_address(address, out, budget);
            out.key(b"taken_by");
            match taken_by {
                Some(item_2) => {
                    out.unsigned(*(item_2));
                }
                None => out.null(),
            }
            out.key(b"since");
            match since {
                Some(item_3) => {
                    out.unsigned((item_3).as_nanos());
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerChipPhase::Closing(field_0) => {
            out.object_start();
            out.key(b"closing");
            super::encode_closing_stage(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::OwnerChipPhase::Held { reason, revision } => {
            out.object_start();
            out.key(b"held");
            out.object_start();
            out.key(b"reason");
            super::encode_hold_category(reason, out, budget);
            out.key(b"revision");
            out.unsigned(*(revision));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerChipPhase::Ended => out.string(b"ended"),
    }
}

pub(super) fn encode_owner_chip_work(
    value: &jig_client::contract::OwnerChipWork,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerChipWork::Preparing => out.string(b"preparing"),
        jig_client::contract::OwnerChipWork::Starting => out.string(b"starting"),
        jig_client::contract::OwnerChipWork::Active => out.string(b"active"),
        jig_client::contract::OwnerChipWork::Writing => out.string(b"writing"),
        jig_client::contract::OwnerChipWork::Tool { call, tool } => {
            out.object_start();
            out.key(b"tool");
            out.object_start();
            out.key(b"call");
            super::hex(call, out);
            out.key(b"tool");
            super::hex(tool, out);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerChipWork::Waiting => out.string(b"waiting"),
        jig_client::contract::OwnerChipWork::Ending => out.string(b"ending"),
    }
}

pub(super) fn encode_owner_delegation(
    value: &jig_client::contract::OwnerDelegation,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"kinds");
    out.array_start();
    for item_1 in &value.kinds {
        super::encode_owner_authority_executor(*(item_1), out, budget);
    }
    out.array_end();
    out.key(b"tasks");
    out.unsigned(u64::from(value.tasks));
    out.key(b"depth");
    out.unsigned(u64::from(value.depth));
    out.object_end();
}

pub(super) fn encode_owner_escalation_holder(
    value: &jig_client::contract::OwnerEscalationHolder,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerEscalationHolder::Task(field_0) => {
            out.object_start();
            out.key(b"task");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::OwnerEscalationHolder::Person(field_0) => {
            out.object_start();
            out.key(b"person");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::OwnerEscalationHolder::Role { project, role } => {
            out.object_start();
            out.key(b"role");
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(*(project)));
            out.key(b"role");
            out.unsigned(u64::from(*(role)));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_executor(
    value: &jig_client::contract::OwnerExecutor,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::OwnerExecutor::Agent { charter } => {
            out.object_start();
            out.key(b"agent");
            out.object_start();
            out.key(b"charter");
            out.unsigned(u64::from(*(charter)));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerExecutor::Procedure { connector, code } => {
            out.object_start();
            out.key(b"procedure");
            out.object_start();
            out.key(b"connector");
            out.unsigned(u64::from(*(connector)));
            out.key(b"code");
            out.unsigned(u64::from(*(code)));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::OwnerExecutor::Person(field_0) => {
            out.object_start();
            out.key(b"person");
            super::encode_person_address(field_0, out, budget);
            out.object_end();
        }
    }
}

pub(super) fn encode_owner_failure_tries(
    value: &jig_client::contract::OwnerFailureTries,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"transient");
    out.unsigned(u64::from(value.transient));
    out.key(b"permanent");
    out.unsigned(u64::from(value.permanent));
    out.key(b"run");
    out.unsigned(u64::from(value.run));
    out.key(b"agent");
    out.unsigned(u64::from(value.agent));
    out.key(b"lost");
    out.unsigned(u64::from(value.lost));
    out.key(b"invalid");
    out.unsigned(u64::from(value.invalid));
    out.object_end();
}
