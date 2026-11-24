//! Wire projections of operations facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::wire_malformed_settings;
pub(in crate::codec) fn wire_task_flags(value: &c::TaskFlags) -> w::TaskFlags {
    w::TaskFlags {
        conversation: value.conversation,
        listed: value.listed,
        retention: wire_retention(&value.retention),
        ended_by: wire_ended_by(&value.ended_by),
    }
}

pub(in crate::codec) fn wire_retention(value: &c::Retention) -> w::Retention {
    match value {
        c::Retention::Kept => w::Retention::Kept,
        c::Retention::ForgottenOnceRead => w::Retention::ForgottenOnceRead,
    }
}

pub(in crate::codec) fn wire_ended_by(value: &c::EndedBy) -> w::EndedBy {
    match value {
        c::EndedBy::Result => w::EndedBy::Result,
        c::EndedBy::Requester => w::EndedBy::Requester,
        c::EndedBy::Standing => w::EndedBy::Standing,
        c::EndedBy::Closer(closer) => w::EndedBy::Closer(w::Closer { connector: closer.connector, code: closer.code }),
    }
}

pub(in crate::codec) fn wire_malformed(value: &c::Malformed) -> w::Malformed {
    match value {
        c::Malformed::Running => w::Malformed::Running,
        c::Malformed::Settings(value) => w::Malformed::Settings(wire_malformed_settings(value)),
        c::Malformed::SettingsUnchanged => w::Malformed::SettingsUnchanged,
        c::Malformed::ProjectMismatch => w::Malformed::ProjectMismatch,
        c::Malformed::TwoAsAllowed => w::Malformed::TwoAsAllowed,
        c::Malformed::ConversationNotAgent => w::Malformed::ConversationNotAgent,
        c::Malformed::Cycle => w::Malformed::Cycle,
        c::Malformed::ProposedAsAllowed => w::Malformed::ProposedAsAllowed,
        c::Malformed::EndedBy => w::Malformed::EndedBy,
        c::Malformed::Name(value) => w::Malformed::Name(w::MalformedName { rule: wire_name_rule(value) }),
    }
}

pub(in crate::codec) fn wire_name_rule(value: &c::NameRule) -> w::NameRule {
    match value {
        c::NameRule::Empty => w::NameRule::Empty,
        c::NameRule::Long => w::NameRule::Long,
        c::NameRule::Control => w::NameRule::Control,
        c::NameRule::Space => w::NameRule::Space,
        c::NameRule::Digits => w::NameRule::Digits,
    }
}

pub(in crate::codec) fn wire_new_deadline(value: &c::NewDeadline) -> w::NewDeadline {
    match value {
        c::NewDeadline::Named => w::NewDeadline::Named,
        c::NewDeadline::Span(value) => w::NewDeadline::Span(w::DeadlineSpan { value: *(value) }),
    }
}

pub(in crate::codec) fn wire_created_given(value: &c::CreatedGiven) -> w::CreatedGiven {
    w::CreatedGiven { task: value.task, amount: value.amount }
}
