//! Wire projections of operations facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::wire_malformed_settings;
pub(in crate::codec) fn wire_task_flags(value: &c::TaskFlags, limits: &w::Limits) -> Result<w::TaskFlags, w::Problem> {
    w::TaskFlags::new(
        limits,
        w::TaskFlagsParts {
            conversation: value.conversation,
            listed: value.listed,
            retention: wire_retention(&value.retention, limits)?,
            ended_by: wire_ended_by(&value.ended_by, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_retention(value: &c::Retention, _limits: &w::Limits) -> Result<w::Retention, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Retention::Kept => w::Retention::Kept,
        c::Retention::ForgottenOnceRead => w::Retention::ForgottenOnceRead,
    })
}

pub(in crate::codec) fn wire_ended_by(value: &c::EndedBy, _limits: &w::Limits) -> Result<w::EndedBy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EndedBy::Result => w::EndedBy::Result,
        c::EndedBy::Requester => w::EndedBy::Requester,
    })
}

pub(in crate::codec) fn wire_malformed(value: &c::Malformed, limits: &w::Limits) -> Result<w::Malformed, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Malformed::Running => w::Malformed::Running,
        c::Malformed::Settings(value) => w::Malformed::Settings(wire_malformed_settings(value, limits)?),
        c::Malformed::SettingsUnchanged => w::Malformed::SettingsUnchanged,
        c::Malformed::ProjectMismatch => w::Malformed::ProjectMismatch,
        c::Malformed::TwoAsAllowed => w::Malformed::TwoAsAllowed,
        c::Malformed::ConversationNotAgent => w::Malformed::ConversationNotAgent,
        c::Malformed::Cycle => w::Malformed::Cycle,
        c::Malformed::ProposedAsAllowed => w::Malformed::ProposedAsAllowed,
        c::Malformed::EndedBy => w::Malformed::EndedBy,
        c::Malformed::Name(value) => w::Malformed::Name(w::MalformedName::new(
            limits,
            w::MalformedNameParts { rule: wire_name_rule(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_name_rule(value: &c::NameRule, _limits: &w::Limits) -> Result<w::NameRule, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NameRule::Empty => w::NameRule::Empty,
        c::NameRule::Long => w::NameRule::Long,
        c::NameRule::Control => w::NameRule::Control,
        c::NameRule::Space => w::NameRule::Space,
        c::NameRule::Digits => w::NameRule::Digits,
    })
}

pub(in crate::codec) fn wire_new_deadline(
    value: &c::NewDeadline,
    limits: &w::Limits,
) -> Result<w::NewDeadline, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NewDeadline::Named => w::NewDeadline::Named,
        c::NewDeadline::Span(value) => {
            w::NewDeadline::Span(w::DeadlineSpan::new(limits, w::DeadlineSpanParts { value: *(value) })?)
        }
    })
}

pub(in crate::codec) fn wire_created_given(
    value: &c::CreatedGiven,
    limits: &w::Limits,
) -> Result<w::CreatedGiven, w::Problem> {
    w::CreatedGiven::new(limits, w::CreatedGivenParts { task: value.task, amount: value.amount })
}
