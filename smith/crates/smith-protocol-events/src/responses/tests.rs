#![expect(
    clippy::disallowed_methods,
    reason = "programming-model.md section 10.2: deterministic unit harnesses use ordinary Rust"
)]

use super::*;
use alloc::boxed::Box;
use skein_lib::Duration;

pub(crate) fn at(millis: u64) -> Time {
    Time::ZERO.checked_add(Duration::from_millis(millis)).unwrap()
}

pub(crate) fn opened(owner: u64, thread: u64) -> ConversationOpened {
    ConversationOpened {
        run: 7,
        conversation: owner,
        thread,
        kind: smith_events::ConversationKind::Main,
        parent: None,
        call: None,
        requested: None,
        share: None,
        clamped: List::with_capacity(0),
        model: Model {
            account: None,
            backend: smith_events::Backend::Direct,
            endpoint: b"endpoint-7".to_vec().into(),
            model: b"model-7".to_vec().into(),
            window_tokens: 123,
            output_tokens: 7,
            effort: Some(b"medium".to_vec().into()),
        },
    }
}

pub(crate) use crate::fixture::prompt;

pub(crate) fn fact(millis: u64, kind: session::FactKind, turn: u64, spent: Option<u64>) -> session::Fact {
    session::Fact { at: at(millis), kind, call: None, response: Some(session::ResponseInfo { number: turn, spent }) }
}

pub(crate) fn intent(book: &mut Book, owner: Token, choice: session::Choice, retry: u32) {
    let records = book.fact(
        Time::ZERO,
        &fact(
            2,
            session::FactKind::CompletionStarted {
                window: 1,
                opener: owner,
                attempt: retry,
                messages: 0,
                max_tokens: 7,
                choice,
            },
            1,
            None,
        ),
    );
    assert!(records.is_empty());
}

#[test]
fn a_compaction_settles_the_prior_failed_response_before_its_request() {
    let mut book = Book::new(1);
    let owner = Token::new(11);
    book.opened(at(1), owner, &opened(11, 0));
    intent(&mut book, owner, session::Choice::Auto, 0);
    book.request(at(3), Time::ZERO, Token::new(99), &prompt(0));
    let failed = fact(
        5,
        session::FactKind::CompletionFailed {
            opener: owner,
            failure: llm::Failure::ContextTooLong,
            evidence: llm::Evidence::Response { status: None },
        },
        1,
        None,
    );
    assert!(book.fact(Time::ZERO, &failed).is_empty());
    let records = book.fact(
        Time::ZERO,
        &fact(6, session::FactKind::compaction_started(owner, 1, b"model-7", 123, 7, Some(b"medium")), 1, None),
    );
    let Event::ResponseCompleted(ended) = &records.get(0).expect("failure terminal").event else {
        panic!("response closes")
    };
    assert_eq!(ended.failure.as_ref().expect("failed request").class, smith_events::FailureClass::ContextTooLong);
    intent(&mut book, owner, session::Choice::Auto, 0);
    let Event::ResponseStarted(started) = book.request(at(7), Time::ZERO, Token::new(100), &prompt(0)).event else {
        panic!("summary request starts")
    };
    assert_eq!(started.response, 2);
}

#[test]
fn an_unsent_history_limit_keeps_no_earlier_attempt_or_failure() {
    let mut book = Book::new(1);
    let owner = Token::new(11);
    book.opened(at(1), owner, &opened(11, 0));
    intent(&mut book, owner, session::Choice::Auto, 0);
    book.request(at(3), Time::ZERO, Token::new(99), &prompt(0));
    let failed = session::FactKind::CompletionFailed {
        opener: owner,
        failure: llm::Failure::Overloaded,
        evidence: llm::Evidence::Unknown,
    };
    book.fact(Time::ZERO, &fact(4, failed, 1, None));
    let limit = session::End::Failed {
        detail: Box::default(),
        failure: llm::Failure::Limit { which: llm::Limit::History, bound: 2 },
        evidence: llm::Evidence::Unsent,
        attempts: 0,
    };
    let ended = session::FactKind::Ended { opener: owner, end: limit, turns: 0, usage: llm::Usage::ZERO };
    book.fact(Time::ZERO, &fact(5, ended, 1, None));
    let ended = book.main_failure.expect("the main history limit reaches its run");
    assert_eq!(ended.attempts, 0);
    assert_eq!(
        ended.failure,
        completion_failure(llm::Failure::Limit { which: llm::Limit::History, bound: 2 }, llm::Evidence::Unsent,)
    );
}

#[test]
fn revised_unsent_intent_and_retry_join_once_to_seen_hand_offs() {
    let mut book = Book::new(2);
    let owner = Token::new(11);
    book.opened(at(1), Token::new(11), &opened(11, 0));
    intent(&mut book, owner, session::Choice::Auto, 0);
    intent(&mut book, owner, session::Choice::Only, 0);
    let first = book.request(at(3), Time::ZERO, Token::new(99), &prompt(0));
    let Event::ResponseStarted(first) = first.event else {
        panic!("hand-off");
    };
    assert_eq!((first.response, first.turn, first.attempt), (1, 1, 0));
    assert_eq!(first.tool_choice, smith_events::ToolChoice::Only);
    assert!(first.prompt.is_none());
    let failed = fact(
        5,
        session::FactKind::CompletionFailed {
            opener: owner,
            failure: llm::Failure::RateLimited { retry_after: Duration::from_millis(29) },
            evidence: llm::Evidence::Response { status: None },
        },
        1,
        None,
    );
    assert!(book.fact(Time::ZERO, &failed).is_empty());
    let records = book.fact(
        Time::ZERO,
        &fact(
            6,
            session::FactKind::CompletionRetried { opener: owner, attempt: 1, delay: Duration::from_millis(31) },
            1,
            None,
        ),
    );
    let Event::ResponseCompleted(failed) = &records.get(0).unwrap().event else {
        panic!("one failure terminal");
    };
    assert_eq!((failed.response, failed.duration_ms, failed.retry_ms), (1, 2, Some(31)));
    assert_eq!(failed.failure.as_ref().unwrap().retry_after_ms, Some(29));
    assert_eq!(failed.usage, unknown_usage());
    intent(&mut book, owner, session::Choice::Only, 1);
    let second = book.request(at(37), Time::ZERO, Token::new(100), &prompt(0));
    let Event::ResponseStarted(second) = second.event else {
        panic!("retry hand-off");
    };
    assert_eq!((second.response, second.turn, second.attempt), (2, 1, 1));
    book.opened(at(38), Token::new(12), &opened(12, 1));
    intent(&mut book, Token::new(12), session::Choice::Only, 0);
    let child = book.request(at(39), Time::ZERO, Token::new(101), &prompt(1));
    let Event::ResponseStarted(child) = child.event else {
        panic!("second conversation");
    };
    assert_eq!(child.response, 3);
}

#[test]
fn internal_owner_tokens_do_not_replace_public_activation_and_conversation_numbers() {
    let mut book = Book::new(1);
    let owner = Token::new(91);
    let mut opening = opened(0, 0);
    opening.run = 77;
    book.opened(at(1), owner, &opening);
    let scope = book.scope(owner).expect("owner joined");
    assert_eq!((scope.run, scope.conversation), (77, 0));
    assert!(book.scope(Token::new(0)).is_none(), "public ordinal is not the internal owner");
    intent(&mut book, owner, session::Choice::Auto, 0);
    let record = book.request(at(3), Time::ZERO, Token::new(99), &prompt(0));
    let Event::ResponseStarted(record) = record.event else { panic!("hand-off") };
    assert_eq!((record.run, record.conversation), (77, 0));
    let completion = seen_completion();
    book.received(Token::new(99), &completion);
    drop(book.fact(
        Time::ZERO,
        &fact(
            5,
            session::FactKind::CompletionAnswered {
                opener: owner,
                stop: completion.stop,
                blocks: 2,
                calls: 2,
                invalid: 2,
            },
            1,
            None,
        ),
    ));
    let records =
        book.fact(Time::ZERO, &fact(7, session::FactKind::Used { opener: owner, usage: completion.usage }, 1, Some(3)));
    let Event::ResponseCompleted(record) = &records.as_slice()[0].event else { panic!("terminal") };
    assert_eq!((record.run, record.conversation), (77, 0));
    let records = book.fact(
        Time::ZERO,
        &fact(
            11,
            session::FactKind::Ended { opener: owner, end: session::End::Closed, turns: 1, usage: completion.usage },
            1,
            None,
        ),
    );
    let Event::ConversationClosed(record) = &records.as_slice()[0].event else { panic!("conversation end") };
    assert_eq!((record.run, record.conversation), (77, 0));
    assert!(book.scope(owner).is_none());
}

pub(crate) fn seen_completion() -> llm::Completion {
    llm::Completion {
        reasoning_dropped: Box::new([]),
        content: Box::new([
            llm::Said::ToolCall {
                id: b"oversized-call".to_vec().into(),
                name: b"tool".to_vec().into(),
                input: b"{}".to_vec().into(),
                call: llm::Decoded::Invalid { problem: llm::Problem::Oversize { bytes: 55, bound: 32 } },
                replay: None,
            },
            llm::Said::ToolCall {
                id: b"cut-call".to_vec().into(),
                name: b"tool".to_vec().into(),
                input: b"{}".to_vec().into(),
                call: llm::Decoded::Invalid { problem: llm::Problem::CutOff { bytes: 13 } },
                replay: None,
            },
        ]),
        stop: llm::Stop::ToolUse,
        usage: llm::Usage {
            input_tokens: Some(17),
            output_tokens: Some(19),
            cache_read_tokens: None,
            cache_write_tokens: Some(0),
            reasoning_tokens: Some(3),
        },
    }
}

#[test]
fn raw_usage_and_rejection_counts_survive_accounting_refusal() {
    for accepted in [false, true] {
        let mut book = Book::new(1);
        let owner = Token::new(11);
        book.opened(at(1), Token::new(11), &opened(11, 0));
        book.fact(Time::ZERO, &fact(2, session::FactKind::Opened { opener: owner }, 1, None));
        intent(&mut book, owner, session::Choice::Only, 0);
        book.request(at(3), Time::ZERO, Token::new(99), &prompt(0));
        let completion = seen_completion();
        book.received(Token::new(99), &completion);
        book.fact(
            Time::ZERO,
            &fact(
                5,
                session::FactKind::CompletionAnswered {
                    opener: owner,
                    stop: completion.stop,
                    blocks: 2,
                    calls: 2,
                    invalid: 2,
                },
                1,
                None,
            ),
        );
        let records = if accepted {
            book.fact(
                Time::ZERO,
                &fact(5, session::FactKind::Used { opener: owner, usage: completion.usage }, 1, Some(23)),
            )
        } else {
            book.fact(
                Time::ZERO,
                &fact(
                    5,
                    session::FactKind::Ended {
                        opener: owner,
                        end: session::End::PriceOverflow,
                        turns: 0,
                        usage: llm::Usage::ZERO,
                    },
                    1,
                    None,
                ),
            )
        };
        let Event::ResponseCompleted(ended) = &records.get(0).unwrap().event else {
            panic!("one typed response terminal");
        };
        assert_eq!(ended.usage, crate::reported_usage(completion.usage));
        assert_eq!((ended.oversized, ended.cut, ended.invalid), (1, 1, 2));
        assert_eq!(ended.spent, if accepted { 23 } else { 0 });
        assert!(ended.completion.is_none());
        assert!(ended.request_bytes.is_none() && ended.first_byte_ms.is_none() && ended.largest_gap_ms.is_none());
        if !accepted {
            assert_eq!(records.len(), 2);
            let Event::ConversationClosed(closed) = &records.get(1).unwrap().event else {
                panic!("then one conversation end");
            };
            assert_eq!((closed.turns, closed.spent), (0, 0));
            assert_eq!(closed.end, ConversationEnd::Overflow);
            assert_eq!(closed.usage, ended.usage, "a reported terminal belongs to its conversation total");
        }
    }
}

#[test]
fn cancellations_and_pre_session_refusals_have_one_terminal_right() {
    for sent in [false, true] {
        let mut book = Book::new(1);
        let owner = Token::new(11);
        book.opened(at(1), Token::new(11), &opened(11, 0));
        intent(&mut book, owner, session::Choice::Only, 0);
        if sent {
            book.request(at(3), Time::ZERO, Token::new(99), &prompt(0));
        }
        let cancelled =
            book.fact(Time::ZERO, &fact(5, session::FactKind::CompletionCancelled { opener: owner }, 1, None));
        assert_eq!(cancelled.len(), u32::from(sent));
        if sent {
            let Event::ResponseCompleted(cancelled) = &cancelled.get(0).unwrap().event else {
                panic!("one cancellation terminal");
            };
            assert_eq!(cancelled.outcome, Outcome::Cancelled);
            assert_eq!(cancelled.usage, unknown_usage());
        }
    }
    let mut book = Book::new(1);
    let owner = Token::new(11);
    book.opened(at(1), Token::new(11), &opened(11, 0));
    book.root_ended(at(5), owner, run::facts::EndKind::Invalid);
    let Event::ConversationClosed(refused) = book.facts_drained(Time::ZERO).unwrap().event else {
        panic!("refusal closes opening");
    };
    assert_eq!(refused.end, ConversationEnd::Refused);
    assert_eq!((refused.turns, refused.spent), (0, 0));
    assert!(book.facts_drained(Time::ZERO).is_none());
    assert!(
        book.fact(
            Time::ZERO,
            &fact(
                5,
                session::FactKind::Ended {
                    opener: owner,
                    end: session::End::Invalid,
                    turns: 0,
                    usage: llm::Usage::ZERO
                },
                1,
                None
            )
        )
        .is_empty()
    );
    // The same capacity can now admit another refused opening.
    book.opened(at(6), Token::new(12), &opened(12, 1));
    book.root_ended(at(7), Token::new(12), run::facts::EndKind::Busy);
    assert!(book.facts_drained(Time::ZERO).is_some());
}
#[test]
fn observed_transport_values_survive_each_terminal_and_absent_fields_stay_unknown() {
    let sources = [
        smith_protocol_llm::Measures::none(),
        smith_protocol_llm::Measures {
            request_bytes: Some(u64::from(u32::MAX) + 1),
            first_byte: Some(Duration::ZERO),
            largest_gap: Some(Duration::from_nanos(7_999_999)),
        },
        smith_protocol_llm::Measures { request_bytes: Some(91), first_byte: None, largest_gap: None },
    ];
    for outcome in [Outcome::Completed, Outcome::Failed, Outcome::Cancelled] {
        for measures in sources {
            let mut book = Book::new(1);
            let owner = Token::new(11);
            let request = Token::new(99);
            book.opened(at(1), owner, &opened(11, 0));
            intent(&mut book, owner, session::Choice::Only, 0);
            book.request(at(3), Time::ZERO, request, &prompt(0));
            book.measured(request, measures);
            let records = match outcome {
                Outcome::Completed => {
                    let completion = seen_completion();
                    book.received(request, &completion);
                    book.fact(
                        Time::ZERO,
                        &fact(
                            4,
                            session::FactKind::CompletionAnswered {
                                opener: owner,
                                stop: completion.stop,
                                blocks: 2,
                                calls: 2,
                                invalid: 2,
                            },
                            1,
                            None,
                        ),
                    );
                    book.fact(
                        Time::ZERO,
                        &fact(5, session::FactKind::Used { opener: owner, usage: completion.usage }, 1, Some(23)),
                    )
                }
                Outcome::Failed => {
                    book.fact(
                        Time::ZERO,
                        &fact(
                            4,
                            session::FactKind::CompletionFailed {
                                opener: owner,
                                failure: llm::Failure::Unavailable,
                                evidence: llm::Evidence::Unknown,
                            },
                            1,
                            None,
                        ),
                    );
                    book.fact(
                        Time::ZERO,
                        &fact(
                            5,
                            session::FactKind::CompletionRetried {
                                opener: owner,
                                attempt: 1,
                                delay: Duration::from_millis(9),
                            },
                            1,
                            None,
                        ),
                    )
                }
                Outcome::Cancelled => {
                    book.fact(Time::ZERO, &fact(5, session::FactKind::CompletionCancelled { opener: owner }, 1, None))
                }
                Outcome::Unknown(_) => unreachable!("the fixture names the three terminals"),
            };
            let Event::ResponseCompleted(response) = &records.as_slice()[0].event else {
                panic!("one response terminal");
            };
            assert_eq!(records.len(), 1);
            assert_eq!(response.outcome, outcome);
            assert_eq!(response.request_bytes, measures.request_bytes);
            assert_eq!(response.first_byte_ms, measures.first_byte.map(|duration| duration.as_millis()));
            assert_eq!(response.largest_gap_ms, measures.largest_gap.map(|duration| duration.as_millis()));
        }
    }
}

#[test]
fn conversation_totals_keep_each_reported_counter_and_no_report_remains_unknown() {
    for reported in [false, true] {
        let mut book = Book::new(1);
        let owner = Token::new(11);
        book.opened(at(1), owner, &opened(11, 0));
        let sources = [
            llm::Usage {
                input_tokens: Some(3),
                output_tokens: None,
                cache_read_tokens: Some(0),
                cache_write_tokens: None,
                reasoning_tokens: None,
            },
            llm::Usage {
                input_tokens: None,
                output_tokens: Some(5),
                cache_read_tokens: None,
                cache_write_tokens: Some(2),
                reasoning_tokens: Some(1),
            },
        ];
        if reported {
            for (index, usage) in sources.into_iter().enumerate() {
                let request = Token::new(u64::try_from(index).unwrap() + 99);
                intent(&mut book, owner, session::Choice::Auto, 0);
                book.request(at(3), Time::ZERO, request, &prompt(0));
                let completion = llm::Completion {
                    content: Box::new([]),
                    stop: llm::Stop::EndTurn,
                    usage,
                    reasoning_dropped: Box::new([]),
                };
                book.received(request, &completion);
                book.fact(
                    Time::ZERO,
                    &fact(
                        4,
                        session::FactKind::CompletionAnswered {
                            opener: owner,
                            stop: completion.stop,
                            blocks: 0,
                            calls: 0,
                            invalid: 0,
                        },
                        1,
                        None,
                    ),
                );
                let terminal =
                    book.fact(Time::ZERO, &fact(5, session::FactKind::Used { opener: owner, usage }, 1, Some(7)));
                let Event::ResponseCompleted(seen) = &terminal.get(0).unwrap().event else { panic!("provider report") };
                assert_eq!(seen.usage, crate::reported_usage(usage));
            }
        }
        let records = book.fact(
            Time::ZERO,
            &fact(
                9,
                session::FactKind::Ended {
                    opener: owner,
                    end: session::End::Closed,
                    turns: if reported { 2 } else { 0 },
                    usage: llm::Usage::NONE,
                },
                1,
                None,
            ),
        );
        let Event::ConversationClosed(ended) = &records.get(0).unwrap().event else { panic!("one conversation end") };
        assert_eq!(ended.spent, if reported { 14 } else { 0 });
        assert_eq!(
            ended.usage,
            if reported {
                Usage {
                    input_tokens: Some(3),
                    output_tokens: Some(5),
                    cache_read_tokens: Some(0),
                    cache_write_tokens: Some(2),
                    reasoning_tokens: Some(1),
                }
            } else {
                unknown_usage()
            }
        );
    }
}

fn fail_attempt(book: &mut Book, owner: Token, retry: u32, capture: crate::sinks::Capture) {
    let request = Token::new(u64::from(retry) + 99);
    let (status, detail) = if retry == 0 { (429, b"first".as_slice()) } else { (404, b"last".as_slice()) };

    intent(book, owner, session::Choice::Auto, retry);
    book.request(at(3), Time::ZERO, request, &prompt(0));
    let failure = if retry == 0 {
        llm::Failure::RateLimited { retry_after: Duration::from_millis(29) }
    } else {
        llm::Failure::Invalid
    };
    book.failed(
        request,
        failure,
        llm::Evidence::Response { status: Some(status) },
        detail,
        capture,
        &llm::Report::none(),
    );
    assert!(
        book.fact(
            Time::ZERO,
            &fact(
                5,
                session::FactKind::CompletionFailed {
                    opener: owner,
                    failure,
                    evidence: llm::Evidence::Response { status: None }
                },
                1,
                None
            )
        )
        .is_empty()
    );
    if retry == 0 {
        let records = book.fact(
            Time::ZERO,
            &fact(
                6,
                session::FactKind::CompletionRetried { opener: owner, attempt: 1, delay: Duration::from_millis(31) },
                1,
                None,
            ),
        );
        let Event::ResponseCompleted(first) = &records.get(0).unwrap().event else { panic!("retry terminal") };
        let observed = first.failure.as_ref().unwrap();
        assert_eq!(observed.status, Some(429));
        assert_eq!(observed.retry_after_ms, Some(29));
        assert_eq!(first.retry_ms, Some(31));
    }
}

fn unlocated_failure(completion: CompletionFailure) -> smith_events::AnswerFailure {
    smith_events::AnswerFailure {
        state: None,
        code: None,
        program: None,
        exit: None,
        detail: None,
        reset_ms: None,
        used: None,
        bound: None,
        dropped: None,
        ended: None,
        turn: None,
        class: smith_events::AnswerClass::Model,
        endpoint: None,
        model: None,
        attempts: None,
        completion: Some(completion),
        account: None,
        which: None,
        reason: None,
    }
}

#[test]
fn seen_failure_joins_keep_retry_identity_and_child_diagnostics_out_of_the_run() {
    for kind in [smith_events::ConversationKind::Main, smith_events::ConversationKind::Child] {
        for capture in [crate::sinks::Capture::None, crate::sinks::Capture::Calls, crate::sinks::Capture::Everything] {
            let mut book = Book::new(1);
            let owner = Token::new(11);
            let mut opening = opened(11, 0);
            opening.kind = kind.clone();
            book.opened(at(1), owner, &opening);
            for retry in [0, 1] {
                fail_attempt(&mut book, owner, retry, capture);
            }
            let records = book.fact(
                Time::ZERO,
                &fact(
                    7,
                    session::FactKind::Ended {
                        opener: owner,
                        end: session::End::Failed {
                            detail: Box::default(),
                            failure: llm::Failure::Invalid,
                            evidence: llm::Evidence::Response { status: None },
                            attempts: 1,
                        },
                        turns: 0,
                        usage: llm::Usage::NONE,
                    },
                    1,
                    None,
                ),
            );
            let Event::ResponseCompleted(last) = &records.get(0).unwrap().event else { panic!("last response") };
            let Event::ConversationClosed(closed) = &records.get(1).unwrap().event else { panic!("conversation") };
            assert_eq!(last.failure, closed.failure);
            let observed = last.failure.as_ref().unwrap();
            assert_eq!(observed.status, Some(404));
            assert_eq!(observed.retry_after_ms, None);
            assert_eq!(
                observed.detail.as_deref(),
                if capture == crate::sinks::Capture::None { None } else { Some(b"last".as_slice()) }
            );
            let mut run = smith_events::RunCompleted {
                run: 7,
                wind_down: false,
                status: smith_events::Status::Failed,
                failure: Some(unlocated_failure(completion_failure(
                    llm::Failure::Invalid,
                    llm::Evidence::Response { status: None },
                ))),
                result: None,
                turns: 0,
                spent: 0,
                usage: unknown_usage(),
                read: None,
                unread: 0,
                duration_ms: 7,
            };
            book.answer_failure(&mut run);
            let answer = run.failure.as_ref().unwrap();
            if kind == smith_events::ConversationKind::Main {
                assert_eq!(answer.completion, last.failure);
                assert_eq!(answer.endpoint, Some(opening.model.endpoint.clone()));
                assert_eq!(answer.model, Some(opening.model.model.clone()));
                assert_eq!(answer.attempts, Some(2));
            } else {
                assert_eq!(answer.completion.as_ref().unwrap().status, None);
                assert_eq!(answer.endpoint, None);
                assert_eq!(answer.attempts, None);
            }
        }
    }
    for phase in [llm::Phase::Connect, llm::Phase::Handshake, llm::Phase::Head, llm::Phase::Idle, llm::Phase::Whole] {
        let failure = completion_failure(llm::Failure::TimedOut { phase }, llm::Evidence::Unknown);
        assert!(failure.phase.is_some());
        assert_eq!(failure.status, None);
        assert_eq!(failure.detail, None);
    }
    let failure = completion_failure(
        llm::Failure::Limit { which: llm::Limit::Path(llm::Cap::ResponseHead), bound: 1935 },
        llm::Evidence::Unsent,
    );
    assert_eq!(
        failure.limit,
        Some(smith_events::FailureLimit { which: b"response_head".as_slice().into(), bound: 1935 })
    );
    assert_eq!(failure.status, None);
}
