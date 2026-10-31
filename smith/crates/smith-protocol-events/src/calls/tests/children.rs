use super::*;
use crate::responses::tests::at;
use smith_domain::run;

const CHILD: Token = Token::new(92);
const USED: run::facts::Used = run::facts::Used { turns: 2, spend: 18, time: Duration::from_millis(13) };

struct Child {
    book: Book,
    sinks: Sinks,
    destination: Token,
    metadata: session::ToolCall,
    seen_id: Box<[u8]>,
}

impl Child {
    fn opened(capture: Capture) -> Self {
        let (mut book, mut sinks, destination, opener, mut metadata) = fixture(&source(), capture);
        metadata.source = session::ToolSource::Run;
        book.fact(
            Time::ZERO,
            &session::Fact {
                at: at(3),
                kind: session::FactKind::DelegateStarted { opener, block: 0 },
                call: Some(metadata.clone()),
                response: None,
            },
            &mut sinks,
        );
        let scope = sinks.scope(opener).unwrap();
        book.called(metadata.owner, scope, run::facts::Asked::SubAgent);
        let seen_id = book.child_opened(
            CHILD,
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
            at(5),
        );
        Self { book, sinks, destination, metadata, seen_id }
    }

    fn with_reported_usage(capture: Capture) -> Self {
        let mut child = Self::opened(capture);
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
            child.book.child_received(CHILD, usage);
            child.book.fact(
                Time::ZERO,
                &session::Fact {
                    at: at(9),
                    kind: session::FactKind::Used { opener: CHILD, usage },
                    call: None,
                    response: Some(session::ResponseInfo { number, spent: Some(charge) }),
                },
                &mut child.sinks,
            );
        }
        child
    }

    fn finish(&mut self, case: Case) -> Box<[Record]> {
        if let Some(kind) = case.yielded {
            self.book.fact(
                Time::ZERO,
                &session::Fact { at: at(17), kind, call: None, response: None },
                &mut self.sinks,
            );
        }
        self.book.fact(
            Time::ZERO,
            &session::Fact {
                at: at(18),
                kind: session::FactKind::Ended {
                    opener: CHILD,
                    end: session::End::Closed,
                    turns: 2,
                    usage: smith_domain::llm::Usage::NONE,
                },
                call: None,
                response: None,
            },
            &mut self.sinks,
        );
        self.book.child_ended(CHILD, case.end, at(18));
        self.book.returned(self.metadata.owner, case.result);
        self.book.fact(
            Time::ZERO,
            &session::Fact { at: at(21), kind: case.terminal, call: Some(self.metadata.clone()), response: None },
            &mut self.sinks,
        );
        self.sinks.ended(
            at(23),
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
        drain(&mut self.sinks, self.destination).into_boxed_slice()
    }
}

struct Case {
    result: run::facts::Return,
    yielded: Option<session::FactKind>,
    end: run::facts::EndKind,
    terminal: session::FactKind,
    stop: smith_events::ChildStop,
    omitted: u64,
}

fn cases(opener: Token) -> [Case; 5] {
    [
        Case {
            result: run::facts::Return::Answered { used: USED, cut: 17 },
            yielded: Some(session::FactKind::Yielded { opener: CHILD, stop: session::Yield::Done }),
            end: run::facts::EndKind::Closed,
            terminal: session::FactKind::DelegateAnswered { opener, bytes: 23, error: false },
            stop: smith_events::ChildStop::End,
            omitted: 17,
        },
        Case {
            result: run::facts::Return::Answered { used: USED, cut: 17 },
            yielded: Some(session::FactKind::Yielded { opener: CHILD, stop: session::Yield::Budget }),
            end: run::facts::EndKind::Closed,
            terminal: session::FactKind::DelegateAnswered { opener, bytes: 23, error: false },
            stop: smith_events::ChildStop::Budget,
            omitted: 17,
        },
        Case {
            result: run::facts::Return::Unanswered { used: USED },
            yielded: None,
            end: run::facts::EndKind::Budget(run::Exhausted::Turns),
            terminal: session::FactKind::DelegateAnswered { opener, bytes: 23, error: true },
            stop: smith_events::ChildStop::Failed,
            omitted: 0,
        },
        Case {
            result: run::facts::Return::Cancelled,
            yielded: None,
            end: run::facts::EndKind::Closed,
            terminal: session::FactKind::DelegateCancelled { opener },
            stop: smith_events::ChildStop::Closed,
            omitted: 0,
        },
        Case {
            result: run::facts::Return::TimedOut,
            yielded: None,
            end: run::facts::EndKind::Closed,
            terminal: session::FactKind::DelegateAnswered { opener, bytes: 23, error: true },
            stop: smith_events::ChildStop::Closed,
            omitted: 0,
        },
    ]
}

#[test]
fn an_opened_child_keeps_the_provider_call_identity() {
    let child = Child::opened(Capture::None);
    assert_eq!(child.seen_id, child.metadata.id);
}

#[test]
fn child_terminals_join_identity_usage_and_omissions() {
    for capture in [Capture::None, Capture::Calls, Capture::Everything] {
        for case in cases(OPENER) {
            let mut scenario = Child::with_reported_usage(capture);
            let (stop, omitted) = (case.stop.clone(), case.omitted);
            let records = scenario.finish(case);
            let [_, terminal, _] = records.as_ref() else { panic!("paired call and end") };
            let Event::ToolCompleted(completed) = &terminal.event else { panic!("typed completion") };
            assert_eq!((completed.run, completed.conversation), (77, 0));
            assert_eq!(completed.call, scenario.metadata.id);
            assert_eq!(completed.omitted_bytes, omitted);
            assert!(completed.clamped);
            let child = completed.child.as_ref().unwrap();
            assert_eq!((child.conversation, child.turns, child.time_ms, child.spent), (19, 2, 13, 18));
            assert_eq!(child.stop, stop);
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
            assert!(scenario.book.calls.is_empty());
        }
    }
}
