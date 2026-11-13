//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v21 as w;

pub(crate) fn wake_rule(value: &w::WakeRule) -> c::WakeRule {
    match value {
        w::WakeRule::Never => c::WakeRule::Never,
        w::WakeRule::Immediate => c::WakeRule::Immediate,
        w::WakeRule::Batch(row) => c::WakeRule::Batch { count: row.count(), age: row.age() },
        w::WakeRule::Paced(row) => c::WakeRule::Paced { span: row.span() },
    }
}

pub(crate) fn results_wake(value: &w::ResultsWake) -> c::ResultsWake {
    match value {
        w::ResultsWake::Never => c::ResultsWake::Never,
        w::ResultsWake::Each => c::ResultsWake::Each,
        w::ResultsWake::LastOrFailure => c::ResultsWake::LastOrFailure,
    }
}

pub(crate) fn wake_policy(value: &w::WakePolicy) -> c::WakePolicy {
    c::WakePolicy {
        words: wake_rule(value.words()),
        notices: wake_rule(value.notices()),
        news: wake_rule(value.news()),
        results: results_wake(value.results()),
        questions: value.questions(),
        answers: value.answers(),
        timers: value.timers(),
    }
}
