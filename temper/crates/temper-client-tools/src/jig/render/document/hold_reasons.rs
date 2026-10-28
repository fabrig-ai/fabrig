//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_hold_reason(value: &jig_client::contract::HoldReason, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::HoldReason::Tries { failures } => {
            encode_hold_reason_tries(failures, out, budget);
        }
        jig_client::contract::HoldReason::Stalled { since, waiting } => {
            encode_hold_reason_stalled(*since, waiting, out, budget);
        }
        jig_client::contract::HoldReason::Dependency { task, how } => {
            encode_hold_reason_dependency(*task, how, out, budget);
        }
        jig_client::contract::HoldReason::Drift { part } => {
            encode_hold_reason_drift(part, out, budget);
        }
        jig_client::contract::HoldReason::EffectFailed { part } => {
            encode_hold_reason_effect_failed(part, out, budget);
        }
        jig_client::contract::HoldReason::EffectUncertain { part } => {
            encode_hold_reason_effect_uncertain(part, out, budget);
        }
        jig_client::contract::HoldReason::Budget { spent, limit } => {
            encode_hold_reason_budget(*spent, *limit, out, budget);
        }
        jig_client::contract::HoldReason::Deadline { at } => {
            encode_hold_reason_deadline(*at, out, budget);
        }
        jig_client::contract::HoldReason::Stopped { by, at } => {
            encode_hold_reason_stopped(by, *at, out, budget);
        }
        jig_client::contract::HoldReason::Resource { resource, holder } => {
            encode_hold_reason_resource(resource, holder, out, budget);
        }
        jig_client::contract::HoldReason::Pool { part } => {
            encode_hold_reason_pool(part, out, budget);
        }
        jig_client::contract::HoldReason::Procedure { part } => {
            encode_hold_reason_procedure(part, out, budget);
        }
    }
}

fn encode_hold_reason_tries(
    failures: &skein_lib::List<jig_client::contract::AttemptFailure>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"tries");
    out.object_start();
    out.key(b"failures");
    out.array_start();
    for item_1 in failures {
        super::encode_attempt_failure(item_1, out, budget);
    }
    out.array_end();
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_stalled(
    since: skein_lib::Wall,
    waiting: &jig_client::contract::Text,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"stalled");
    out.object_start();
    out.key(b"since");
    out.unsigned((since).as_nanos());
    out.key(b"waiting");
    budget.words((waiting).bytes(), out);
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_dependency(
    task: u64,
    how: &jig_client::contract::Ending,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"dependency");
    out.object_start();
    out.key(b"task");
    out.unsigned(task);
    out.key(b"how");
    super::encode_ending(how, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_drift(part: &jig_client::contract::Part, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"drift");
    out.object_start();
    out.key(b"part");
    super::encode_part(part, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_effect_failed(part: &jig_client::contract::Part, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"effect_failed");
    out.object_start();
    out.key(b"part");
    super::encode_part(part, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_effect_uncertain(part: &jig_client::contract::Part, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"effect_uncertain");
    out.object_start();
    out.key(b"part");
    super::encode_part(part, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_budget(spent: u64, limit: u64, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"budget");
    out.object_start();
    out.key(b"spent");
    out.unsigned(spent);
    out.key(b"limit");
    out.unsigned(limit);
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_deadline(at: skein_lib::Wall, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"deadline");
    out.object_start();
    out.key(b"at");
    out.unsigned((at).as_nanos());
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_stopped(
    by: &jig_client::contract::Party,
    at: skein_lib::Wall,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"stopped");
    out.object_start();
    out.key(b"by");
    super::encode_party(by, out, budget);
    out.key(b"at");
    out.unsigned((at).as_nanos());
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_resource(
    resource: &jig_client::contract::Part,
    holder: &jig_client::contract::Actor,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"resource");
    out.object_start();
    out.key(b"resource");
    super::encode_part(resource, out, budget);
    out.key(b"holder");
    super::encode_actor(holder, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_pool(part: &jig_client::contract::Part, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"pool");
    out.object_start();
    out.key(b"part");
    super::encode_part(part, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_hold_reason_procedure(part: &jig_client::contract::Part, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"procedure");
    out.object_start();
    out.key(b"part");
    super::encode_part(part, out, budget);
    out.object_end();
    out.object_end();
}
pub(super) fn encode_impact(value: &jig_client::contract::Impact, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"tasks");
    super::encode_phase_counts(&value.tasks, out, budget);
    out.key(b"live_runs");
    out.unsigned(u64::from(value.live_runs));
    out.key(b"proposals");
    out.unsigned(u64::from(value.proposals));
    out.object_end();
}
