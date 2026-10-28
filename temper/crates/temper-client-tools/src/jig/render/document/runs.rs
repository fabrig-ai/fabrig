//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_retry_notice(
    value: &jig_client::contract::RetryNotice,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::RetryNotice::OwnerFailed { class, count, allowance } => {
            out.object_start();
            out.key(b"owner_failed");
            out.object_start();
            out.key(b"class");
            super::encode_hold_failure(*(class), out, budget);
            out.key(b"count");
            out.unsigned(u64::from(*(count)));
            out.key(b"allowance");
            out.unsigned(u64::from(*(allowance)));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::RetryNotice::Failed { class, count, allowance } => {
            out.object_start();
            out.key(b"failed");
            out.object_start();
            out.key(b"class");
            super::encode_failure_class(*(class), out, budget);
            out.key(b"count");
            out.unsigned(u64::from(*(count)));
            out.key(b"allowance");
            out.unsigned(u64::from(*(allowance)));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::RetryNotice::Refused { count } => {
            out.object_start();
            out.key(b"refused");
            out.object_start();
            out.key(b"count");
            out.unsigned(u64::from(*(count)));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_revision(value: jig_client::contract::Revision, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    out.object_start();
    out.key(b"value");
    out.unsigned(value.value);
    out.object_end();
}

pub(super) fn encode_run_observation(
    value: &jig_client::contract::RunObservation,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"channel");
    out.unsigned(value.channel.raw());
    out.key(b"work");
    encode_run_work(&value.work, out, budget);
    out.key(b"since");
    out.unsigned(value.since.as_nanos());
    out.key(b"reported_at");
    out.unsigned(value.reported_at.as_nanos());
    out.key(b"started");
    match &value.started {
        Some(item_1) => {
            out.unsigned((item_1).as_nanos());
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_run_stage(value: &jig_client::contract::RunStage, out: &mut Encoder, _budget: &mut super::Cut) {
    match value {
        jig_client::contract::RunStage::Idle => out.string(b"idle"),
        jig_client::contract::RunStage::Due => out.string(b"due"),
        jig_client::contract::RunStage::Preparing => out.string(b"preparing"),
        jig_client::contract::RunStage::Claimed { attempt } => {
            out.object_start();
            out.key(b"claimed");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::RunStage::Running { attempt } => {
            out.object_start();
            out.key(b"running");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::RunStage::BackingOff { until } => {
            out.object_start();
            out.key(b"backing_off");
            out.object_start();
            out.key(b"until");
            out.unsigned((until).as_nanos());
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_run_work(value: &jig_client::contract::RunWork, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::RunWork::Preparing => out.string(b"preparing"),
        jig_client::contract::RunWork::Starting => out.string(b"starting"),
        jig_client::contract::RunWork::Active => out.string(b"active"),
        jig_client::contract::RunWork::Writing => out.string(b"writing"),
        jig_client::contract::RunWork::Tool { call, tool } => {
            out.object_start();
            out.key(b"tool");
            out.object_start();
            out.key(b"call");
            super::encode_call_identity(call, out, budget);
            out.key(b"tool");
            budget.words((tool).bytes(), out);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::RunWork::Waiting => out.string(b"waiting"),
        jig_client::contract::RunWork::Ending => out.string(b"ending"),
    }
}
