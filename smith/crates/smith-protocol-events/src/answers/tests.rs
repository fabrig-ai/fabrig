use super::*;

fn meta() -> AnswerMeta {
    AnswerMeta {
        now: skein_lib::Time::ZERO,
        run: 7,
        wind_down: false,
        read: Some(13),
        unread: 1,
        elapsed: Duration::from_millis(17),
    }
}

fn cases(spent: run::Spend) -> [(run::Answer, Status, Option<AnswerClass>, u64); 9] {
    [
        (run::Answer::Parked { spent, turns: 2 }, Status::Parked, None, 19),
        (run::Answer::Refused(run::Refusal::Busy), Status::Refused, Some(AnswerClass::Busy), 0),
        (
            run::Answer::Refused(run::Refusal::Invalid(run::Invalid::Workspace)),
            Status::Refused,
            Some(AnswerClass::Invalid),
            0,
        ),
        (
            run::Answer::Failed { failure: run::Failure::Cancelled, spent, turns: 2 },
            Status::Failed,
            Some(AnswerClass::Cancelled),
            19,
        ),
        (
            run::Answer::Failed { failure: run::Failure::Stale, spent, turns: 2 },
            Status::Failed,
            Some(AnswerClass::Stale),
            19,
        ),
        (
            run::Answer::Failed {
                failure: run::Failure::Budget { exhausted: run::Exhausted::Spend, used: None, bound: None },
                spent,
                turns: 2,
            },
            Status::Failed,
            Some(AnswerClass::Budget),
            19,
        ),
        (
            run::Answer::Failed {
                failure: run::Failure::Policy(run::Policy::Unfinished { nudges: 3, rejected: 5 }),
                spent,
                turns: 2,
            },
            Status::Failed,
            Some(AnswerClass::Policy),
            19,
        ),
        (
            run::Answer::Failed {
                failure: run::Failure::Transcript(run::TranscriptRefusal::Unresolved),
                spent,
                turns: 2,
            },
            Status::Failed,
            Some(AnswerClass::Transcript),
            19,
        ),
        (
            run::Answer::Failed {
                failure: run::Failure::Model(run::Fault::Completion {
                    failure: run::CompletionFailure::RateLimited { retry_after: Duration::from_millis(23) },
                    evidence: run::CompletionEvidence::Response { status: None },
                    model: 0,
                    attempts: 1,
                    report: run::Report::none(),
                }),
                spent,
                turns: 2,
            },
            Status::Failed,
            Some(AnswerClass::Model),
            19,
        ),
    ]
}

#[test]
fn every_seen_answer_kind_preserves_its_terminal_and_accounting() {
    let spent = run::Spend {
        turns: 2,
        input: Some(3),
        cache_read: Some(5),
        cache_write: Some(7),
        output: Some(11),
        reasoning: Some(2),
        units: 19,
    };
    for (seen, expected_status, expected_class, expected_spend) in cases(spent) {
        for capture in [Capture::None, Capture::Calls, Capture::Everything] {
            let projected = answer_record(meta(), &seen, capture);
            assert_eq!(projected.status, expected_status);
            assert_eq!(
                match &projected.failure {
                    Some(why) => Some(&why.class),
                    None => None,
                },
                expected_class.as_ref()
            );
            assert_eq!(projected.spent, expected_spend);
            assert_eq!(projected.usage.input_tokens, if expected_spend == 0 { None } else { Some(3) });
            assert_eq!(projected.result, None);
            assert_eq!((projected.run, projected.read, projected.unread, projected.duration_ms), (7, Some(13), 1, 17));
        }
    }
    let why = failure_record(
        run::Failure::Model(run::Fault::Completion {
            failure: run::CompletionFailure::RateLimited { retry_after: Duration::from_millis(23) },
            evidence: run::CompletionEvidence::Response { status: None },
            model: 0,
            attempts: 1,
            report: run::Report::none(),
        }),
        meta().now,
    );
    assert_eq!(
        why.completion,
        Some(CompletionFailure {
            field: None,
            code: None,
            request_id: None,
            phase: None,
            limit: None,
            status: None,
            detail: None,
            class: FailureClass::RateLimited,
            evidence: Evidence::Response,
            retry_after_ms: Some(23),
        })
    );
    assert_eq!(
        failure_record(run::Failure::Transcript(run::TranscriptRefusal::Unresolved), meta().now).reason.as_deref(),
        Some(b"unresolved".as_slice())
    );
}

#[test]
fn a_limited_accounts_reset_is_measured_at_the_answer() {
    for (reset_at, answer_at, expected) in [
        (Some(skein_lib::Time::from_nanos(100_000_000)), 20_000_000, Some(80)),
        (Some(skein_lib::Time::ZERO), 20_000_000, Some(0)),
        (None, 20_000_000, None),
        (Some(skein_lib::Time::from_nanos(100_000_000)), 60_000_000, Some(40)),
    ] {
        let mut meta = meta();
        meta.now = skein_lib::Time::from_nanos(answer_at);
        meta.elapsed = Duration::from_secs(999);
        let answer = run::Answer::Failed {
            failure: run::Failure::Model(run::Fault::Limited { account: 23, reset_at }),
            spent: run::Spend::ZERO,
            turns: 0,
        };
        let record = answer_record(meta, &answer, Capture::None);
        let failure = record.failure.expect("failed answer");
        assert_eq!((failure.account, failure.reset_ms, failure.completion), (Some(23), expected, None));
    }
}

#[test]
fn a_budget_failure_keeps_run_usage_instead_of_product_elapsed_time() {
    let answer = run::Answer::Failed {
        failure: run::Failure::Budget { exhausted: run::Exhausted::Time, used: Some(1073), bound: Some(1000) },
        spent: run::Spend::ZERO,
        turns: 0,
    };
    let record = answer_record(meta(), &answer, Capture::None);
    let failure = record.failure.expect("failed answer");
    assert_eq!(failure.which.as_deref(), Some(b"time".as_slice()));
    assert_eq!(failure.used, Some(1073));
    assert_eq!(failure.bound, Some(1000));
    assert_eq!(record.duration_ms, 17);
}
#[test]
fn program_failures_keep_their_terminal_metadata() {
    for (exit, expected) in [(None, None), (Some(7), Some(smith_events::AgentExit::Code(7)))] {
        let fault = run::Failure::Model(run::Fault::Program {
            name: b"claude_code".as_slice().into(),
            exit,
            detail: b"program detail".as_slice().into(),
        });
        let value = failure_record(fault, skein_lib::Time::ZERO);
        assert_eq!(value.class, AnswerClass::Model);
        assert_eq!(value.program.as_deref(), Some(b"claude_code".as_slice()));
        assert_eq!(value.exit, expected);
        assert_eq!(value.detail.as_deref(), Some(b"program detail".as_slice()));
    }
}
