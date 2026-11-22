//! Durable task values translated exhaustively (protocol/store/records.md, 1).
use super::Context;
use crate::generated::tasks as mapped;
use crate::tasks_v13 as wire;
use jig_domain_tasks as domain;

/// Translate `ResultKind` to its durable value.
pub fn result_kind(source: &domain::ResultKind, limits: &wire::Limits) -> Result<wire::ResultKind, wire::Problem> {
    Ok(match source {
        domain::ResultKind::Approved { state, words_present } => {
            wire::ResultKind::Approved(wire::ApprovalMessage { state: state.clone(), words_present: *words_present })
        }
        domain::ResultKind::Job => wire::ResultKind::Job,
        domain::ResultKind::NoChange => wire::ResultKind::NoChange,
        domain::ResultKind::Report => wire::ResultKind::Report,
        domain::ResultKind::Verdict { code, fields } => wire::ResultKind::Verdict(wire::VerdictMessage {
            code: *code,
            fields: super::pieces::fields(
                fields,
                limits,
                limits.verdict_message_fields,
                wire::paths::VERDICT_MESSAGE_FIELDS,
            )?,
        }),
        domain::ResultKind::Change { connector, kind, resource } => {
            wire::ResultKind::Change(wire::ChangeMessage { connector: *connector, kind: *kind, resource: *resource })
        }
        domain::ResultKind::Failed => wire::ResultKind::Failed,
        domain::ResultKind::Cancelled => wire::ResultKind::Cancelled,
    })
}
/// Read `ResultKind` from its durable value.
pub fn read_result_kind(source: wire::ResultKind, limits: &wire::Limits) -> Result<domain::ResultKind, wire::Problem> {
    Ok(match source {
        wire::ResultKind::Approved(source) => {
            let wire::ApprovalMessage { state, words_present } = source;
            domain::ResultKind::Approved { state, words_present }
        }
        wire::ResultKind::Job => domain::ResultKind::Job,
        wire::ResultKind::NoChange => domain::ResultKind::NoChange,
        wire::ResultKind::Report => domain::ResultKind::Report,
        wire::ResultKind::Verdict(source) => {
            let wire::VerdictMessage { code, fields } = source;
            domain::ResultKind::Verdict {
                code,
                fields: super::pieces::read_fields(fields, limits, wire::paths::VERDICT_MESSAGE_FIELDS)?,
            }
        }
        wire::ResultKind::Change(source) => {
            let wire::ChangeMessage { connector, kind, resource } = source;
            domain::ResultKind::Change { connector, kind, resource }
        }
        wire::ResultKind::Failed => domain::ResultKind::Failed,
        wire::ResultKind::Cancelled => domain::ResultKind::Cancelled,
    })
}

/// Translate `MessageKind` to its durable value.
pub fn message_kind(source: &domain::MessageKind, limits: &wire::Limits) -> Result<wire::MessageKind, wire::Problem> {
    mapped::message_kind_to_record(source, &Context::new(limits, None))
}
/// Read `MessageKind` from its durable value.
pub fn read_message_kind(
    source: wire::MessageKind,
    limits: &wire::Limits,
) -> Result<domain::MessageKind, wire::Problem> {
    mapped::message_kind_to_domain(source, &Context::new(limits, None))
}
