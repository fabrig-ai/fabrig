use skein_lib::Duration;
use smith_domain_run::{
    Answer, CompletionEvidence, CompletionFailure, CompletionPhase, Exhausted, Fault, Limit, Policy, Refusal, Report,
    RunFailure, Spend, TranscriptRefusal,
};

use smith_domain_run::outcome::{Declared, DeclaredFailure};

use crate::{Ended, ended, ended_channel};

#[test]
fn channel_answers_keep_their_domain_twins_classification() {
    let mut answers = vec![
        Answer::Accepted {
            outcome: Declared::Failure(DeclaredFailure { reason: b"done".as_slice().into(), fields: Box::default() }),
            spent: Spend::ZERO,
            turns: 1,
        },
        Answer::Parked { spent: Spend::ZERO, turns: 1 },
        Answer::Refused(Refusal::Busy),
    ];
    answers.extend(
        [
            RunFailure::Cancelled,
            RunFailure::Stale,
            RunFailure::Transcript(TranscriptRefusal::Malformed),
            RunFailure::Budget { exhausted: Exhausted::Time, used: None, bound: None },
            RunFailure::Policy(Policy::Unfinished { nudges: 1, rejected: 0 }),
        ]
        .into_iter()
        .map(failed),
    );
    answers.extend(
        [
            Fault::Limited { account: 0, reset_at: None },
            Fault::Refused { account: 0, code: Box::default() },
            Fault::Unauthorized { account: 0 },
            Fault::Untooled { name: Box::default(), detail: Box::default() },
            Fault::Program { name: Box::default(), exit: None, detail: Box::default() },
            Fault::Provider,
            Fault::Declined,
            Fault::Truncated,
            Fault::Malformed,
        ]
        .into_iter()
        .map(|fault| failed(RunFailure::Model(fault))),
    );
    answers.extend(
        [
            CompletionFailure::Overloaded,
            CompletionFailure::Unavailable,
            CompletionFailure::RateLimited { retry_after: Duration::from_secs(2) },
            CompletionFailure::TimedOut { phase: CompletionPhase::Connect },
            CompletionFailure::TimedOut { phase: CompletionPhase::Handshake },
            CompletionFailure::TimedOut { phase: CompletionPhase::Head },
            CompletionFailure::TimedOut { phase: CompletionPhase::Idle },
            CompletionFailure::TimedOut { phase: CompletionPhase::Whole },
            CompletionFailure::Limit { which: Limit::History, bound: 1 },
            CompletionFailure::ApplicationLimit,
            CompletionFailure::Protocol,
            CompletionFailure::Cancelled,
            CompletionFailure::ContextTooLong,
            CompletionFailure::Invalid,
            CompletionFailure::Refused,
            CompletionFailure::Unsupported,
            CompletionFailure::Unauthorized,
            CompletionFailure::Exhausted { retry_after: None },
        ]
        .into_iter()
        .map(|failure| {
            failed(RunFailure::Model(Fault::Completion {
                failure,
                evidence: CompletionEvidence::Unknown,
                model: 0,
                attempts: 2,
                report: Report::none(),
            }))
        }),
    );
    for answer in answers {
        let expected = ended(&answer);
        let description = format!("{answer:?}");
        let channel = smith_protocol_translate::answer_record(
            skein_lib::Time::ZERO,
            answer,
            None,
            &smith_channel::CEILINGS,
            &smith_charter::CEILINGS,
        )
        .expect("bounded answer");
        assert_eq!(ended_channel(&channel), expected, "{description}");
    }
}

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
