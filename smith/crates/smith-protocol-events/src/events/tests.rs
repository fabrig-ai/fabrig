use super::*;
use crate::responses::tests::{at, fact, opened, prompt, seen_completion};
use crate::sinks::tests::{drain_ready, limits};
use smith_domain::session;

#[test]
fn invalid_limits_are_refused() {
    let mut limits = Sources { session: smith_agent_world::LIMITS.session, output: 4096, events: None };
    assert!(valid(&limits));
    limits.events = Some(EventLimits {
        sinks: crate::sinks::tests::limits(1),
        capture: Capture::Everything,
        ..crate::fixture::event_limits()
    });
    assert!(valid(&limits));
    let base = limits;
    limits.events.as_mut().unwrap().sinks.output = 0;
    assert!(!valid(&limits));
    limits = base;
    limits.session.answer_bytes = u64::MAX;
    assert!(!valid(&limits));
    // Call joins exist even without content capture.
    limits.events.as_mut().unwrap().capture = Capture::None;
    assert!(!valid(&limits));
    limits.events = None;
    assert!(valid(&limits));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "two independent raw-accounting terminal controls exercise the complete producer and wire"
)]
fn refused_raw_totals_and_overflow_reach_the_wire_answer_and_operator_once() {
    for overflow in [false, true] {
        let bounds = limits(1);
        let destination = Token::new(400);
        let owner = Token::new(11);
        let mut producer = Producer {
            sinks: Sinks::new(&bounds, Capture::Calls, bounds.output, crate::sinks::tests::DEADLINE),
            names: channel::Endpoints::new(List::with_capacity(0)),
            programs: Box::default(),
            calls: crate::calls::Book::new(&smith_agent_world::LIMITS.session, Capture::Calls),
            messages: crate::messages::Book::disabled(),
            capture: Capture::Calls,
            origin: Some(Time::ZERO),
            start: None,
            admitted: Some(7),
            admitted_at: None,
            activation: Some(7),
            answer_at: None,
            overflow_notice: None,
            wind_down: false,
            unread: 0,
            check_at: None,
        };
        producer.sinks.add(crate::sinks::Destination::Trace, destination, crate::sinks::Delivery::Complete).unwrap();
        producer.sinks.conversation(at(1), Time::ZERO, owner, opened(11, 0));
        drop(drain_ready(&mut producer.sinks, destination));
        for turn in 1..=if overflow { 2 } else { 1 } {
            let mut completion = seen_completion();
            if overflow {
                completion.usage.input_tokens = Some(if turn == 1 { u64::MAX - 1 } else { 2 });
            }
            producer.sinks.session(
                Time::ZERO,
                &fact(
                    2,
                    session::FactKind::CompletionStarted {
                        window: 1,
                        opener: owner,
                        attempt: 0,
                        messages: 0,
                        max_tokens: 7,
                        choice: session::Choice::Only,
                    },
                    turn,
                    None,
                ),
            );
            producer.sinks.request(at(3), Time::ZERO, Token::new(99), &prompt(0));
            drop(drain_ready(&mut producer.sinks, destination));
            let seen_owner = producer.sinks.received(Token::new(99), &completion);
            producer.received(seen_owner, &completion);
            producer.sinks.session(
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
                    turn,
                    None,
                ),
            );
            let end = if overflow && turn == 1 {
                session::FactKind::Used { opener: owner, usage: completion.usage }
            } else {
                session::FactKind::Ended {
                    opener: owner,
                    end: if overflow { session::End::UsageOverflow } else { session::End::PriceOverflow },
                    turns: u32::from(overflow),
                    usage: domain::llm::Usage::ZERO,
                }
            };
            producer.sinks.session(Time::ZERO, &fact(5, end, turn, if overflow && turn == 1 { Some(0) } else { None }));
            let records = drain_ready(&mut producer.sinks, destination);
            if !(overflow && turn == 1) {
                let Event::ConversationClosed(closed) = &records.last().unwrap().event else {
                    panic!("conversation terminal")
                };
                assert_eq!(closed.usage.input_tokens, if overflow { None } else { Some(17) });
                assert_eq!(closed.usage.output_tokens, Some(if overflow { 38 } else { 19 }));
            }
        }
        let cause = if overflow { run::Overflow::Usage } else { run::Overflow::Spend };
        producer.answer(
            at(7),
            at(1),
            &run::Answer::Failed {
                failure: run::Failure::Budget { exhausted: run::Exhausted::Overflow(cause), used: None, bound: None },
                spent: run::Spend::ZERO,
                turns: u32::from(overflow),
            },
            None,
        );
        let records = drain_ready(&mut producer.sinks, destination);
        let Event::RunCompleted(answer) = &records[0].event else { panic!("wire answer") };
        assert_eq!(answer.usage.input_tokens, if overflow { None } else { Some(17) });
        assert_eq!(answer.usage.output_tokens, Some(if overflow { 38 } else { 19 }));
        assert_eq!(answer.usage.cache_read_tokens, None);
        assert_eq!(answer.usage.cache_write_tokens, Some(0));
        assert_eq!(answer.usage.reasoning_tokens, Some(if overflow { 6 } else { 3 }));
        assert_eq!(
            answer.failure.as_ref().unwrap().which.as_deref(),
            Some(if overflow { b"usage_overflow".as_slice() } else { b"spend_overflow".as_slice() })
        );
        assert_eq!(producer.pop_overflow_notice(), Some(cause));
        assert_eq!(producer.pop_overflow_notice(), None);
    }
}
