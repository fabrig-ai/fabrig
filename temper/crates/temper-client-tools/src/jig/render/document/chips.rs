//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_chip(value: &jig_client::contract::Chip, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"task");
    out.unsigned(value.task);
    out.key(b"title");
    super::encode_document_text(&value.title, out, budget);
    out.key(b"executor");
    super::encode_executor(&value.executor, out, budget);
    out.key(b"phase");
    super::encode_phase(value.phase, out, budget);
    out.key(b"details");
    encode_chip_phase(&value.details, out, budget);
    out.object_end();
}

pub(super) fn encode_chip_phase(value: &jig_client::contract::ChipPhase, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::ChipPhase::Owner { phase, details } => {
            encode_chip_phase_owner(phase, details, out, budget);
        }
        jig_client::contract::ChipPhase::Active(field_0) => {
            encode_chip_phase_active(field_0, out, budget);
        }
        jig_client::contract::ChipPhase::Waiting { dependencies, hold } => {
            encode_chip_phase_waiting(dependencies, hold.as_ref(), out, budget);
        }
        jig_client::contract::ChipPhase::Idle { wake } => {
            encode_chip_phase_idle(wake, out, budget);
        }
        jig_client::contract::ChipPhase::Starting => out.string(b"starting"),
        jig_client::contract::ChipPhase::Running { attempt, since, tool } => {
            encode_chip_phase_running(*attempt, *since, tool.as_ref(), out, budget);
        }
        jig_client::contract::ChipPhase::WaitingForWords { attempt, since } => {
            encode_chip_phase_waiting_for_words(*attempt, *since, out, budget);
        }
        jig_client::contract::ChipPhase::BackingOff { at, attempt, tries, class, reason } => {
            encode_chip_phase_backing_off(*at, *attempt, *tries, *class, reason, out, budget);
        }
        jig_client::contract::ChipPhase::Procedure { panel } => {
            encode_chip_phase_procedure(panel, out, budget);
        }
        jig_client::contract::ChipPhase::Person { addressed, taken, since } => {
            encode_chip_phase_person(addressed, taken.as_ref(), since.as_ref(), out, budget);
        }
        jig_client::contract::ChipPhase::Closing(field_0) => {
            encode_chip_phase_closing(field_0, out, budget);
        }
        jig_client::contract::ChipPhase::Held { reason, since, revision, was } => {
            encode_chip_phase_held(reason, *since, *revision, was.as_ref(), out, budget);
        }
        jig_client::contract::ChipPhase::Ended { how, at } => {
            encode_chip_phase_ended(how, *at, out, budget);
        }
    }
}

fn encode_chip_phase_owner(
    phase: &jig_client::contract::OwnerChipPhase,
    details: &jig_client::contract::OwnerChipDetails,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"owner");
    out.object_start();
    out.key(b"phase");
    super::encode_owner_chip_phase(phase, out, budget);
    out.key(b"details");
    super::encode_owner_chip_details(details, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_chip_phase_active(field_0: &jig_client::contract::ActiveRun, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"active");
    super::encode_active_run(field_0, out, budget);
    out.object_end();
}

fn encode_chip_phase_waiting(
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

fn encode_chip_phase_idle(wake: &jig_client::contract::WakePolicy, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"idle");
    out.object_start();
    out.key(b"wake");
    super::encode_wake_policy(wake, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_chip_phase_running(
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

fn encode_chip_phase_waiting_for_words(
    attempt: u64,
    since: skein_lib::Wall,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
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

fn encode_chip_phase_backing_off(
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

fn encode_chip_phase_procedure(panel: &jig_client::contract::Part, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"procedure");
    out.object_start();
    out.key(b"panel");
    super::encode_part(panel, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_chip_phase_person(
    addressed: &jig_client::contract::Recipient,
    taken: Option<&jig_client::contract::TakenBy>,
    since: Option<&skein_lib::Wall>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"person");
    out.object_start();
    out.key(b"addressed");
    super::encode_recipient(addressed, out, budget);
    out.key(b"taken");
    match taken {
        Some(item_4) => {
            super::encode_taken_by(item_4, out, budget);
        }
        None => out.null(),
    }
    out.key(b"since");
    match since {
        Some(item_5) => {
            out.unsigned((item_5).as_nanos());
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}

fn encode_chip_phase_closing(field_0: &jig_client::contract::Closing, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"closing");
    encode_closing(field_0, out, budget);
    out.object_end();
}

fn encode_chip_phase_held(
    reason: &jig_client::contract::HoldReason,
    since: skein_lib::Wall,
    revision: jig_client::contract::Revision,
    was: Option<&jig_client::contract::HeldRuntime>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"held");
    out.object_start();
    out.key(b"reason");
    super::encode_hold_reason(reason, out, budget);
    out.key(b"since");
    out.unsigned((since).as_nanos());
    out.key(b"revision");
    super::encode_revision(revision, out, budget);
    out.key(b"was");
    match was {
        Some(item_6) => {
            super::encode_held_runtime(item_6, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}

fn encode_chip_phase_ended(
    how: &jig_client::contract::Ending,
    at: skein_lib::Wall,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"ended");
    out.object_start();
    out.key(b"how");
    super::encode_ending(how, out, budget);
    out.key(b"at");
    out.unsigned((at).as_nanos());
    out.object_end();
    out.object_end();
}
pub(super) fn encode_closing(value: &jig_client::contract::Closing, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"delegates");
    out.unsigned(u64::from(value.delegates));
    out.key(b"connectors");
    match &value.connectors {
        Some(item_1) => {
            out.array_start();
            for item_2 in item_1 {
                out.unsigned(u64::from(*(item_2)));
            }
            out.array_end();
        }
        None => out.null(),
    }
    out.key(b"stage");
    match &value.stage {
        Some(item_3) => {
            encode_closing_stage(item_3, out, budget);
        }
        None => out.null(),
    }
    out.key(b"ending");
    match &value.ending {
        Some(item_4) => {
            super::encode_ending(item_4, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
}

pub(super) fn encode_closing_stage(
    value: &jig_client::contract::ClosingStage,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::ClosingStage::Run { attempt } => {
            out.object_start();
            out.key(b"run");
            out.object_start();
            out.key(b"attempt");
            out.unsigned(*(attempt));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::ClosingStage::Delegates => out.string(b"delegates"),
        jig_client::contract::ClosingStage::Effects => out.string(b"effects"),
        jig_client::contract::ClosingStage::Releases => out.string(b"releases"),
        jig_client::contract::ClosingStage::Settled => out.string(b"settled"),
    }
}
