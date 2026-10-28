//! Concrete integer JSON for the client contract (contract.md, 5).
use alloc::boxed::Box;
use skein_json::writer::Encoder;
pub(super) fn encode_owner_wake_policy(
    value: &jig_client::contract::OwnerWakePolicy,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"words");
    super::encode_wake_rule(&value.words, out, budget);
    out.key(b"notices");
    super::encode_wake_rule(&value.notices, out, budget);
    out.key(b"news");
    super::encode_wake_rule(&value.news, out, budget);
    out.key(b"results");
    super::encode_results_wake(value.results, out, budget);
    out.key(b"questions");
    out.boolean(value.questions);
    out.key(b"answers");
    out.boolean(value.answers);
    out.key(b"timers");
    out.boolean(value.timers);
    out.object_end();
}

pub(super) fn encode_parameter_document(
    value: &jig_client::contract::ParameterDocument,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    match value {
        jig_client::contract::ParameterDocument::Number { name, value } => {
            out.object_start();
            out.key(b"number");
            out.object_start();
            out.key(b"name");
            out.unsigned(u64::from(*(name)));
            out.key(b"value");
            out.unsigned(*(value));
            out.object_end();
            out.object_end();
        }
        jig_client::contract::ParameterDocument::Bytes { name, value } => {
            out.object_start();
            out.key(b"bytes");
            out.object_start();
            out.key(b"name");
            out.unsigned(u64::from(*(name)));
            out.key(b"value");
            super::encode_document_text(value, out, budget);
            out.object_end();
            out.object_end();
        }
        jig_client::contract::ParameterDocument::Resource { name, connector, resource } => {
            out.object_start();
            out.key(b"resource");
            out.object_start();
            out.key(b"name");
            out.unsigned(u64::from(*(name)));
            out.key(b"connector");
            out.unsigned(u64::from(*(connector)));
            out.key(b"resource");
            out.unsigned(*(resource));
            out.object_end();
            out.object_end();
        }
    }
}

pub(super) fn encode_part(value: &jig_client::contract::Part, out: &mut Encoder, budget: &mut super::Cut) {
    match value {
        jig_client::contract::Part::Connector { connector } => {
            encode_part_connector(*connector, out, budget);
        }
        jig_client::contract::Part::Resource { connector, path } => {
            encode_part_resource(*connector, path, out, budget);
        }
        jig_client::contract::Part::Kind { connector, kind } => {
            encode_part_kind(*connector, *kind, out, budget);
        }
        jig_client::contract::Part::Panel { connector, task } => {
            encode_part_panel(*connector, *task, out, budget);
        }
        jig_client::contract::Part::Result { connector, task } => {
            encode_part_result(*connector, *task, out, budget);
        }
        jig_client::contract::Part::Effect { connector, proposal } => {
            encode_part_effect(*connector, proposal, out, budget);
        }
        jig_client::contract::Part::OwnerEffect { connector, proposal } => {
            encode_part_owner_effect(*connector, *proposal, out, budget);
        }
        jig_client::contract::Part::Outbox { connector, scope, entry } => {
            encode_part_outbox(*connector, *scope, *entry, out, budget);
        }
        jig_client::contract::Part::Hold { connector, task } => {
            encode_part_hold(*connector, *task, out, budget);
        }
        jig_client::contract::Part::Project { connector, project } => {
            encode_part_project(*connector, *project, out, budget);
        }
        jig_client::contract::Part::Deployment { connector } => {
            encode_part_deployment(*connector, out, budget);
        }
    }
}

fn encode_part_connector(connector: u16, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"connector");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.object_end();
    out.object_end();
}

fn encode_part_resource(
    connector: u16,
    path: &skein_lib::List<Box<[u8]>>,
    out: &mut Encoder,
    _budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"resource");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"path");
    out.array_start();
    for item_1 in path {
        super::hex(item_1, out);
    }
    out.array_end();
    out.object_end();
    out.object_end();
}

fn encode_part_kind(connector: u16, kind: u16, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"kind");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"kind");
    out.unsigned(u64::from(kind));
    out.object_end();
    out.object_end();
}

fn encode_part_panel(connector: u16, task: u64, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"panel");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"task");
    out.unsigned(task);
    out.object_end();
    out.object_end();
}

fn encode_part_result(connector: u16, task: u64, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"result");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"task");
    out.unsigned(task);
    out.object_end();
    out.object_end();
}

fn encode_part_effect(
    connector: u16,
    proposal: &jig_client::contract::ProposalKey,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"effect");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"proposal");
    super::encode_proposal_key(proposal, out, budget);
    out.object_end();
    out.object_end();
}

fn encode_part_owner_effect(connector: u16, proposal: u64, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"owner_effect");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"proposal");
    out.unsigned(proposal);
    out.object_end();
    out.object_end();
}

fn encode_part_outbox(
    connector: u16,
    scope: jig_client::contract::PartScope,
    entry: u64,
    out: &mut Encoder,
    budget: &mut super::Cut,
) {
    out.object_start();
    out.key(b"outbox");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"scope");
    super::encode_part_scope(scope, out, budget);
    out.key(b"entry");
    out.unsigned(entry);
    out.object_end();
    out.object_end();
}

fn encode_part_hold(connector: u16, task: u64, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"hold");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"task");
    out.unsigned(task);
    out.object_end();
    out.object_end();
}

fn encode_part_project(connector: u16, project: u32, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"project");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.key(b"project");
    out.unsigned(u64::from(project));
    out.object_end();
    out.object_end();
}

fn encode_part_deployment(connector: u16, out: &mut Encoder, _budget: &mut super::Cut) {
    out.object_start();
    out.key(b"deployment");
    out.object_start();
    out.key(b"connector");
    out.unsigned(u64::from(connector));
    out.object_end();
    out.object_end();
}
