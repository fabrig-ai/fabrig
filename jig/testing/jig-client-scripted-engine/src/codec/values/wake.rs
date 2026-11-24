//! Wire projections of owner wake facts (client/domain/testing.md, 4).
use super::super::{c, list, w};
pub(in crate::codec) fn wire_wake_policy(value: &c::WakePolicy) -> w::WakePolicy {
    w::WakePolicy {
        words: wire_wake_rule(&value.words),
        notices: wire_wake_rule(&value.notices),
        news: wire_wake_rule(&value.news),
        results: wire_results_wake(&value.results),
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
}

pub(in crate::codec) fn wire_wake_rule(value: &c::WakeRule) -> w::WakeRule {
    match value {
        c::WakeRule::Never => w::WakeRule::Never,
        c::WakeRule::Immediate => w::WakeRule::Immediate,
        c::WakeRule::Batch { count, age } => w::WakeRule::Batch(w::WakeRuleBatch { count: *(count), age: *(age) }),
        c::WakeRule::Paced { span } => w::WakeRule::Paced(w::WakeRulePaced { span: *span }),
    }
}

pub(in crate::codec) fn wire_results_wake(value: &c::ResultsWake) -> w::ResultsWake {
    match value {
        c::ResultsWake::Never => w::ResultsWake::Never,
        c::ResultsWake::Each => w::ResultsWake::Each,
        c::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
    }
}

pub(in crate::codec) fn wire_subscriptions(value: &c::Subscriptions) -> w::Subscriptions {
    w::Subscriptions { entries: list(value.entries.iter().map(wire_subscription)) }
}

pub(in crate::codec) fn wire_subscription(value: &c::Subscription) -> w::Subscription {
    w::Subscription { number: value.number, kind: wire_subscription_kind(&value.kind) }
}

pub(in crate::codec) fn wire_subscription_kind(value: &c::SubscriptionKind) -> w::SubscriptionKind {
    match value {
        c::SubscriptionKind::Task { target, held, result } => {
            w::SubscriptionKind::Task(w::SubscriptionKindTask { target: *(target), held: *(held), result: *(result) })
        }
        c::SubscriptionKind::Timer { at, period } => {
            w::SubscriptionKind::Timer(w::SubscriptionKindTimer { at: (at).as_nanos(), period: *period })
        }
        c::SubscriptionKind::Topic { connector, topic } => {
            w::SubscriptionKind::Topic(w::SubscriptionKindTopic { connector: *(connector), topic: *(topic) })
        }
    }
}
