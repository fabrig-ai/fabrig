//! Wire projections of owner wake facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
pub(in crate::codec) fn wire_wake_policy(
    value: &c::WakePolicy,
    limits: &w::Limits,
) -> Result<w::WakePolicy, w::Problem> {
    Ok(w::WakePolicy {
        words: wire_wake_rule(&value.words, limits)?,
        notices: wire_wake_rule(&value.notices, limits)?,
        news: wire_wake_rule(&value.news, limits)?,
        results: wire_results_wake(&value.results, limits)?,
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    })
}

pub(in crate::codec) fn wire_wake_rule(value: &c::WakeRule, _limits: &w::Limits) -> Result<w::WakeRule, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::WakeRule::Never => w::WakeRule::Never,
        c::WakeRule::Immediate => w::WakeRule::Immediate,
        c::WakeRule::Batch { count, age } => w::WakeRule::Batch(w::WakeRuleBatch { count: *(count), age: *(age) }),
        c::WakeRule::Paced { span } => w::WakeRule::Paced(w::WakeRulePaced { span: *span }),
    })
}

pub(in crate::codec) fn wire_results_wake(
    value: &c::ResultsWake,
    _limits: &w::Limits,
) -> Result<w::ResultsWake, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ResultsWake::Never => w::ResultsWake::Never,
        c::ResultsWake::Each => w::ResultsWake::Each,
        c::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
    })
}

pub(in crate::codec) fn wire_subscriptions(
    value: &c::Subscriptions,
    limits: &w::Limits,
) -> Result<w::Subscriptions, w::Problem> {
    Ok(w::Subscriptions { entries: list(value.entries.iter().map(|item| wire_subscription(item, limits)))? })
}

pub(in crate::codec) fn wire_subscription(
    value: &c::Subscription,
    limits: &w::Limits,
) -> Result<w::Subscription, w::Problem> {
    Ok(w::Subscription { number: value.number, kind: wire_subscription_kind(&value.kind, limits)? })
}

pub(in crate::codec) fn wire_subscription_kind(
    value: &c::SubscriptionKind,
    _limits: &w::Limits,
) -> Result<w::SubscriptionKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SubscriptionKind::Task { target, held, result } => {
            w::SubscriptionKind::Task(w::SubscriptionKindTask { target: *(target), held: *(held), result: *(result) })
        }
        c::SubscriptionKind::Timer { at, period } => w::SubscriptionKind::Timer(w::SubscriptionKindTimer {
            at: (at).as_nanos(),
            period: (period).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        }),
        c::SubscriptionKind::Topic { connector, topic } => {
            w::SubscriptionKind::Topic(w::SubscriptionKindTopic { connector: *(connector), topic: *(topic) })
        }
    })
}
