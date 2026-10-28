//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_template_deadline(
    value: &jig_client::contract::TemplateDeadline,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::TemplateDeadline::None => out.string(b"none"),
        jig_client::contract::TemplateDeadline::Span(field_0) => {
            out.object_start();
            out.key(b"span");
            out.unsigned((field_0).as_nanos());
            out.object_end();
        }
        jig_client::contract::TemplateDeadline::Given => out.string(b"given"),
    }
}

pub(super) fn encode_template_executor(
    value: jig_client::contract::TemplateExecutor,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::TemplateExecutor::Charter(field_0) => {
            out.object_start();
            out.key(b"charter");
            out.unsigned(u64::from(*(field_0)));
            out.object_end();
        }
        jig_client::contract::TemplateExecutor::Chosen => out.string(b"chosen"),
    }
}

pub(super) fn encode_verdict_choice(
    value: &jig_client::contract::VerdictChoice,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"code");
    out.unsigned(u64::from(value.code));
    out.key(b"words");
    out.unsigned(u64::from(value.words));
    out.key(b"followups");
    out.unsigned(u64::from(value.followups));
    out.key(b"fields");
    out.array_start();
    for item_1 in &value.fields {
        super::encode_field_rule(item_1, out, budget);
    }
    out.array_end();
    out.object_end();
}

pub(super) fn encode_waited_for(value: jig_client::contract::WaitedFor, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::WaitedFor::Question => out.string(b"question"),
        jig_client::contract::WaitedFor::Proposal => out.string(b"proposal"),
        jig_client::contract::WaitedFor::Escalation => out.string(b"escalation"),
        jig_client::contract::WaitedFor::PersonTask => out.string(b"person_task"),
    }
}

pub(super) fn encode_waits_on(value: &jig_client::contract::WaitsOn, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::WaitsOn::Party { whom, entries, task, kind } => {
            out.object_start();
            out.key(b"party");
            out.object_start();
            out.key(b"whom");
            encode_whom(whom, out, budget);
            out.key(b"entries");
            out.unsigned(u64::from(*(entries)));
            out.key(b"task");
            out.unsigned(*(task));
            out.key(b"kind");
            encode_waited_for(*(kind), out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::WaitsOn::Dependencies { pending, count } => {
            out.object_start();
            out.key(b"dependencies");
            out.object_start();
            out.key(b"pending");
            out.array_start();
            for task in pending {
                out.unsigned(*task);
            }
            out.array_end();
            out.key(b"count");
            out.unsigned(u64::from(*count));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::WaitsOn::Delegates => out.string(b"delegates"),
        jig_client::contract::WaitsOn::News => out.string(b"news"),
        jig_client::contract::WaitsOn::Nothing => out.string(b"nothing"),
    }
}

pub(super) fn encode_wake_policy(value: &jig_client::contract::WakePolicy, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"words");
    out.boolean(value.words);
    out.key(b"results");
    out.boolean(value.results);
    out.key(b"topics");
    out.array_start();
    for item_1 in &value.topics {
        out.unsigned(u64::from(*(item_1)));
    }
    out.array_end();
    out.key(b"subscriptions");
    out.array_start();
    for item_2 in &value.subscriptions {
        super::encode_part(item_2, out, budget);
    }
    out.array_end();
    out.object_end();
}

pub(super) fn encode_wake_rule(value: &jig_client::contract::WakeRule, out: &mut Encoder, _budget: &mut super::Cut) {
    match value {
        jig_client::contract::WakeRule::Never => out.string(b"never"),
        jig_client::contract::WakeRule::Immediate => out.string(b"immediate"),
        jig_client::contract::WakeRule::Batch { count, age } => {
            out.object_start();
            out.key(b"batch");
            out.object_start();
            out.key(b"count");
            out.unsigned(u64::from(*(count)));
            out.key(b"age");
            out.unsigned((age).as_nanos());
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_whom(value: &jig_client::contract::Whom, out: &mut Encoder, _budget: &mut super::Cut) {
    match value {
        jig_client::contract::Whom::Person(field_0) => {
            out.object_start();
            out.key(b"person");
            out.unsigned(*(field_0));
            out.object_end();
        }
        jig_client::contract::Whom::Role { project, role } => {
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

pub(super) fn encode_writer_slot(value: &jig_client::contract::WriterSlot, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"resource");
    super::encode_part(&value.resource, out, budget);
    out.key(b"task");
    out.unsigned(value.task);
    out.key(b"attempt");
    match &value.attempt {
        Some(item_1) => {
            out.unsigned(*(item_1));
        }
        None => out.null(),
    }
    out.object_end();
}
