//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_ending(value: &jig_client::contract::Ending, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::Ending::Task(field_0) => {
            out.object_start();
            out.key(b"task");
            super::encode_owner_chip_ending(field_0, out, budget);
            out.object_end();
        }
        jig_client::contract::Ending::Done => out.string(b"done"),
        jig_client::contract::Ending::Failed { reason } => {
            out.object_start();
            out.key(b"failed");
            out.object_start();
            out.key(b"reason");
            match reason {
                Some(item_1) => {
                    budget.words((item_1).bytes(), out);
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Ending::Cancelled { by, reason, at } => {
            out.object_start();
            out.key(b"cancelled");
            out.object_start();
            out.key(b"by");
            super::encode_actor(by, out, budget);
            out.key(b"reason");
            match reason {
                Some(item_2) => {
                    budget.words((item_2).bytes(), out);
                }
                None => out.null(),
            }
            out.key(b"at");
            match at {
                Some(item_3) => {
                    out.unsigned((item_3).as_nanos());
                }
                None => out.null(),
            }
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Ending::Withdrawn { reason } => {
            out.object_start();
            out.key(b"withdrawn");
            out.object_start();
            out.key(b"reason");
            budget.words((reason).bytes(), out);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Ending::Decided { by, party, how, at } => {
            out.object_start();
            out.key(b"decided");
            out.object_start();
            out.key(b"by");
            out.unsigned(*(by));
            out.key(b"party");
            match party {
                Some(item_4) => {
                    super::encode_party(item_4, out, budget);
                }
                None => out.null(),
            }
            out.key(b"how");
            match how {
                Some(item_5) => {
                    super::encode_decision(item_5, out, budget);
                }
                None => out.null(),
            }
            out.key(b"at");
            out.unsigned((at).as_nanos());
            out.object_end();
            out.object_end();
        }
        jig_client::contract::Ending::Deleted => out.string(b"deleted"),
    }
}

pub(super) fn encode_executor(value: &jig_client::contract::ChipExecutor, out: &mut Encoder, _budget: &mut super::Cut) {
    match value {
        jig_client::contract::ChipExecutor::Agent { charter } => {
            out.object_start();
            out.key(b"agent");
            out.object_start();
            out.key(b"charter");
            out.unsigned(u64::from(*(charter)));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::ChipExecutor::Procedure { connector, kind } => {
            out.object_start();
            out.key(b"procedure");
            out.object_start();
            out.key(b"connector");
            out.unsigned(u64::from(*(connector)));
            out.key(b"kind");
            out.unsigned(u64::from(*(kind)));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::ChipExecutor::Party { party } => {
            out.object_start();
            out.key(b"party");
            out.object_start();
            out.key(b"party");
            out.unsigned(*(party));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::ChipExecutor::Role { project, role } => {
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

pub(super) fn encode_failure_class(
    value: jig_client::contract::FailureClass,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::FailureClass::HostLost => out.string(b"host_lost"),
        jig_client::contract::FailureClass::Temporary => out.string(b"temporary"),
        jig_client::contract::FailureClass::Permanent => out.string(b"permanent"),
        jig_client::contract::FailureClass::Stall => out.string(b"stall"),
    }
}

pub(super) fn encode_field_rule(value: &jig_client::contract::FieldRule, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"name");
    super::hex(&value.name, out);
    out.key(b"max");
    out.unsigned(u64::from(value.max));
    out.object_end();
}
