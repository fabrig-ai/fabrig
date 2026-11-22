use skein_lib::Duration;
use smith_domain_run::{
    Answer, CompletionEvidence, CompletionFailure, CompletionPhase, Exhausted, Fault, Limit, Policy, Refusal, Report,
    RunFailure, Spend, TranscriptRefusal,
};

use smith_domain_run::outcome::{Declared, DeclaredFailure};

use crate::{Ended, ended};

fn failed(failure: RunFailure) -> Answer {
    Answer::Failed { failure, spent: Spend::ZERO, turns: 0 }
}

#[test]
fn accepted_and_parked_complete_but_entrance_refusals_fail() {
    let accepted = Answer::Accepted {
        outcome: Declared::Failure(DeclaredFailure {
            reason: b"cannot do it".as_slice().into(),
            fields: Box::default(),
        }),
        spent: Spend::ZERO,
        turns: 1,
    };
    assert_eq!(ended(&accepted), Ended::Completed);
    assert_eq!(ended(&Answer::Parked { spent: Spend::ZERO, turns: 1 }), Ended::Completed);
    assert_eq!(ended(&Answer::Refused(Refusal::Busy)), Ended::Failed);
    for failure in [
        RunFailure::Cancelled,
        RunFailure::Stale,
        RunFailure::Transcript(TranscriptRefusal::Malformed),
        RunFailure::Budget { exhausted: Exhausted::Time, used: Some(1), bound: Some(1) },
        RunFailure::Policy(Policy::Unfinished { nudges: 1, rejected: 0 }),
    ] {
        assert_eq!(ended(&failed(failure)), Ended::Failed);
    }
}

#[test]
fn model_faults_keep_account_and_program_ends_distinct() {
    for (fault, expected) in [
        (Fault::Limited { account: 0, reset_at: None }, Ended::Limited),
        (Fault::Refused { account: 0, code: Box::default() }, Ended::Refused),
        (Fault::Unauthorized { account: 0 }, Ended::Unauthorized),
        (Fault::Untooled { name: Box::default(), detail: Box::default() }, Ended::Untooled),
        (Fault::Program { name: Box::default(), exit: None, detail: Box::default() }, Ended::Failed),
        (Fault::Provider, Ended::Failed),
        (Fault::Declined, Ended::Failed),
        (Fault::Truncated, Ended::Failed),
        (Fault::Malformed, Ended::Failed),
    ] {
        assert_eq!(ended(&failed(RunFailure::Model(fault.clone()))), expected, "{fault:?}");
    }
}

#[test]
fn only_retryable_completion_classes_end_as_outages() {
    for (failure, expected) in [
        (CompletionFailure::Overloaded, Ended::Outage),
        (CompletionFailure::Unavailable, Ended::Outage),
        (CompletionFailure::RateLimited { retry_after: Duration::from_secs(2) }, Ended::Outage),
        (CompletionFailure::TimedOut { phase: CompletionPhase::Connect }, Ended::Outage),
        (CompletionFailure::TimedOut { phase: CompletionPhase::Handshake }, Ended::Outage),
        (CompletionFailure::TimedOut { phase: CompletionPhase::Head }, Ended::Outage),
        (CompletionFailure::TimedOut { phase: CompletionPhase::Idle }, Ended::Outage),
        (CompletionFailure::TimedOut { phase: CompletionPhase::Whole }, Ended::Failed),
        (CompletionFailure::Limit { which: Limit::History, bound: 1 }, Ended::Failed),
        (CompletionFailure::ApplicationLimit, Ended::Failed),
        (CompletionFailure::Protocol, Ended::Failed),
        (CompletionFailure::Cancelled, Ended::Failed),
        (CompletionFailure::ContextTooLong, Ended::Failed),
        (CompletionFailure::Invalid, Ended::Failed),
        (CompletionFailure::Refused, Ended::Failed),
        (CompletionFailure::Unsupported, Ended::Failed),
        (CompletionFailure::Unauthorized, Ended::Unauthorized),
        (CompletionFailure::Exhausted { retry_after: None }, Ended::Failed),
    ] {
        let answer = failed(RunFailure::Model(Fault::Completion {
            failure,
            evidence: CompletionEvidence::Unknown,
            model: 0,
            attempts: 2,
            report: Report::none(),
        }));
        assert_eq!(ended(&answer), expected, "{failure:?}");
    }
}
