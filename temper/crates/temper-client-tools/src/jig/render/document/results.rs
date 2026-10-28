//! Concrete integer JSON for the client contract (contract.md, 5).
use skein_json::writer::Encoder;
pub(super) fn encode_result_details(
    value: &jig_client::contract::ResultDetails,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"contract");
    super::encode_task_contract_document(&value.contract, out, budget);
    out.key(b"requester");
    super::encode_owner_requester(&value.requester, out, budget);
    out.key(b"executor");
    super::encode_owner_executor(&value.executor, out, budget);
    out.key(b"chosen_by");
    match &value.chosen_by {
        Some(item_1) => {
            out.unsigned(*(item_1));
        }
        None => out.null(),
    }
    out.key(b"delegates");
    out.array_start();
    for item_2 in &value.delegates {
        super::encode_owner_task_link(item_2, out, budget);
    }
    out.array_end();
    out.key(b"followups");
    match &value.followups {
        Some(item_3) => {
            out.array_start();
            for item_4 in item_3 {
                super::encode_owner_task_link(item_4, out, budget);
            }
            out.array_end();
        }
        None => out.null(),
    }
    out.key(b"proposal");
    match &value.proposal {
        Some(item_5) => {
            super::encode_proposal_key(item_5, out, budget);
        }
        None => out.null(),
    }
    out.key(b"cut");
    out.boolean(value.cut);
    out.object_end();
}

pub(super) fn encode_result_field(
    value: &jig_client::contract::ResultField,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"name");
    super::hex(&value.name, out);
    out.key(b"value");
    super::hex(&value.value, out);
    out.object_end();
}

pub(super) fn encode_result_value(
    value: &jig_client::contract::ResultValue,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::ResultValue::Report(field_0) => {
            encode_result_value_report(field_0, out, budget);
        }
        jig_client::contract::ResultValue::OwnerVerdict { verdict, words, fields } => {
            encode_result_value_owner_verdict(*verdict, words, fields, out, budget);
        }
        jig_client::contract::ResultValue::Verdict { verdict, words, fields, tasks, proposal } => {
            encode_result_value_verdict(*verdict, words, fields, tasks, proposal.as_ref(), out, budget);
        }
        jig_client::contract::ResultValue::Connector { summary, part } => {
            encode_result_value_connector(summary, part, out, budget);
        }
        jig_client::contract::ResultValue::Change { kind, part, words } => {
            encode_result_value_change(*kind, part, words, out, budget);
        }
        jig_client::contract::ResultValue::Choice { option, by, reason } => {
            encode_result_value_choice(*option, by, reason, out, budget);
        }
        jig_client::contract::ResultValue::Approved { state, words } => {
            encode_result_value_approved(state, words.as_ref(), out, budget);
        }
        jig_client::contract::ResultValue::Failure(field_0) => {
            encode_result_value_failure(field_0, out, budget);
        }
        jig_client::contract::ResultValue::Words(field_0) => {
            encode_result_value_words(field_0, out, budget);
        }
    }
}

fn encode_result_value_report(
    field_0: &jig_client::contract::DocumentText,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"report");
    super::encode_document_text(field_0, out, budget);
    out.object_end();
}

fn encode_result_value_owner_verdict(
    verdict: u32,
    words: &jig_client::contract::DocumentText,
    fields: &skein_lib::List<jig_client::contract::ResultField>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"owner_verdict");
    out.object_start();
    out.key(b"verdict");
    out.unsigned(u64::from(verdict));
    out.key(b"words");
    super::encode_document_text(words, out, budget);
    out.key(b"fields");
    out.array_start();
    for item_1 in fields {
        encode_result_field(item_1, out, budget);
    }
    out.array_end();
    out.object_end();
    out.object_end();
}

fn encode_result_value_verdict(
    verdict: u32,
    words: &jig_client::contract::DocumentText,
    fields: &skein_lib::List<jig_client::contract::ResultField>,
    tasks: &skein_lib::List<jig_client::contract::Chip>,
    proposal: Option<&jig_client::contract::ProposalKey>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"verdict");
    out.object_start();
    out.key(b"verdict");
    out.unsigned(u64::from(verdict));
    out.key(b"words");
    super::encode_document_text(words, out, budget);
    out.key(b"fields");
    out.array_start();
    for item_2 in fields {
        encode_result_field(item_2, out, budget);
    }
    out.array_end();
    out.key(b"tasks");
    out.array_start();
    for item_3 in tasks {
        super::encode_chip(item_3, out, budget);
    }
    out.array_end();
    out.key(b"proposal");
    match proposal {
        Some(item_4) => {
            super::encode_proposal_key(item_4, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}

fn encode_result_value_connector(
    summary: &jig_client::contract::DocumentText,
    part: &jig_client::contract::Part,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"connector");
    out.object_start();
    out.key(b"summary");
    super::encode_document_text(summary, out, budget);
    out.key(b"part");
    super::encode_part(part, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_result_value_change(
    kind: u16,
    part: &jig_client::contract::OwnerPart,
    words: &jig_client::contract::DocumentText,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"change");
    out.object_start();
    out.key(b"kind");
    out.unsigned(u64::from(kind));
    out.key(b"part");
    super::encode_owner_part(part, out, budget);
    out.key(b"words");
    super::encode_document_text(words, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_result_value_choice(
    option: u32,
    by: &jig_client::contract::Party,
    reason: &jig_client::contract::DocumentText,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"choice");
    out.object_start();
    out.key(b"option");
    out.unsigned(u64::from(option));
    out.key(b"by");
    super::encode_party(by, out, budget);
    out.key(b"reason");
    super::encode_document_text(reason, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_result_value_approved(
    state: &[u8],
    words: Option<&jig_client::contract::DocumentText>,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"approved");
    out.object_start();
    out.key(b"state");
    super::hex(state, out);
    out.key(b"words");
    match words {
        Some(item_5) => {
            super::encode_document_text(item_5, out, budget);
        }
        None => out.null(),
    }
    out.object_end();
    out.object_end();
}

fn encode_result_value_failure(
    field_0: &jig_client::contract::DocumentText,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"failure");
    super::encode_document_text(field_0, out, budget);
    out.object_end();
}

fn encode_result_value_words(field_0: &jig_client::contract::DocumentText, out: &mut Encoder, budget: &mut super::Cut) {
    out.object_start();
    out.key(b"words");
    super::encode_document_text(field_0, out, budget);
    out.object_end();
}
pub(super) fn encode_results_wake(
    value: jig_client::contract::ResultsWake,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    let value = &value;
    match value {
        jig_client::contract::ResultsWake::Never => out.string(b"never"),
        jig_client::contract::ResultsWake::Each => out.string(b"each"),
        jig_client::contract::ResultsWake::LastOrFailure => out.string(b"last_or_failure"),
    }
}

pub(super) fn encode_retention(value: jig_client::contract::Retention, out: &mut Encoder, _budget: &mut super::Cut) {
    let value = &value;
    match value {
        jig_client::contract::Retention::Kept => out.string(b"kept"),
        jig_client::contract::Retention::ForgottenOnceRead => out.string(b"forgotten_once_read"),
    }
}
