#![expect(
    clippy::disallowed_methods,
    reason = "programming-model.md section10.2: deterministic test harnesses use ordinary Rust"
)]

use super::*;
use crate::sinks::tests::{drain, drain_ready, grant};
use crate::sinks::{Capture, Delivery, Destination, Ended};
use skein_lib::Duration;

#[test]
fn invalid_limits_are_refused() {
    let mut limits = source();
    limits.sessions = 2;
    assert!(valid(&limits));
    for invalid in [
        session::Limits { sessions: u32::MAX, ..limits },
        session::Limits { parallel_tools: u32::MAX, ..limits },
        session::Limits { answer_bytes: u64::MAX, ..limits },
        session::Limits { completion_blocks: u32::MAX, ..limits },
    ] {
        assert!(!valid(&invalid));
    }
}

fn no_tool_result(record: &Record, capture: Capture) {
    let limits = crate::sinks::tests::limits(1).records;
    let line = smith_events::write(record, &capture.encoded(), &limits).unwrap().unwrap();
    assert!(!core::str::from_utf8(&line).unwrap().contains("\"result\":"));
}

fn source() -> session::Limits {
    session::Limits { sessions: 1, parallel_tools: 2, completion_bytes: 1024, ..smith_agent_world::LIMITS.session }
}

fn fixture(source: &session::Limits, capture: Capture) -> (Book, Sinks, Token, Token, session::ToolCall) {
    let book = Book::new(source, capture);
    let limits = crate::sinks::tests::limits(1);
    let mut sinks = Sinks::new(&limits, capture, 7, crate::sinks::tests::DEADLINE);
    let destination = Token::new(999);
    sinks.add(Destination::Trace, destination, Delivery::Complete).unwrap();
    grant(&mut sinks, destination);
    let opener = Token::new(91);
    sinks.conversation(
        Time::ZERO,
        Time::ZERO,
        opener,
        smith_events::ConversationOpened {
            run: 77,
            conversation: 0,
            kind: smith_events::ConversationKind::Main,
            parent: None,
            call: None,
            thread: 0,
            model: smith_events::Model {
                account: None,
                backend: smith_events::Backend::Direct,
                endpoint: b"endpoint-7".as_slice().into(),
                model: b"model-7".as_slice().into(),
                window_tokens: 4096,
                output_tokens: 32,
                effort: None,
            },
            requested: None,
            share: None,
            clamped: List::with_capacity(0),
        },
    );
    let opening = drain_ready(&mut sinks, destination);
    assert_eq!(opening.len(), 1);
    let metadata = session::ToolCall {
        owner: Token::new(43),
        id: b"provider-\"id".as_slice().into(),
        name: b"tool".as_slice().into(),
        source: session::ToolSource::Workspace,
        effect: tools::Effect::Read,
        deadline: Time::from_nanos(103_000_000),
        input_bytes: 1234,
    };
    (book, sinks, destination, opener, metadata)
}

fn timed_out_command_fact(opener: Token, metadata: session::ToolCall) -> session::Fact {
    session::Fact {
        at: metadata.deadline,
        kind: session::FactKind::Tools {
            opener,
            fact: tools::FactKind::Answered {
                session: Token::new(88),
                tool: tools::Tool::Shell,
                verdict: tools::Verdict::Exited { exit: tools::Exit::TimedOut },
                bytes: 0,
                omitted_bytes: 0,
                job: None,
                clamped: true,
            },
        },
        call: Some(metadata),
        response: None,
    }
}

fn running_fact(opener: Token, metadata: session::ToolCall) -> session::Fact {
    session::Fact {
        at: Time::from_nanos(1_000_000_000),
        kind: session::FactKind::Tools {
            opener,
            fact: tools::FactKind::Answered {
                session: Token::new(88),
                tool: tools::Tool::Shell,
                verdict: tools::Verdict::Running,
                bytes: 0,
                job: Some(tools::JobNumber(7)),
                omitted_bytes: 0,
                clamped: false,
            },
        },
        call: Some(metadata),
        response: None,
    }
}

#[test]
fn a_running_shell_terminal_names_its_seen_job() {
    let source = source();
    let (mut book, mut sinks, destination, opener, metadata) = fixture(&source, Capture::None);
    book.fact(Time::ZERO, &running_fact(opener, metadata), &mut sinks);
    let records = drain_ready(&mut sinks, destination);
    let Event::ToolCompleted(completed) = &records[1].event else { panic!("shell terminal") };
    assert_eq!(completed.job, Some(7));
    assert_eq!(completed.verdict, smith_events::Verdict::Running);
}

#[test]
fn an_unreported_job_end_keeps_its_shell_identity() {
    let source = source();
    let (mut book, mut sinks, destination, opener, metadata) = fixture(&source, Capture::None);
    book.fact(Time::ZERO, &running_fact(opener, metadata.clone()), &mut sinks);
    let mut records = drain_ready(&mut sinks, destination);
    book.fact(
        Time::ZERO,
        &session::Fact {
            at: Time::from_nanos(11_000_000_000),
            kind: session::FactKind::JobEnded {
                opener,
                job: tools::JobNumber(7),
                end: tools::JobEnd::Cancelled,
                unread_bytes: 13,
                started: Time::from_nanos(1_000_000_000),
            },
            call: Some(metadata),
            response: None,
        },
        &mut sinks,
    );
    records.extend(drain_ready(&mut sinks, destination));
    let Event::JobEnded(ended) = &records[2].event else { panic!("unreported job terminal") };
    assert_eq!(ended.call.as_ref(), b"provider-\"id");
    assert_eq!(ended.job, 7);
    assert_eq!(ended.verdict, smith_events::Verdict::Cancelled);
    assert_eq!(ended.unread_bytes, 13);
    assert_eq!(ended.duration_ms, 10_000);
}

#[test]
fn a_timed_out_command_is_projected_as_timed_out_with_its_bound() {
    let source = source();
    let (mut book, mut sinks, destination, opener, mut metadata) = fixture(&source, Capture::Everything);
    metadata.name = b"shell".as_slice().into();
    metadata.effect = tools::Effect::Write;
    let start =
        book.started(Time::ZERO, Time::from_nanos(3_000_000), opener, Scope { run: 77, conversation: 0 }, &metadata);
    sinks.record(&start);
    book.fact(Time::ZERO, &timed_out_command_fact(opener, metadata), &mut sinks);
    let records = drain_ready(&mut sinks, destination);
    let [started, completed] = records.as_slice() else { panic!("one command start and terminal") };
    let Event::ToolStarted(started) = &started.event else { panic!("command start") };
    let Event::ToolCompleted(completed) = &completed.event else { panic!("command terminal") };
    assert_eq!(started.deadline_ms, Some(100));
    assert_eq!(completed.verdict, smith_events::Verdict::TimedOut);
    assert_eq!(completed.exit, None);
    assert!(completed.clamped);
    assert_eq!(completed.duration_ms, 100);
    assert!(book.calls.is_empty());
}

fn workspace_fact(
    opener: Token,
    metadata: session::ToolCall,
    verdict: tools::Verdict,
    omitted: u64,
    clamped: bool,
) -> session::Fact {
    session::Fact {
        at: Time::from_nanos(14_000_000),
        kind: session::FactKind::Tools {
            opener,
            fact: tools::FactKind::Answered {
                session: Token::new(88),
                tool: tools::Tool::Read,
                verdict,
                bytes: 13,
                omitted_bytes: omitted,
                job: None,
                clamped,
            },
        },
        call: Some(metadata),
        response: None,
    }
}

#[test]
fn seen_workspace_terminals_keep_scalar_measurements_and_provider_identity_under_every_capture() {
    let source = source();
    for capture in [Capture::None, Capture::Calls, Capture::Everything] {
        for (verdict, expected, exit, omitted, clamped) in [
            (tools::Verdict::Read, smith_events::Verdict::Read, None, 19, true),
            (tools::Verdict::Read, smith_events::Verdict::Read, None, 0, false),
            (
                tools::Verdict::Exited { exit: tools::Exit::Code { code: 0 } },
                smith_events::Verdict::Exited,
                Some(smith_events::CommandExit::Code { code: 0 }),
                21,
                true,
            ),
            (
                tools::Verdict::Exited { exit: tools::Exit::Signal { signal: 15 } },
                smith_events::Verdict::Exited,
                Some(smith_events::CommandExit::Signal { signal: 15 }),
                0,
                false,
            ),
            (tools::Verdict::Exited { exit: tools::Exit::TimedOut }, smith_events::Verdict::TimedOut, None, 9, true),
            (tools::Verdict::TimedOut, smith_events::Verdict::TimedOut, None, 0, true),
            (tools::Verdict::Cancelled, smith_events::Verdict::Cancelled, None, 0, true),
        ] {
            let (mut book, mut sinks, destination, opener, metadata) = fixture(&source, capture);
            book.fact(
                Time::ZERO,
                &session::Fact {
                    at: Time::from_nanos(3_000_000),
                    kind: session::FactKind::Tools {
                        opener,
                        fact: tools::FactKind::Started { session: Token::new(88), tool: tools::Tool::Read },
                    },
                    call: Some(metadata.clone()),
                    response: None,
                },
                &mut sinks,
            );
            book.fact(Time::ZERO, &workspace_fact(opener, metadata.clone(), verdict, omitted, clamped), &mut sinks);
            assert!(book.calls.is_empty());
            assert_eq!(book.bytes, 0);
            sinks.ended(
                Time::from_nanos(20_000_000),
                Time::ZERO,
                &Ended {
                    exit: 0,
                    answered: true,
                    aborted: false,
                    teardown: Some(Duration::from_millis(6)),
                    usage: skein_io::Measured::Unread,
                    channel: None,
                },
            );
            let records = drain(&mut sinks, destination);
            let [started, completed, _] = records.as_slice() else { panic!("one start and one completed record") };
            let Event::ToolStarted(started) = &started.event else { panic!("start") };
            let Event::ToolCompleted(completed) = &completed.event else { panic!("completed") };
            assert_eq!((started.run, started.conversation, completed.run, completed.conversation), (77, 0, 77, 0));
            assert_eq!(started.call, metadata.id);
            assert_eq!(completed.call, metadata.id);
            assert_eq!(started.tool, metadata.name);
            assert_eq!(completed.tool, metadata.name);
            assert_eq!(started.input_bytes, 1234);
            assert_eq!(started.deadline_ms, Some(100));
            assert_eq!(started.input, None);
            no_tool_result(records.get(1).expect("terminal"), capture);
            assert_eq!(completed.verdict, expected);
            assert_eq!(completed.exit, exit);
            assert_eq!(
                (completed.bytes, completed.omitted_bytes, completed.clamped, completed.duration_ms),
                (13, omitted, clamped, 11)
            );
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one 15-case child control keeps the three-layer typed source join together")]
fn seen_child_terminals_join_public_identity_reported_usage_and_omissions_under_every_capture() {
    use smith_domain::run;
    let source = source();
    let used = run::facts::Used { turns: 2, spend: 18, time: Duration::from_millis(13) };
    for capture in [Capture::None, Capture::Calls, Capture::Everything] {
        for (result, yielded, end, expected, omitted) in [
            (
                run::facts::Return::Answered { used, cut: 17 },
                session::Yield::Done,
                run::facts::EndKind::Closed,
                smith_events::ChildStop::End,
                17,
            ),
            (
                run::facts::Return::Answered { used, cut: 17 },
                session::Yield::Budget,
                run::facts::EndKind::Closed,
                smith_events::ChildStop::Budget,
                17,
            ),
            (
                run::facts::Return::Unanswered { used },
                session::Yield::Done,
                run::facts::EndKind::Budget(run::Exhausted::Turns),
                smith_events::ChildStop::Failed,
                0,
            ),
            (
                run::facts::Return::Cancelled,
                session::Yield::Done,
                run::facts::EndKind::Closed,
                smith_events::ChildStop::Closed,
                0,
            ),
            (
                run::facts::Return::TimedOut,
                session::Yield::Done,
                run::facts::EndKind::Closed,
                smith_events::ChildStop::Closed,
                0,
            ),
        ] {
            let (mut book, mut sinks, destination, opener, mut metadata) = fixture(&source, capture);
            metadata.source = session::ToolSource::Run;
            book.fact(
                Time::ZERO,
                &session::Fact {
                    at: Time::from_nanos(3_000_000),
                    kind: session::FactKind::DelegateStarted { opener, block: 0 },
                    call: Some(metadata.clone()),
                    response: None,
                },
                &mut sinks,
            );
            let scope = sinks.scope(opener).unwrap();
            book.called(metadata.owner, scope, run::facts::Asked::SubAgent);
            let child = Token::new(92);
            let seen_id = book.child_opened(
                child,
                19,
                metadata.owner,
                scope,
                run::Granted {
                    turns: 2,
                    time: Duration::from_millis(30),
                    spend: 18,
                    turns_set: run::Set::Reserve,
                    time_set: run::Set::Asked,
                },
                Time::from_nanos(5_000_000),
            );
            assert_eq!(seen_id, metadata.id);
            for (number, charge, usage) in [
                (
                    1,
                    7,
                    smith_domain::llm::Usage {
                        input_tokens: Some(3),
                        output_tokens: None,
                        cache_read_tokens: Some(0),
                        cache_write_tokens: None,
                        reasoning_tokens: None,
                    },
                ),
                (
                    2,
                    11,
                    smith_domain::llm::Usage {
                        input_tokens: None,
                        output_tokens: Some(5),
                        cache_read_tokens: None,
                        cache_write_tokens: Some(2),
                        reasoning_tokens: Some(1),
                    },
                ),
            ] {
                book.child_received(child, usage);
                book.fact(
                    Time::ZERO,
                    &session::Fact {
                        at: Time::from_nanos(9_000_000),
                        kind: session::FactKind::Used { opener: child, usage },
                        call: None,
                        response: Some(session::ResponseInfo { number, spent: Some(charge) }),
                    },
                    &mut sinks,
                );
            }
            let answered = result == run::facts::Return::Answered { used, cut: 17 };
            if answered {
                book.fact(
                    Time::ZERO,
                    &session::Fact {
                        at: Time::from_nanos(17_000_000),
                        kind: session::FactKind::Yielded { opener: child, stop: yielded },
                        call: None,
                        response: None,
                    },
                    &mut sinks,
                );
            }
            book.fact(
                Time::ZERO,
                &session::Fact {
                    at: Time::from_nanos(18_000_000),
                    kind: session::FactKind::Ended {
                        opener: child,
                        end: session::End::Closed,
                        turns: 2,
                        usage: smith_domain::llm::Usage::NONE,
                    },
                    call: None,
                    response: None,
                },
                &mut sinks,
            );
            book.child_ended(child, end, Time::from_nanos(18_000_000));
            book.returned(metadata.owner, result);
            let terminal = if result == run::facts::Return::Cancelled {
                session::FactKind::DelegateCancelled { opener }
            } else {
                session::FactKind::DelegateAnswered { opener, bytes: 23, error: !answered }
            };
            book.fact(
                Time::ZERO,
                &session::Fact {
                    at: Time::from_nanos(21_000_000),
                    kind: terminal,
                    call: Some(metadata.clone()),
                    response: None,
                },
                &mut sinks,
            );
            sinks.ended(
                Time::from_nanos(23_000_000),
                Time::ZERO,
                &Ended {
                    exit: 0,
                    answered: true,
                    aborted: false,
                    teardown: Some(Duration::ZERO),
                    usage: skein_io::Measured::Unread,
                    channel: None,
                },
            );
            let records = drain(&mut sinks, destination);
            let [_, terminal, _] = records.as_slice() else { panic!("paired call and end") };
            let Event::ToolCompleted(completed) = &terminal.event else { panic!("typed completion") };
            assert_eq!((completed.run, completed.conversation), (77, 0));
            assert_eq!(completed.call, metadata.id);
            assert_eq!(completed.omitted_bytes, omitted);
            assert!(completed.clamped);
            let child = completed.child.as_ref().unwrap();
            assert_eq!((child.conversation, child.turns, child.time_ms, child.spent), (19, 2, 13, 18));
            assert_eq!(child.stop, expected);
            assert_eq!(
                child.usage,
                smith_events::Usage {
                    input_tokens: Some(3),
                    output_tokens: Some(5),
                    cache_read_tokens: Some(0),
                    cache_write_tokens: Some(2),
                    reasoning_tokens: Some(1)
                }
            );
            no_tool_result(terminal, capture);
            assert!(book.calls.is_empty());
        }
    }
}
#[test]
fn seen_main_deliveries_keep_the_exclusive_provider_call_under_every_capture() {
    use smith_domain::run;
    let source = source();
    for capture in [Capture::None, Capture::Calls, Capture::Everything] {
        for (status, returned, expected) in [
            (run::DeliveryStatus::Delivered, run::facts::Return::Delivered, smith_events::Delivered::Delivered),
            (run::DeliveryStatus::Nothing, run::facts::Return::Nothing, smith_events::Delivered::Nothing),
            (run::DeliveryStatus::Refused, run::facts::Return::DeliveryRefused, smith_events::Delivered::Refused),
            (
                run::DeliveryStatus::Failed(run::DeliveryReason::Unknown),
                run::facts::Return::DeliveryFailed,
                smith_events::Delivered::Failed,
            ),
            (run::DeliveryStatus::Stale, run::facts::Return::Stale, smith_events::Delivered::Stale),
        ] {
            for ask in [run::facts::Asked::Finish, run::facts::Asked::Deliver] {
                let (mut book, mut sinks, destination, opener, mut metadata) = fixture(&source, capture);
                metadata.source = session::ToolSource::Run;
                metadata.name = if ask == run::facts::Asked::Finish {
                    b"finish".as_slice().into()
                } else {
                    b"deliver".as_slice().into()
                };
                book.fact(
                    Time::ZERO,
                    &session::Fact {
                        at: Time::from_nanos(3_000_000),
                        kind: session::FactKind::DelegateStarted { opener, block: 0 },
                        call: Some(metadata.clone()),
                        response: None,
                    },
                    &mut sinks,
                );
                book.called(metadata.owner, Scope { run: 77, conversation: 0 }, ask);
                book.delivered(status);
                book.returned(
                    metadata.owner,
                    if ask == run::facts::Asked::Finish { run::facts::Return::Accepted } else { returned },
                );
                book.fact(
                    Time::ZERO,
                    &session::Fact {
                        at: Time::from_nanos(9_000_000),
                        kind: session::FactKind::DelegateAnswered { opener, bytes: 23, error: false },
                        call: Some(metadata.clone()),
                        response: None,
                    },
                    &mut sinks,
                );
                sinks.ended(
                    Time::from_nanos(11_000_000),
                    Time::ZERO,
                    &Ended {
                        exit: 0,
                        answered: true,
                        aborted: false,
                        teardown: Some(Duration::ZERO),
                        usage: skein_io::Measured::Unread,
                        channel: None,
                    },
                );
                let records = drain(&mut sinks, destination);
                let [_, terminal, _] = records.as_slice() else { panic!("paired provider call and process end") };
                let Event::ToolCompleted(completed) = &terminal.event else { panic!("provider terminal") };
                assert_eq!(completed.call, metadata.id);
                assert_eq!(completed.tool, metadata.name);
                assert_eq!(completed.delivery, Some(expected.clone()));
                assert_eq!(completed.child, None);
                assert_eq!((completed.omitted_bytes, completed.clamped), (0, false));
                assert!(book.calls.is_empty() && book.landing.is_none());
            }
        }
    }
}
