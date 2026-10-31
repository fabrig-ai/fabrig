//! Event projection scenarios with bounded typed inputs.

#![expect(
    clippy::disallowed_macros,
    reason = "programming-model.md, section 10.2: deterministic test harness uses ordinary Rust"
)]

use alloc::boxed::Box;

pub(crate) fn priced_charter() -> smith_domain::run::Charter {
    use skein_lib::Duration;
    use smith_domain::run;
    run::Charter {
        mode: run::Mode::Finite,
        reserve: run::charter::Reserve { turns: 0, time: Duration::ZERO },
        instructions: Box::default(),
        brief: run::Brief {
            sections: Box::new([run::Section { title: Box::default(), text: b"Review the change.".as_slice().into() }]),
        },
        conventions: None,
        resume: false,
        waiting: Duration::from_secs(300),
        grants: run::charter::Grants {
            wait: true,
            deliver: None,
            agents: false,
            host_tools: Box::default(),
            tools: run::charter::Tools { inspect: false, modify: false, shell: false },
        },
        outcome: run::outcome::OutcomeSpec {
            change: None,
            verdicts: Box::default(),
            failure: None,
            report: Some(run::outcome::TextSpec { max: 4096, fields: Box::default() }),
        },
        budget: run::Budget { turns: 512, spend: 25_000_000, time: Duration::from_secs(3600) },
        llm: run::charter::Llm {
            effort: None,
            prices: run::Prices {
                input: 1_250_000,
                cached: 125_000,
                cache_write: None,
                output: 10_000_000,
                unit: 1_000_000,
            },
            account: Some(0),
            endpoint: run::charter::Endpoint(1),
            model: b"gpt-5".as_slice().into(),
            window: 2_097_152,
            output: 65_536,
            dialect: 1,
        },
        models: Box::default(),
    }
}

fn stream_limits() -> smith_events::Limits {
    // Reuse the sink fixture's record bounds; codecs need no IO destinations.
    crate::sinks::tests::limits(0).records
}

fn assert_stream_record(seen: &smith_events::Record, literal: &[u8]) {
    let limits = stream_limits();
    let expected = smith_events::read(literal, &limits).unwrap().unwrap();
    assert_eq!(*seen, expected);
}

fn compaction_intent(book: &mut crate::responses::Book, owner: skein_lib::Token, window: u32) {
    use crate::responses::tests::fact;
    use smith_domain::session;
    assert!(
        book.fact(
            skein_lib::Time::ZERO,
            &fact(
                3,
                session::FactKind::CompletionStarted {
                    opener: owner,
                    window,
                    attempt: 0,
                    messages: 0,
                    max_tokens: 7,
                    choice: session::Choice::Auto,
                },
                3,
                None
            )
        )
        .is_empty()
    );
}

fn compaction_answer(book: &mut crate::responses::Book, owner: skein_lib::Token, stop: smith_domain::llm::Stop) {
    use crate::responses::tests::fact;
    use smith_domain::{llm, session};
    let usage = llm::Usage { input_tokens: Some(2), output_tokens: Some(3), ..llm::Usage::NONE };
    book.received(
        skein_lib::Token::new(99),
        &llm::Completion { content: Box::new([]), stop, usage, reasoning_dropped: Box::new([]) },
    );
    book.fact(
        skein_lib::Time::ZERO,
        &fact(
            5,
            session::FactKind::CompletionAnswered { opener: owner, stop, blocks: 0, calls: 0, invalid: 0 },
            3,
            None,
        ),
    );
    let records =
        book.fact(skein_lib::Time::ZERO, &fact(6, session::FactKind::Used { opener: owner, usage }, 3, Some(7)));
    let end = match stop {
        llm::Stop::Refusal => "refusal",
        llm::Stop::EndTurn | llm::Stop::ToolUse | llm::Stop::MaxTokens => "end",
    };
    let literal = alloc::format!(
        "{{\"v\":4,\"type\":\"response.completed\",\"t_ms\":5,\"run\":7,\"conversation\":1,\"response\":1,\"outcome\":\"completed\",\"stop\":\"{end}\",\"blocks\":0,\"calls\":0,\"invalid\":0,\"oversized\":0,\"cut\":0,\"usage\":{{\"input_tokens\":2,\"cache_read_tokens\":null,\"cache_write_tokens\":null,\"output_tokens\":3,\"reasoning_tokens\":null}},\"spent\":7,\"request_bytes\":null,\"first_byte_ms\":null,\"largest_gap_ms\":null,\"duration_ms\":1}}"
    );
    assert_stream_record(records.get(0).unwrap(), literal.as_bytes());
}

#[test]
fn a_compaction_is_a_conversation_of_kind_compaction_in_the_stream() {
    use crate::responses::tests::{at, fact, opened, prompt};
    use skein_lib::{Time, Token};
    use smith_domain::{llm, session};
    let owner = Token::new(11);
    let mut book = crate::responses::Book::new(1);
    book.opened(at(1), owner, &opened(0, 0));
    let opened = book.fact(
        Time::ZERO,
        &fact(2, session::FactKind::compaction_started(owner, 1, b"model-7", 123, 7, Some(b"medium")), 3, None),
    );
    assert_stream_record(opened.get(0).unwrap(), br#"{"v":4,"type":"conversation.opened","t_ms":2,"run":7,"conversation":1,"kind":"compaction","parent":0,"thread":0,"model":{"endpoint":"endpoint-7","model":"model-7","window_tokens":123,"output_tokens":7,"effort":"medium","account":null,"backend":"direct"}}"#);
    compaction_intent(&mut book, owner, 1);
    let started = book.request(at(4), Time::ZERO, Token::new(99), &prompt(0));
    assert_stream_record(&started, br#"{"v":4,"type":"response.started","t_ms":4,"run":7,"conversation":1,"response":1,"turn":1,"attempt":0,"window":1,"model":{"endpoint":"endpoint-7","model":"model-7","window_tokens":123,"output_tokens":7,"effort":"medium","account":null,"backend":"direct"},"messages":0,"output_tokens":7,"tool_choice":"only"}"#);
    assert_stream_record(
        &book.delta(at(4), Time::ZERO, Token::new(99), 0, b"summary".as_slice().into()),
        br#"{"v":4,"type":"text.delta","t_ms":4,"run":7,"conversation":1,"response":1,"block":0,"text":"summary"}"#,
    );
    book.prompt_recorded(Token::new(99), 4);
    compaction_answer(&mut book, owner, llm::Stop::EndTurn);
    let closed = book.fact(
        Time::ZERO,
        &fact(
            7,
            session::FactKind::CompactionEnded { opener: owner, window: 2, end: session::Compacted::Opened },
            3,
            None,
        ),
    );
    assert_stream_record(closed.get(0).unwrap(), br#"{"v":4,"type":"conversation.closed","t_ms":7,"run":7,"conversation":1,"end":"closed","turns":1,"usage":{"input_tokens":2,"cache_read_tokens":null,"cache_write_tokens":null,"output_tokens":3,"reasoning_tokens":null},"spent":7,"duration_ms":5}"#);
    compaction_intent(&mut book, owner, 2);
    let next = book.request(at(8), Time::ZERO, Token::new(100), &prompt(0));
    assert_stream_record(&next, br#"{"v":4,"type":"response.started","t_ms":8,"run":7,"conversation":0,"response":2,"turn":3,"attempt":0,"window":2,"model":{"endpoint":"endpoint-7","model":"model-7","window_tokens":123,"output_tokens":7,"effort":"medium","account":null,"backend":"direct"},"messages":0,"output_tokens":7,"tool_choice":"only"}"#);
    assert!(book.observation(Token::new(100)).first);
    assert_eq!(book.observation(Token::new(100)).messages, 0);
    assert_eq!(book.next_conversation(7), 2);
}

#[test]
fn a_refused_compaction_closes_failed_with_no_completion_failure() {
    use crate::responses::tests::{at, fact, opened, prompt};
    use skein_lib::{Time, Token};
    use smith_domain::{llm, session};
    let owner = Token::new(11);
    let mut book = crate::responses::Book::new(1);
    book.opened(at(1), owner, &opened(0, 0));
    book.fact(
        Time::ZERO,
        &fact(2, session::FactKind::compaction_started(owner, 1, b"model-7", 123, 7, Some(b"medium")), 3, None),
    );
    compaction_intent(&mut book, owner, 1);
    book.request(at(4), Time::ZERO, Token::new(99), &prompt(0));
    compaction_answer(&mut book, owner, llm::Stop::Refusal);
    let closed = book.fact(
        Time::ZERO,
        &fact(
            7,
            session::FactKind::CompactionEnded { opener: owner, window: 1, end: session::Compacted::Failed },
            3,
            None,
        ),
    );
    assert_stream_record(closed.get(0).unwrap(), br#"{"v":4,"type":"conversation.closed","t_ms":7,"run":7,"conversation":1,"end":"failed","turns":1,"usage":{"input_tokens":2,"cache_read_tokens":null,"cache_write_tokens":null,"output_tokens":3,"reasoning_tokens":null},"spent":7,"duration_ms":5}"#);
    let parent = book.fact(
        Time::ZERO,
        &fact(
            7,
            session::FactKind::Ended {
                opener: owner,
                end: session::End::NoSummary { fault: session::NoSummary::Refused },
                turns: 3,
                usage: llm::Usage::NONE,
            },
            3,
            None,
        ),
    );
    assert_stream_record(parent.get(0).unwrap(), br#"{"v":4,"type":"conversation.closed","t_ms":7,"run":7,"conversation":0,"end":"failed","turns":3,"usage":{"input_tokens":2,"cache_read_tokens":null,"cache_write_tokens":null,"output_tokens":3,"reasoning_tokens":null},"spent":7,"duration_ms":6}"#);
}

#[test]
fn a_failed_compaction_settles_its_response_before_closing_and_keeps_the_parent_limit() {
    use crate::responses::tests::{at, fact, opened, prompt};
    use skein_lib::{Time, Token};
    use smith_domain::{llm, session};
    use smith_events::Event;
    let owner = Token::new(11);
    let mut book = crate::responses::Book::new(1);
    book.opened(at(1), owner, &opened(0, 0));
    book.fact(
        Time::ZERO,
        &fact(2, session::FactKind::compaction_started(owner, 1, b"model-7", 123, 7, Some(b"medium")), 3, None),
    );
    compaction_intent(&mut book, owner, 1);
    book.request(at(4), Time::ZERO, Token::new(99), &prompt(0));
    book.failed(
        Token::new(99),
        llm::Failure::ContextTooLong,
        llm::Evidence::Response { status: Some(400) },
        b"context",
        crate::sinks::Capture::Calls,
        &llm::Report::none(),
    );
    book.fact(
        Time::ZERO,
        &fact(
            5,
            session::FactKind::CompletionFailed {
                opener: owner,
                failure: llm::Failure::ContextTooLong,
                evidence: llm::Evidence::Response { status: None },
            },
            3,
            None,
        ),
    );
    let records = book.fact(
        Time::ZERO,
        &fact(
            6,
            session::FactKind::CompactionEnded { opener: owner, window: 1, end: session::Compacted::Failed },
            3,
            None,
        ),
    );
    assert_eq!(records.len(), 2);
    let Event::ResponseCompleted(response) = &records.get(0).unwrap().event else { panic!("failed summary response") };
    let Event::ConversationClosed(compaction) = &records.get(1).unwrap().event else { panic!("failed compaction") };
    assert_eq!(response.conversation, 1);
    assert_eq!(compaction.conversation, 1);
    assert_eq!(response.failure, compaction.failure);
    let parent = book.fact(
        Time::ZERO,
        &fact(
            6,
            session::FactKind::Ended {
                opener: owner,
                end: session::End::Failed {
                    detail: Box::default(),
                    failure: llm::Failure::Limit { which: llm::Limit::Window, bound: 123 },
                    evidence: llm::Evidence::Response { status: None },
                    attempts: 1,
                },
                turns: 2,
                usage: llm::Usage::NONE,
            },
            3,
            None,
        ),
    );
    let Event::ConversationClosed(parent) = &parent.get(0).unwrap().event else { panic!("parent terminal") };
    assert_eq!(parent.conversation, 0);
    let failure = parent.failure.as_ref().unwrap();
    assert_eq!(failure.class, smith_events::FailureClass::Limit);
    assert_eq!(failure.limit.as_ref().unwrap().which.as_ref(), b"window");
    assert_eq!(failure.status, Some(400));
    assert_eq!(failure.detail.as_deref(), Some(b"context".as_slice()));
}

fn account_completion_record(
    failure: smith_domain::llm::Failure,
    capture: crate::sinks::Capture,
) -> smith_events::Record {
    use crate::responses::tests::{at, fact, intent, opened, prompt};
    use skein_lib::{Time, Token};
    use smith_domain::{llm, session};
    let owner = Token::new(11);
    let request = Token::new(99);
    let mut book = crate::responses::Book::new(1);
    book.opened(at(1), owner, &opened(0, 0));
    intent(&mut book, owner, session::Choice::Auto, 0);
    book.request(at(3), Time::ZERO, request, &prompt(0));
    let report = llm::Report {
        field: b"effort".as_slice().into(),
        code: b"not_allowed".as_slice().into(),
        request: b"request-7".as_slice().into(),
    };
    book.failed(request, failure, llm::Evidence::Response { status: Some(400) }, b"provider text", capture, &report);
    book.fact(
        Time::ZERO,
        &fact(
            4,
            session::FactKind::CompletionFailed {
                opener: owner,
                failure,
                evidence: llm::Evidence::Response { status: Some(400) },
            },
            1,
            None,
        ),
    );
    book.fact(
        Time::ZERO,
        &fact(
            5,
            session::FactKind::Ended {
                opener: owner,
                end: session::End::Failed {
                    detail: Box::default(),
                    failure,
                    evidence: llm::Evidence::Response { status: Some(400) },
                    attempts: 1,
                },
                turns: 0,
                usage: llm::Usage::NONE,
            },
            1,
            None,
        ),
    )
    .get(0)
    .unwrap()
    .clone()
}

#[test]
fn an_unsupported_completion_names_its_field_with_no_retry() {
    let record = account_completion_record(smith_domain::llm::Failure::Unsupported, crate::sinks::Capture::None);
    assert_stream_record(&record, br#"{"v":4,"type":"response.completed","t_ms":4,"run":7,"conversation":0,"response":1,"outcome":"failed","blocks":0,"calls":0,"invalid":0,"oversized":0,"cut":0,"usage":{"input_tokens":null,"cache_read_tokens":null,"cache_write_tokens":null,"output_tokens":null,"reasoning_tokens":null},"spent":0,"request_bytes":null,"first_byte_ms":null,"largest_gap_ms":null,"duration_ms":1,"failure":{"class":"unsupported","evidence":"response","status":400,"field":"effort","code":"not_allowed","request_id":"request-7"}}"#);
}

#[test]
fn a_refused_completions_detail_is_content() {
    struct Row {
        name: &'static str,
        capture: crate::sinks::Capture,
        detail: &'static str,
    }
    for row in [
        Row { name: "none", capture: crate::sinks::Capture::None, detail: "" },
        Row { name: "calls", capture: crate::sinks::Capture::Calls, detail: ",\"detail\":\"provider text\"" },
    ] {
        let record = account_completion_record(smith_domain::llm::Failure::Refused, row.capture);
        let literal = alloc::format!(
            "{{\"v\":4,\"type\":\"response.completed\",\"t_ms\":4,\"run\":7,\"conversation\":0,\"response\":1,\"outcome\":\"failed\",\"blocks\":0,\"calls\":0,\"invalid\":0,\"oversized\":0,\"cut\":0,\"usage\":{{\"input_tokens\":null,\"cache_read_tokens\":null,\"cache_write_tokens\":null,\"output_tokens\":null,\"reasoning_tokens\":null}},\"spent\":0,\"request_bytes\":null,\"first_byte_ms\":null,\"largest_gap_ms\":null,\"duration_ms\":1,\"failure\":{{\"class\":\"refused\",\"evidence\":\"response\",\"status\":400,\"field\":\"effort\",\"code\":\"not_allowed\",\"request_id\":\"request-7\"{detail}}}}}",
            detail = row.detail
        );
        let limits = stream_limits();
        let expected = smith_events::read(literal.as_bytes(), &limits).unwrap().unwrap();
        assert_eq!(record, expected, "{}", row.name);
    }
}

mod builtin;
