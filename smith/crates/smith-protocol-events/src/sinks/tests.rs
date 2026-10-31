#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "programming-model.md section 10.2: deterministic harnesses use ordinary Rust"
)]

use super::*;

mod program;
mod view;

/// The write deadline IO keeps on each test destination.
pub(crate) use super::fixture::{DEADLINE, drain, drain_ready, grant, limits};

#[test]
fn the_live_view_drops_without_holding_a_none_capture_destination() {
    let limits = limits(1);
    let token = Token::new(400);
    let mut sinks = Sinks::with_front(&limits, Capture::None, 7, DEADLINE, true);
    sinks.add(Destination::Trace, token, Delivery::Complete).unwrap();
    for number in 0..6 {
        sinks.record(&Record {
            t_ms: number,
            event: Event::TextDelta(smith_events::TextDelta {
                run: 1,
                conversation: 1,
                response: 1,
                block: 0,
                text: b"live".as_slice().into(),
            }),
        });
    }
    assert!(sinks.takes(), "a full live view never holds the run");
    assert_eq!(sinks.front_lost(), 2);
    for number in 0..4 {
        let record = sinks.pop_front().expect("the bounded live view");
        let View::TextDelta { t_ms, delta } = record else { panic!("typed delta") };
        assert_eq!(t_ms, number);
        assert_eq!(delta.text.as_ref(), b"live");
    }
    assert!(sinks.pop_front().is_none());
    assert!(drain_ready(&mut sinks, token).is_empty(), "trace capture remains none");
}

#[test]
fn authoritative_credit_uses_its_own_metadata_and_answer_bounds() {
    let mut limits = limits(2);
    limits.records.string = 524_288;
    let line = limits.line_ceiling(Capture::Everything).unwrap();
    let authoritative = largest_line(
        &smith_events::Limits { string: limits.metadata_string, ..limits.records },
        Capture::Everything,
        limits.answer_bytes,
        limits.metadata_string,
    )
    .unwrap();
    limits.output = authoritative.checked_mul(4).unwrap().checked_add(line.checked_mul(2).unwrap()).unwrap();
    assert!(limits.output < line.checked_mul(6).unwrap(), "ordinary identities do not enter authoritative records");
    assert!(valid(&limits, Capture::Everything));
    let mut sinks = Sinks::new(&limits, Capture::Everything, 1, DEADLINE);
    sinks.add(Destination::Trace, Token::new(1), Delivery::Complete).unwrap();
    assert!(sinks.takes(), "two ordinary records and every authoritative record have separate reserved credit");
}

#[test]
fn only_everything_captures_the_exact_accepted_report_text_and_fields() {
    let seen = smith_domain::run::Answer::Accepted {
        outcome: smith_domain::run::outcome::Declared::Report(smith_domain::run::outcome::Report {
            text: b"Frozen task token: ALPHA.\n\"quoted\"".to_vec().into(),
            fields: Box::new([smith_domain::run::outcome::Field {
                name: b"source".to_vec().into(),
                value: b"\nreport".to_vec().into(),
            }]),
        }),
        spent: smith_domain::run::Spend {
            units: 17,
            input: Some(19),
            output: Some(23),
            ..smith_domain::run::Spend::ZERO
        },
        turns: 2,
    };
    for capture in [Capture::None, Capture::Calls, Capture::Everything] {
        let mut limits = limits(1);
        limits.output =
            largest_line(&limits.records, capture, limits.answer_bytes, limits.metadata_string).unwrap() * 6;
        let token = Token::new(11);
        let mut sinks = Sinks::new(&limits, capture, 7, DEADLINE);
        sinks.add(Destination::Trace, token, Delivery::Complete).unwrap();
        let answer = crate::answer_record(
            crate::AnswerMeta {
                now: Time::ZERO,
                run: 7,
                wind_down: true,
                read: Some(31),
                unread: 1,
                elapsed: Duration::from_millis(29),
            },
            &seen,
            capture,
        );
        sinks.record(&Record { t_ms: 37, event: Event::RunCompleted(answer) });
        // The same policy grants the request's content in 02.6.
        let request =
            golden(include_bytes!("../../../smith-events/golden/v4/response_started_everything.jsonl"), &limits);
        let Event::ResponseStarted(expected) = &request.event else { panic!("golden request") };
        let expected = expected.prompt.clone();
        sinks.record(&request);
        sinks.finish();
        let records = drain(&mut sinks, token);
        let Event::RunCompleted(answer) = &records[0].event else { panic!("typed answer") };
        assert_eq!(answer.status, smith_events::Status::Accepted);
        assert_eq!(
            (answer.run, answer.turns, answer.spent, answer.read, answer.unread, answer.duration_ms),
            (7, 2, 17, Some(31), 1, 29)
        );
        assert!(answer.wind_down);
        assert_eq!(answer.usage.input_tokens, Some(19));
        assert_eq!(answer.usage.output_tokens, Some(23));
        assert_eq!(answer.usage.reasoning_tokens, None);
        let result = answer.result.as_ref().unwrap();
        assert_eq!(result.form, smith_events::Form::Report);
        if capture == Capture::Everything {
            assert_eq!(result.text.as_deref(), Some(b"Frozen task token: ALPHA.\n\"quoted\"".as_slice()));
            assert_eq!(
                result.fields.as_ref().unwrap().as_slice(),
                &[smith_events::Field { name: b"source".to_vec().into(), text: b"\nreport".to_vec().into() }]
            );
        } else {
            assert_eq!(result.text, None);
            assert_eq!(result.fields, None);
        }
        let Event::ResponseStarted(request) = &records[1].event else { panic!("typed request") };
        assert_eq!(request.prompt, if capture == Capture::Everything { expected } else { None });
    }
}

#[test]
fn invalid_limits_are_refused() {
    let mut base = limits(1);
    base.output = output_bound(&base, Capture::Calls).unwrap();
    assert!(valid(&base, Capture::Calls));
    for bounds in [
        SinkLimits { metadata_string: base.records.string + 1, ..base },
        SinkLimits { records: smith_events::Limits { items: 18, ..base.records }, ..base },
        SinkLimits { records: smith_events::Limits { string: 18, ..base.records }, metadata_string: 18, ..base },
        SinkLimits { scalar_content: u32::MAX, ..base },
        SinkLimits { answer_bytes: u32::MAX, ..base },
        SinkLimits { observation_line: u32::MAX, ..base },
        SinkLimits { fact_extra: u32::MAX, ..base },
        SinkLimits { output: base.output - 1, ..base },
    ] {
        assert!(!valid(&bounds, Capture::Calls), "{bounds:?}");
    }
}

#[test]
fn the_complete_emitted_vocabulary_must_fit_the_receiving_bounds() {
    let mut bounds = limits(1);
    bounds.records.string = 18;
    bounds.metadata_string = 18;
    assert!(!valid(&bounds, Capture::None));
    bounds.records.string = 19;
    bounds.metadata_string = 19;
    assert!(valid(&bounds, Capture::None));
    bounds.records.items = 17;
    assert!(!valid(&bounds, Capture::None));
}

#[test]
fn settled_end_preserves_read_or_unread_resources_and_the_seen_exit() {
    let measured = io::kernel::Usage {
        own: io::kernel::Resources {
            user: Duration::from_nanos(1_000_000),
            system: Duration::from_nanos(2_000_000),
            peak_rss_bytes: 5,
        },
        children: io::kernel::Resources {
            user: Duration::from_nanos(3_000_000),
            system: Duration::from_nanos(4_000_000),
            peak_rss_bytes: 6,
        },
    };
    for usage in [io::Measured::Read(measured), io::Measured::Unread] {
        for exit in [0, 1] {
            let limits = limits(1);
            let token = Token::new(11);
            let mut sinks = Sinks::new(&limits, Capture::None, limits.output, DEADLINE);
            sinks.add(Destination::Trace, token, Delivery::Complete).unwrap();
            sinks.record(&counted(1));
            let ended = Ended {
                exit,
                answered: exit == 0,
                aborted: false,
                teardown: if exit == 0 { Some(Duration::from_nanos(3_000_000)) } else { None },
                usage,
                channel: Some(7),
            };
            sinks.ended(Time::from_nanos(12_000_000), Time::from_nanos(2_000_000), &ended);
            let records = drain(&mut sinks, token);
            assert!(sinks.closed());
            assert_eq!(records.len(), 2);
            let last = records.last().unwrap();
            assert_eq!(last.t_ms, 10);
            let Event::SessionEnded(end) = &last.event else { panic!("one final process end") };
            assert_eq!(end.exit, i64::from(exit));
            assert_eq!(end.answered, exit == 0);
            assert_eq!(end.teardown_ms, if exit == 0 { Some(3) } else { None });
            assert_eq!(end.loss, smith_events::Loss { events: 0, channel: Some(7) });
            assert_eq!(end.emitted.iter().map(|entry| entry.count).sum::<u64>(), 2);
            assert_eq!(end.emitted.iter().find(|entry| entry.record.as_ref() == b"session.ended").unwrap().count, 1);
            match usage {
                io::Measured::Read(_) => {
                    assert_eq!(
                        end.cpu_ms,
                        Some(smith_events::Cpu { user: 1, system: 2, children_user: 3, children_system: 4 })
                    );
                    assert_eq!(end.peak_rss_bytes, Some(smith_events::PeakRss { self_bytes: 5, children: 6 }));
                }
                io::Measured::Unread => {
                    assert_eq!(end.cpu_ms, None);
                    assert_eq!(end.peak_rss_bytes, None);
                }
            }
        }
    }
}

#[test]
fn each_settled_end_reports_its_destination_loss_and_counts_itself_once() {
    let count = u64::from(LINE_SLOTS).next_power_of_two();
    let limits = limits(2);
    let token = Token::new(11);
    let mut sinks = Sinks::new(&limits, Capture::None, limits.output, DEADLINE);
    sinks.add(Destination::Trace, token, Delivery::BestEffort).unwrap();
    sinks.add(Destination::Output, Token::new(12), Delivery::BestEffort).unwrap();
    sinks.up(Time::ZERO, io::Event::Failed { owner: Token::new(12), error: io::Error::Other });
    sinks.up(Time::ZERO, io::Event::Closed { owner: Token::new(12) });
    for name in 0..count {
        sinks.record(&counted(name));
    }
    let ended =
        Ended { exit: 0, answered: true, aborted: false, teardown: None, usage: io::Measured::Unread, channel: None };
    sinks.ended(Time::ZERO, Time::ZERO, &ended);
    let records = drain(&mut sinks, token);
    let Event::SessionEnded(end) = &records.last().unwrap().event else { panic!("end is last") };
    assert_eq!(end.emitted.iter().map(|entry| entry.count).sum::<u64>(), count + 1);
    assert_eq!(u64::try_from(records.len()).unwrap().checked_add(end.loss.events).unwrap(), count + 1);
    assert_eq!(sinks.writers.get(1).unwrap().loss, count + 1);
    assert_eq!(sinks.pop_given_up(), Some(GivenUp { destination: Destination::Output, why: Why::WriteFailed }));
    assert_eq!(sinks.pop_given_up(), None);
    assert!(sinks.closed());
}

fn settled() -> Ended {
    Ended { exit: 0, answered: true, aborted: false, teardown: None, usage: io::Measured::Unread, channel: None }
}

/// One trace destination with one counted record sent at `at` and not yet drained.
fn sending(at: Time) -> (Sinks, Token, Queue<io::Request>) {
    let limits = limits(1);
    let token = Token::new(11);
    let mut sinks = Sinks::new(&limits, Capture::None, limits.output, DEADLINE);
    sinks.add(Destination::Trace, token, Delivery::Complete).unwrap();
    let mut out = Queue::with_capacity(2);
    sinks.record(&counted(1));
    sinks.down(at, &mut out);
    let Some(io::Request::Stream { down: stream::Down::Demand { .. }, .. }) = out.pop() else {
        panic!("a full-credit demand")
    };
    grant(&mut sinks, token);
    sinks.down(at, &mut out);
    let Some(io::Request::Stream { down: stream::Down::Send(_), .. }) = out.pop() else { panic!("one send") };
    (sinks, token, out)
}

#[test]
fn a_failed_write_mid_run_names_its_destination_and_why_once_and_later_records_count_lost() {
    let at = Time::from_nanos(5_000_000_000);
    let (mut sinks, token, mut out) = sending(at);
    sinks.record(&counted(2));
    sinks.up(at, io::Event::Stream { owner: token, up: stream::Up::Failed(stream::Fault::Other) });
    assert_eq!(sinks.pop_given_up(), Some(GivenUp { destination: Destination::Trace, why: Why::WriteFailed }));
    assert_eq!(sinks.pop_given_up(), None, "the shell names a destination once");
    assert_eq!(sinks.writers.get(0).unwrap().loss, 2, "the record in flight and the one queued");
    sinks.record(&counted(3));
    sinks.ended(at, Time::ZERO, &settled());
    assert_eq!(sinks.writers.get(0).unwrap().loss, 4, "a given-up destination gets nothing more, its end too");
    sinks.down(at, &mut out);
    assert!(out.is_empty(), "nothing is tried there again");
    sinks.up(at, io::Event::Closed { owner: token });
    assert_eq!(sinks.pop_given_up(), None);
    assert!(sinks.closed());
}

#[test]
fn a_destination_failed_at_its_write_deadline_was_behind_by_the_sinks_own_clock() {
    let at = Time::from_nanos(5_000_000_000);
    let due = at.saturating_add(DEADLINE);
    let before = Time::from_nanos(due.as_nanos().checked_sub(1).unwrap());
    for (now, why) in [(before, Why::WriteFailed), (due, Why::Behind)] {
        let (mut sinks, token, _) = sending(at);
        sinks.up(now, io::Event::Stream { owner: token, up: stream::Up::Failed(stream::Fault::Other) });
        assert_eq!(sinks.pop_given_up(), Some(GivenUp { destination: Destination::Trace, why }));
    }
    let (mut sinks, token, mut out) = sending(at);
    sinks.down(at, &mut out);
    let Some(io::Request::Stream { down: stream::Down::Demand { .. }, .. }) = out.pop() else {
        panic!("the demand that proves the send drained")
    };
    grant(&mut sinks, token);
    sinks.down(due, &mut out);
    assert!(out.is_empty(), "the drained send leaves nothing in flight");
    sinks.up(due, io::Event::Stream { owner: token, up: stream::Up::Failed(stream::Fault::Other) });
    assert_eq!(
        sinks.pop_given_up(),
        Some(GivenUp { destination: Destination::Trace, why: Why::WriteFailed }),
        "with no write outstanding, the deadline cannot have passed"
    );
}

#[test]
fn a_reset_stream_says_its_reader_went_away() {
    let at = Time::from_nanos(5_000_000_000);
    let (mut sinks, token, _) = sending(at);
    sinks.up(
        at.saturating_add(DEADLINE),
        io::Event::Stream { owner: token, up: stream::Up::Failed(stream::Fault::Reset) },
    );
    assert_eq!(sinks.pop_given_up(), Some(GivenUp { destination: Destination::Trace, why: Why::ReaderGone }));
    let (mut sinks, token, _) = sending(at);
    sinks.up(at, io::Event::Failed { owner: token, error: io::Error::Reset });
    assert_eq!(sinks.pop_given_up(), Some(GivenUp { destination: Destination::Trace, why: Why::ReaderGone }));
}

#[test]
fn a_teardowns_abort_gives_no_destination_up() {
    let at = Time::from_nanos(5_000_000_000);
    let (mut sinks, token, mut out) = sending(at);
    sinks.record(&counted(2));
    sinks.ended(at, Time::ZERO, &settled());
    sinks.abort(&mut out);
    let Some(io::Request::Abort { entity }) = out.pop() else { panic!("the abort") };
    assert_eq!(entity, token);
    assert!(out.is_empty());
    sinks.abort(&mut out);
    assert!(out.is_empty(), "a stream is aborted once");
    assert_eq!(sinks.pop_given_up(), None, "the abort is the end, not a failure");
    sinks.up(at, io::Event::Stream { owner: token, up: stream::Up::Room });
    sinks.up(at, io::Event::Stream { owner: token, up: stream::Up::Failed(stream::Fault::Other) });
    sinks.up(at, io::Event::Failed { owner: token, error: io::Error::Other });
    assert_eq!(sinks.pop_given_up(), None, "what follows the abort gives nothing up either");
    assert!(!sinks.closed(), "the aborted stream still owes Closed");
    sinks.down(at, &mut out);
    assert!(out.is_empty(), "an aborted destination is never written again");
    sinks.up(at, io::Event::Closed { owner: token });
    assert_eq!(sinks.pop_given_up(), None);
    assert!(sinks.closed());
    assert_eq!(sinks.writers.get(0).unwrap().loss, 3, "what the abort cut off counts as lost");
}

#[test]
fn a_failure_already_named_before_the_abort_is_still_named_once() {
    let at = Time::from_nanos(5_000_000_000);
    let (mut sinks, token, mut out) = sending(at);
    sinks.up(at, io::Event::Stream { owner: token, up: stream::Up::Failed(stream::Fault::Other) });
    sinks.abort(&mut out);
    let Some(io::Request::Abort { .. }) = out.pop() else { panic!("the abort") };
    sinks.up(at, io::Event::Closed { owner: token });
    assert_eq!(sinks.pop_given_up(), Some(GivenUp { destination: Destination::Trace, why: Why::WriteFailed }));
    assert_eq!(sinks.pop_given_up(), None);
    assert!(sinks.closed());
}

fn counted(name: u64) -> Record {
    Record { t_ms: name, event: Event::MessageUnread(smith_events::MessageUnread { run: 7, name }) }
}

fn golden(bytes: &[u8], limits: &SinkLimits) -> Record {
    smith_events::read(bytes, &limits.records).unwrap().unwrap()
}

#[test]
fn a_slow_complete_destination_preserves_order_and_reserves_authoritative_room() {
    let count = u64::from(LINE_SLOTS).next_power_of_two();
    let limits = limits(1);
    let token = Token::new(11);
    let mut sinks = Sinks::new(&limits, Capture::None, limits.output, DEADLINE);
    sinks.add(Destination::Trace, token, Delivery::Complete).unwrap();
    let mut expected = Vec::new();
    let start = golden(include_bytes!("../../../smith-events/golden/v4/session_started_none.jsonl"), &limits);
    sinks.record(&start);
    expected.push(start);
    for name in 0..count {
        if !sinks.takes() {
            break;
        }
        let record = counted(name);
        sinks.record(&record);
        expected.push(record);
    }
    assert!(!sinks.takes(), "the output cap holds the next fact");
    let completed = golden(include_bytes!("../../../smith-events/golden/v4/run_completed_none.jsonl"), &limits);
    sinks.record(&completed);
    expected.push(completed);
    let end = golden(include_bytes!("../../../smith-events/golden/v4/session_ended_none.jsonl"), &limits);
    sinks.record(&end);
    expected.push(end);
    sinks.finish();
    assert_eq!(drain(&mut sinks, token), expected);
    assert_eq!(sinks.writers.get(0).unwrap().loss, 0);
    assert!(sinks.closed());
    let emitted = sinks.emitted();
    assert_eq!(emitted.iter().map(|entry| entry.count).sum::<u64>(), u64::try_from(expected.len()).unwrap());
}

#[test]
fn best_effort_loses_only_the_counted_records_that_find_it_behind() {
    let count = u64::from(LINE_SLOTS).next_power_of_two();
    let limits = limits(1);
    let token = Token::new(11);
    let mut sinks = Sinks::new(&limits, Capture::None, limits.output, DEADLINE);
    sinks.add(Destination::Trace, token, Delivery::BestEffort).unwrap();
    for name in 0..count {
        sinks.record(&counted(name));
    }
    assert!(sinks.takes(), "best effort never holds its owner");
    let lost = sinks.writers.get(0).unwrap().loss;
    assert!(lost > 0);
    sinks.finish();
    let records = drain(&mut sinks, token);
    assert_eq!(u64::try_from(records.len()).unwrap().checked_add(lost).unwrap(), count);
    assert_eq!(sinks.emitted().get(0).unwrap().count, count);
}

#[test]
fn a_failed_stream_counts_queued_inflight_and_later_records_and_notifies_once() {
    let limits = limits(1);
    let token = Token::new(11);
    let mut sinks = Sinks::new(&limits, Capture::None, 1, DEADLINE);
    sinks.add(Destination::Trace, token, Delivery::Complete).unwrap();
    sinks.record(&counted(1));
    let mut out = Queue::with_capacity(2);
    sinks.down(Time::ZERO, &mut out);
    let Some(io::Request::Stream { down: stream::Down::Demand { .. }, .. }) = out.pop() else {
        panic!("full-credit demand")
    };
    grant(&mut sinks, token);
    sinks.down(Time::ZERO, &mut out);
    let Some(io::Request::Stream { down: stream::Down::Send(_), .. }) = out.pop() else { panic!("one owning send") };
    assert!(sinks.writers.get(0).unwrap().active.is_some(), "only a prefix left the owner");
    assert_eq!(
        sinks.writers.get(0).unwrap().pending.as_ref().unwrap().bytes,
        0,
        "prefixes keep the whole record credit"
    );
    sinks.record(&counted(2));
    sinks.up(Time::ZERO, io::Event::Stream { owner: token, up: stream::Up::Failed(stream::Fault::Other) });
    assert_eq!(sinks.pop_given_up(), Some(GivenUp { destination: Destination::Trace, why: Why::WriteFailed }));
    assert_eq!(sinks.pop_given_up(), None);
    sinks.record(&counted(3));
    assert!(sinks.takes());
    assert_eq!(sinks.writers.get(0).unwrap().loss, 3);
    sinks.up(Time::ZERO, io::Event::Failed { owner: token, error: io::Error::TimedOut });
    assert_eq!(sinks.pop_given_up(), None);
    assert!(!sinks.closed(), "the append descriptor still owes Closed");
    sinks.up(Time::ZERO, io::Event::Closed { owner: token });
    assert!(sinks.closed());
    sinks.record(&counted(4));
    assert_eq!(sinks.writers.get(0).unwrap().loss, 4);
    sinks.down(Time::ZERO, &mut out);
    assert!(out.is_empty(), "an abandoned destination is never written again");
}

#[test]
fn one_destination_failing_does_not_abandon_the_other() {
    let limits = limits(2);
    let token = Token::new(11);
    let mut sinks = Sinks::new(&limits, Capture::None, limits.output, DEADLINE);
    sinks.add(Destination::Trace, token, Delivery::Complete).unwrap();
    sinks.add(Destination::Output, Token::new(12), Delivery::Complete).unwrap();
    sinks.record(&counted(1));
    sinks.up(Time::ZERO, io::Event::Failed { owner: Token::new(12), error: io::Error::Other });
    sinks.up(Time::ZERO, io::Event::Closed { owner: Token::new(12) });
    sinks.record(&counted(2));
    assert_eq!(sinks.pop_given_up(), Some(GivenUp { destination: Destination::Output, why: Why::WriteFailed }));
    sinks.finish();
    assert_eq!(drain(&mut sinks, token), [counted(1), counted(2)]);
    assert_eq!(sinks.writers.get(0).unwrap().loss, 0);
    assert_eq!(sinks.writers.get(1).unwrap().loss, 2);
    assert!(sinks.closed());
}
