//! Provider recovery, retry policy and deadlines.
//! Contract: smith/docs/design/domain/session.md, sections 3, 4 and 6.

use super::*;

#[test]
fn time_expires_while_a_session_rests_between_batches() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let away = Call::Read { path: outside(), offset: 1, window: tools::Budget::Default };
    let next = Call::Write { path: outside(), content: bytes(b"x") };
    let content = Box::new([tool_call(b"first", away), tool_call(b"next", next)]);
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }), None);
    h.after(BUDGET.time);
    h.domain.reclaim();
    assert_eq!(h.resume(), Some(ended(OUT_OF_TIME, 1)));
    assert_eq!(h.turn_records.len(), 1);
}

#[test]
fn transient_failures_are_retried_after_a_backoff_until_the_retries_run_out() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    for _ in 0..LIMITS.retries {
        assert_eq!(
            h.step(Event::Failed {
                retry_ok: true,
                owner,
                failure: Failure::Overloaded,
                evidence: crate::llm::Evidence::Unknown,
                detail: Box::default()
            }),
            None
        );
        let retry = h.domain.next_deadline().expect("a retry is armed");
        assert!(retry <= h.env.now.saturating_add(LIMITS.backoff_max), "the backoff is capped");
        h.env.now = retry;
        drop(calling(h.fire()));
    }
    let end = h.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::Overloaded,
        evidence: crate::llm::Evidence::Unknown,
        detail: Box::default(),
    });
    assert_eq!(
        end,
        Some(ended(
            End::Failed {
                detail: Box::default(),
                failure: Failure::Overloaded,
                evidence: crate::llm::Evidence::Unknown,
                attempts: LIMITS.retries + 1
            },
            0
        ))
    );
}

#[test]
fn a_rate_limit_is_waited_out_and_lasting_failures_are_not_retried() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(2);
    let failure = Failure::RateLimited { retry_after: Duration::from_secs(5) };
    assert_eq!(
        h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default()
        }),
        None
    );
    assert_eq!(h.domain.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(5))));

    for failure in [Failure::Invalid, Failure::Refused, Failure::Unsupported] {
        let mut h = Harness::new(&LIMITS);
        let (owner, _) = h.open(1);
        let terminal = h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default(),
        });
        assert_eq!(
            terminal,
            Some(ended(
                End::Failed { detail: Box::default(), failure, evidence: crate::llm::Evidence::Unknown, attempts: 1 },
                0
            )),
            "{failure:?}"
        );
    }
}

#[test]
fn each_attempt_keeps_model_progress_and_the_same_session_expiry() {
    let mut h = Harness::new(&LIMITS);
    h.after(Duration::from_secs(7));
    let progress = crate::Progress { head: Duration::from_secs(11), idle: Duration::from_secs(13) };
    let request = Spec { progress, budget: Budget { time: Duration::from_secs(90), ..BUDGET }, ..spec() };
    step(&mut h.domain, &h.env, Event::Open { opener: Token::new(1), opening: opening(request) }, &mut h.out);
    let Some(Request::Opened { .. }) = h.next() else {
        panic!("opened session");
    };
    let Some(Request::Complete { owner, deadlines, .. }) = h.one() else {
        panic!("first completion");
    };
    assert_eq!(
        deadlines,
        crate::Deadlines {
            head: progress.head,
            idle: progress.idle,
            whole: Time::ZERO.saturating_add(Duration::from_secs(97)),
        }
    );
    assert_eq!(
        h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::Overloaded,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default(),
        }),
        None
    );
    h.env.now = h.domain.next_deadline().expect("retry before expiry");
    let Some(Request::Complete { deadlines: retried, .. }) = h.fire() else {
        panic!("retry completion");
    };
    assert_eq!(retried, deadlines, "progress repeats but whole never restarts per attempt");
}

#[test]
fn a_timeout_at_session_expiry_ends_for_time_once_without_retry() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    h.after(BUDGET.time);
    assert_eq!(
        h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::TimedOut { phase: crate::llm::CompletionPhase::Whole },
            evidence: crate::llm::Evidence::Response { status: None },
            detail: Box::default(),
        }),
        Some(ended(OUT_OF_TIME, 0))
    );
    assert_eq!(h.domain.next_deadline(), None, "no retry or expiry remains");
    assert_eq!(h.step(Event::Close { session: owner }), None, "the closed session has no further terminal");
}

#[test]
fn each_transient_timeout_phase_retries_with_the_same_absolute_expiry() {
    for phase in [
        crate::llm::CompletionPhase::Connect,
        crate::llm::CompletionPhase::Handshake,
        crate::llm::CompletionPhase::Head,
        crate::llm::CompletionPhase::Idle,
    ] {
        let mut h = Harness::new(&LIMITS);
        let (owner, _) = h.open(1);
        assert_eq!(
            h.step(Event::Failed {
                retry_ok: true,
                owner,
                failure: Failure::TimedOut { phase },
                evidence: crate::llm::Evidence::Unknown,
                detail: Box::default()
            }),
            None
        );
        h.env.now = h.domain.next_deadline().expect("transient retry is armed");
        let Some(Request::Complete { deadlines, .. }) = h.fire() else { panic!("retry for {phase:?}") };
        assert_eq!(deadlines.whole, Time::ZERO.saturating_add(BUDGET.time));
        let end = h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::TimedOut { phase: crate::llm::CompletionPhase::Whole },
            evidence: crate::llm::Evidence::Response { status: None },
            detail: Box::default(),
        });
        assert_eq!(
            end,
            Some(ended(OUT_OF_TIME, 0)),
            "Whole never spends another retry, even before the parent clock catches up"
        );
        assert_eq!(h.domain.next_deadline(), None);
    }
}

#[test]
fn whole_expiry_or_elapsed_session_time_ends_once_even_with_no_retries() {
    for phase in [
        crate::llm::CompletionPhase::Connect,
        crate::llm::CompletionPhase::Handshake,
        crate::llm::CompletionPhase::Head,
        crate::llm::CompletionPhase::Idle,
        crate::llm::CompletionPhase::Whole,
    ] {
        let mut h = Harness::new(&Limits { retries: 0, ..LIMITS });
        let (owner, _) = h.open(1);
        if phase != crate::llm::CompletionPhase::Whole {
            h.after(BUDGET.time);
        }
        assert_eq!(
            h.step(Event::Failed {
                retry_ok: true,
                owner,
                failure: Failure::TimedOut { phase },
                evidence: crate::llm::Evidence::Unknown,
                detail: Box::default()
            }),
            Some(ended(OUT_OF_TIME, 0))
        );
        assert_eq!(h.domain.next_deadline(), None);
        assert_eq!(h.step(Event::Close { session: owner }), None);
    }
}

#[test]
fn an_expiring_session_in_backoff_or_yielded_ends_at_once() {
    let mut h = Harness::new(&Limits {
        backoff_base: Duration::from_secs(3600),
        backoff_max: Duration::from_secs(3600),
        ..LIMITS
    });
    let (owner, _) = h.open(1);
    assert_eq!(
        h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::Unavailable,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default()
        }),
        None
    );
    h.after(BUDGET.time);
    assert_eq!(h.fire(), Some(ended(OUT_OF_TIME, 0)));
    assert_eq!(h.domain.next_deadline(), None);

    let mut h = Harness::new(&LIMITS);
    let _session: Token = h.yielded();
    h.after(BUDGET.time);
    assert_eq!(h.fire(), Some(ended(OUT_OF_TIME, 1)));
    assert_eq!(h.domain.next_deadline(), None);
}

#[test]
fn a_late_oversized_completion_keeps_its_retry_attempt_count() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    assert!(
        h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::Overloaded,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default(),
        })
        .is_none()
    );
    h.env.now = h.domain.next_deadline().expect("retry is armed");
    drop(calling(h.fire()));
    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::Cancel { owner }));
    let completion = completion(Box::new([text(&[b'x'; 4097])]), Stop::EndTurn);
    assert_eq!(
        h.step(Event::Completed { owner, completion }),
        Some(ended(
            End::Failed {
                detail: Box::default(),
                failure: Failure::Limit { which: crate::llm::Limit::Path(crate::llm::Cap::Answer), bound: 4096 },
                evidence: crate::llm::Evidence::Response { status: None },
                attempts: 2,
            },
            1
        ))
    );
}

#[test]
fn retries_cancels_and_refusals_are_told_too() {
    let mut h = Harness::new(&Limits { sessions: 1, ..LIMITS });
    let opener = Token::new(1);
    let (owner, _) = h.open(1);
    assert_eq!(
        h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::Overloaded,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default()
        }),
        None
    );
    let retry = h.domain.next_deadline().expect("a retry is armed");
    let started = Fact::CompletionStarted {
        window: 1,
        opener,
        attempt: 0,
        messages: 1,
        max_tokens: 1024,
        choice: crate::facts::Choice::Auto,
    };
    let delay = retry.saturating_since(Time::ZERO);
    h.told(&[
        Fact::Opened { opener },
        started,
        Fact::Tools { opener, fact: tools::FactKind::Opened { session: owner } },
        Fact::CompletionFailed { opener, failure: Failure::Overloaded, evidence: crate::llm::Evidence::Unknown },
        Fact::CompletionRetried { opener, attempt: 1, delay },
    ]);
    h.env.now = retry;
    drop(calling(h.fire()));
    h.told(&[Fact::CompletionStarted {
        window: 1,
        opener,
        attempt: 1,
        messages: 1,
        max_tokens: 1024,
        choice: crate::facts::Choice::Auto,
    }]);

    let refused = Token::new(2);
    drop(h.step(Event::Open { opener: refused, opening: opening(spec()) }));
    h.told(&[Fact::Ended { opener: refused, end: End::Busy, turns: 0, usage: Usage::ZERO }]);

    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::Cancel { owner }));
    h.told(&[Fact::Tools { opener, fact: tools::FactKind::Closed { session: owner } }]);
    drop(h.step(Event::Cancelled { owner }));
    let end = Fact::Ended { opener, end: End::Closed, turns: 0, usage: Usage::ZERO };
    h.told(&[Fact::CompletionCancelled { opener }, end]);
}

#[test]
fn a_childs_final_note_follows_settled_results_and_is_not_repeated_on_retry() {
    let mut harness = Harness::new(&LIMITS);
    let mut selected = spec();
    selected.budget.turns = 2;
    selected.final_turn = Some(crate::FinalTurn { note: bytes(b"Answer now."), allowance: Duration::from_secs(1) });
    let (owner, first) = harness.open_with(1, selected);
    assert_eq!(first.choice, crate::llm::ToolChoice::Auto);
    let (io, _) = running(harness.step(Event::Completed { owner, completion: reading() }));
    let (_, final_prompt) = calling(harness.step(ran(io, b"main.rs")));
    assert_eq!(final_prompt.choice, crate::llm::ToolChoice::None);
    let messages = &final_prompt.messages;
    let Block::ToolResult { id, .. } = &messages[messages.len() - 2].content[0] else {
        panic!("the latest read result precedes the note");
    };
    assert_eq!(id.as_ref(), b"c1");
    assert_eq!(messages.last().unwrap().content.as_ref(), &[text(b"Answer now.")]);
    assert_eq!(
        harness.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::Overloaded,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default(),
        }),
        None
    );
    harness.env.now = harness.domain.next_deadline().expect("retry wake");
    let (_, retry) = calling(harness.fire());
    assert_eq!(retry.messages, final_prompt.messages, "retry preserves exactly one note after the results");
    assert_eq!(retry.choice, crate::llm::ToolChoice::None);
    assert_eq!(yielded(harness.step(Event::Completed { owner, completion: done() })).1, Yield::Budget);
    assert_eq!(harness.turns, 2, "one failed attempt spends no turn");
}

#[test]
fn a_native_request_limit_is_context_too_long_without_retry() {
    use crate::llm::{Cap, Limit};
    for cap in [Cap::Request, Cap::Retained] {
        let mut harness = Harness::new(&LIMITS);
        let (owner, _) = harness.open(1);
        let failure = Failure::Limit { which: Limit::Path(cap), bound: 8123 };
        let expected =
            if cap == Cap::Request { Failure::Limit { which: Limit::Window, bound: 65_536 } } else { failure };
        assert_eq!(
            harness.step(Event::Failed {
                retry_ok: true,
                owner,
                failure,
                evidence: crate::llm::Evidence::Unsent,
                detail: Box::default(),
            }),
            Some(ended(
                End::Failed {
                    detail: Box::default(),
                    failure: expected,
                    evidence: crate::llm::Evidence::Unsent,
                    attempts: 1
                },
                0
            ))
        );
        assert_eq!(harness.domain.next_deadline(), None);
    }
}

#[test]
fn an_unauthorized_call_is_retried_only_for_an_account() {
    use crate::llm::Evidence;
    let failure = Failure::Unauthorized;
    let response = Evidence::Response { status: Some(401) };
    for (account, retry_ok, evidence, terminal) in [
        (Some(0), true, Evidence::Unknown, false),
        (Some(0), false, Evidence::Unknown, false),
        (Some(0), false, Evidence::Unsent, false),
        (Some(0), true, response, false),
        (Some(0), false, response, true),
        (Some(0), false, Evidence::Response { status: None }, true),
        (None, true, Evidence::Unknown, true),
        (None, false, Evidence::Unknown, true),
        (None, true, response, true),
        (None, false, Evidence::Unsent, true),
    ] {
        let mut harness = Harness::new(&LIMITS);
        let (owner, _) = harness.open_with(1, Spec { account, ..spec() });
        let request = harness.step(Event::Failed { owner, failure, evidence, detail: Box::default(), retry_ok });
        let expected = if terminal {
            Some(ended(End::Failed { detail: Box::default(), failure, evidence, attempts: 1 }, 0))
        } else {
            None
        };
        assert_eq!(request, expected, "account {account:?}, retry_ok {retry_ok}, evidence {evidence:?}");
    }
}
