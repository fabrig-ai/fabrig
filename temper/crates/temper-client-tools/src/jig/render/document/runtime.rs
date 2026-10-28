//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_runtime(value: &jig_client::contract::Runtime, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::Runtime::Active(field_0) => {
            encode_runtime_active(field_0, out, budget);
        }
        jig_client::contract::Runtime::Waiting { dependencies, hold } => {
            encode_runtime_waiting(dependencies, hold.as_ref(), out, budget);
        }
        jig_client::contract::Runtime::Idle => out.string(b"idle"),
        jig_client::contract::Runtime::Starting => out.string(b"starting"),
        jig_client::contract::Runtime::Running { attempt, since, tool } => {
            encode_runtime_running(*attempt, *since, tool.as_ref(), out, budget);
        }
        jig_client::contract::Runtime::WaitingForWords { attempt, since } => {
            encode_runtime_waiting_for_words(*attempt, *since, out, budget);
        }
        jig_client::contract::Runtime::BackingOff { at, attempt, tries, class, reason } => {
            encode_runtime_backing_off(*at, *attempt, *tries, *class, reason, out, budget);
        }
        jig_client::contract::Runtime::Procedure { panel } => {
            encode_runtime_procedure(panel, out, budget);
        }
        jig_client::contract::Runtime::Person { addressed, since } => {
            encode_runtime_person(addressed, since.as_ref(), out, budget);
        }
        jig_client::contract::Runtime::Closing(field_0) => {
            encode_runtime_closing(field_0, out, budget);
        }
        jig_client::contract::Runtime::Held { was } => {
            encode_runtime_held(was.as_ref(), out, budget);
        }
        jig_client::contract::Runtime::Ended => out.string(b"ended"),
    }
}

fn encode_runtime_active(field_0: &jig_client::contract::ActiveRun, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"active");
    super::encode_active_run(field_0, out, budget);
    out.object_end();
}

fn encode_runtime_waiting(
    dependencies: &skein_lib::List<u64>,
    hold: Option<&jig_client::contract::Part>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"waiting");
    out.object_start();
    out.key(b"dependencies");
    out.array_start();
    for item_1 in dependencies {
        out.unsigned(*(item_1));
    }
    out.array_end();
    out.key(b"hold");
    match hold {
        Some(item_2) => {
            super::encode_part(item_2, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}

fn encode_runtime_running(
    attempt: u64,
    since: skein_lib::Wall,
    tool: Option<&jig_client::contract::Text>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"running");
    out.object_start();
    out.key(b"attempt");
    out.unsigned(attempt);
    out.key(b"since");
    out.unsigned((since).as_nanos());
    out.key(b"tool");
    match tool {
        Some(item_3) => {
            budget.words((item_3).bytes(), out);
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}

fn encode_runtime_waiting_for_words(attempt: u64, since: skein_lib::Wall, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"waiting_for_words");
    out.object_start();
    out.key(b"attempt");
    out.unsigned(attempt);
    out.key(b"since");
    out.unsigned((since).as_nanos());
    out.object_end();
    out.object_end();
}

fn encode_runtime_backing_off(
    at: skein_lib::Wall,
    attempt: u64,
    tries: u32,
    class: jig_client::contract::FailureClass,
    reason: &jig_client::contract::Text,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"backing_off");
    out.object_start();
    out.key(b"at");
    out.unsigned((at).as_nanos());
    out.key(b"attempt");
    out.unsigned(attempt);
    out.key(b"tries");
    out.unsigned(u64::from(tries));
    out.key(b"class");
    super::encode_failure_class(class, out, budget);
    out.key(b"reason");
    budget.words((reason).bytes(), out);
    out.object_end();
    out.object_end();
}

fn encode_runtime_procedure(panel: &jig_client::contract::Part, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"procedure");
    out.object_start();
    out.key(b"panel");
    super::encode_part(panel, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_runtime_person(
    addressed: &jig_client::contract::Recipient,
    since: Option<&skein_lib::Wall>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"person");
    out.object_start();
    out.key(b"addressed");
    super::encode_recipient(addressed, out, budget);
    out.key(b"since");
    match since {
        Some(item_4) => {
            out.unsigned((item_4).as_nanos());
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}

fn encode_runtime_closing(field_0: &jig_client::contract::Closing, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"closing");
    super::encode_closing(field_0, out, budget);
    out.object_end();
}

fn encode_runtime_held(was: Option<&jig_client::contract::HeldRuntime>, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"held");
    out.object_start();
    out.key(b"was");
    match was {
        Some(item_5) => {
            super::encode_held_runtime(item_5, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}
pub(super) fn encode_specification_document(
    value: &jig_client::contract::SpecificationDocument,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"specification");
    super::encode_document_specification(&value.specification, out, budget);
    out.key(b"cut");
    out.boolean(value.cut);
    out.object_end();
}

pub(super) fn encode_spend_line(value: &jig_client::contract::SpendLine, out: &mut Encoder, _budget: &mut super::Cut) {
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
    out.key(b"spent");
    out.unsigned(value.spent);
    out.object_end();
}

pub(super) fn encode_steering(value: &jig_client::contract::Steering, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"addressed");
    match &value.addressed {
        Some(item_1) => {
            super::encode_recipient(item_1, out, budget);
        }
        None => out.null(),
    }
    out.key(b"person_revision");
    match &value.person_revision {
        Some(item_2) => {
            super::encode_revision(*(item_2), out, budget);
        }
        None => out.null(),
    }
    out.key(b"taken_by");
    match &value.taken_by {
        Some(item_3) => {
            out.unsigned(*(item_3));
        }
        None => out.null(),
    }
    out.key(b"revision");
    super::encode_revision(value.revision, out, budget);
    out.key(b"spec");
    encode_specification_document(&value.spec, out, budget);
    out.key(b"instructions");
    match &value.instructions {
        Some(item_4) => {
            super::encode_document_text(item_4, out, budget);
        }
        None => out.null(),
    }
    out.key(b"wake");
    super::encode_owner_wake_policy(&value.wake, out, budget);
    out.key(b"dependencies");
    out.array_start();
    for item_5 in &value.dependencies {
        out.unsigned(*(item_5));
    }
    out.array_end();
    out.key(b"authority");
    super::encode_owner_authority_document(&value.authority, out, budget);
    out.key(b"budget");
    super::encode_budget(&value.budget, out, budget);
    out.object_end();
}

pub(super) fn encode_subscription(
    value: &jig_client::contract::Subscription,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"number");
    out.unsigned(value.number);
    out.key(b"kind");
    super::encode_subscription_kind(&value.kind, out, budget);
    out.object_end();
}
