use super::*;
#[test]
fn the_hello_s_jig_range_reaches_the_domain() {
    let harness = fixtures::Harness::streaming(fixtures::frame_subject());
    assert!(harness.has(&Request::Hello { family: None, versions: Some(c::VersionRange { first: 22, last: 22 }) }));
}
#[test]
fn the_hello_s_family_range_reaches_its_owner() {
    let harness = fixtures::Harness::streaming(fixtures::family_subject());
    assert!(harness.has(&Request::Hello { family: Some(19), versions: Some(c::VersionRange { first: 23, last: 23 }) }));
}

#[test]
fn a_hello_reopens_no_watch_of_an_owner_it_does_not_serve() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.open(fixtures::WATCH, fixtures::family_subject());
    harness.clear();
    let bytes = fixtures::hello_event(fixtures::HEARTBEAT);
    let w::StreamEvent::Hello(hello) = w::StreamEvent::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap() else {
        panic!("fixture hello");
    };
    let mut hello = hello.into_parts();
    hello.served.clear();
    hello
        .served
        .push(w::Served::new(&w::CEILINGS, w::ServedParts { owner: w::Owner::Jig, first: 22, last: 22 }).unwrap())
        .unwrap();
    harness.stream_record(&fixtures::event_bytes(&w::StreamEvent::Hello(w::Hello::new(&w::CEILINGS, hello).unwrap())));
    assert!(harness.has(&Request::Hello { family: Some(19), versions: None }));
    for request in &harness.out {
        if let Request::Exchange { .. } = request {
            panic!("no opening exchange outside its owner range");
        }
    }
    harness.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut harness.out);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Closed));
    for request in &harness.out {
        if let Request::Exchange { .. } = request {
            panic!("no opening exchange outside its owner range");
        }
    }
}
#[test]
fn a_delivery_before_its_opening_s_answer_admits_it() {
    let mut harness = fixtures::Harness::streaming(fixtures::family_subject());
    let (exchange, name) = harness.opening();
    harness.stream_record(&fixtures::family_delivery(name));
    harness.answer(exchange, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    assert!(harness.has(&Request::Watch { watch: fixtures::WATCH, event: c::WatchEvent::Opened }));
    harness.expected_family();
}
#[test]
fn silence_past_twice_the_heartbeat_drops_each_watch() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    let watch = harness.open(fixtures::WATCH, fixtures::frame_subject());
    harness.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
    harness.advance(fixtures::past_two_heartbeats());
    assert_eq!(harness.ended(watch), Some(c::WatchEnd::Dropped));
}
#[test]
fn a_second_watch_uses_the_stream_already_opened() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    harness.clear();
    harness.open(Token::new(42), fixtures::family_subject());
    assert!(!harness.has(&Request::OpenStream { credential: None }));
}
#[test]
fn an_opening_waits_for_the_stream_s_name() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.open(fixtures::WATCH, fixtures::frame_subject());
    assert_eq!(harness.out.len(), 1);
    assert!(harness.has(&Request::OpenStream { credential: None }));
}
#[test]
fn a_watch_s_opening_answer_is_handed_up() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    let (exchange, _) = harness.opening();
    harness.answer(exchange, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    assert!(harness.has(&Request::Watch { watch: fixtures::WATCH, event: c::WatchEvent::Opened }));
}
#[test]
fn each_watch_refusal_is_handed_up() {
    for (why, expected) in [
        (w::WatchRefusal::Unknown, c::WatchEnd::Unknown),
        (w::WatchRefusal::NotAllowed, c::WatchEnd::NotAllowed),
        (w::WatchRefusal::Busy, c::WatchEnd::Busy),
        (w::WatchRefusal::SignedOut, c::WatchEnd::SignedOut),
    ] {
        let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
        let (exchange, _) = harness.opening();
        let value = w::OpenAnswer::Refused(
            w::OpenAnswerRefused::new(fixtures::limits().wire(), w::OpenAnswerRefusedParts { why }).unwrap(),
        );
        harness.answer(exchange, &fixtures::open_answer(&value));
        assert_eq!(harness.ended(fixtures::WATCH), Some(expected));
    }
}
#[test]
fn a_close_s_answer_and_stream_ending_yield_one_terminal() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    let (opening, name) = harness.opening();
    harness.answer(opening, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    let close = harness.close();
    harness.answer(close, &fixtures::close_answer());
    harness.stream_record(&fixtures::ending(name, w::WatchEnd::Closed));
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Closed));
    assert_eq!(harness.closed_count(), 1);
}
#[test]
fn an_opening_waits_for_a_read_reply() {
    let mut h = fixtures::Harness::admitted_stream();
    let mut reads = h.fill_reads();
    let watch = Token::new(42);
    h.open(watch, fixtures::family_subject());
    assert!(h.out.is_empty(), "local exchange pressure is not Busy");
    assert_eq!(h.link.in_flight(), fixtures::EXCHANGE_ROOM);
    h.answer(reads.pop().unwrap(), &fixtures::read_none_answer(w::ReadNone::Unknown));
    let (opening, _) = h.opening();
    assert_eq!(h.closed_count(), 0);
    assert_eq!(h.link.in_flight(), fixtures::EXCHANGE_ROOM);
    assert!(!h.link.has_exchange_room());
    h.clear();
    h.answer(opening, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    assert!(h.has(&Request::Watch { watch, event: c::WatchEvent::Opened }));
}

#[test]
fn an_opening_waits_for_a_read_loss() {
    let mut h = fixtures::Harness::admitted_stream();
    let mut reads = h.fill_reads();
    h.open(Token::new(42), fixtures::frame_subject());
    assert!(h.out.is_empty());
    h.link.step(Time::ZERO, Event::Lost { exchange: reads.pop().unwrap() }, &mut h.out);
    h.opening();
    assert_eq!(h.closed_count(), 0);
    assert_eq!(h.link.in_flight(), fixtures::EXCHANGE_ROOM);
}

#[test]
fn held_openings_leave_in_request_order_not_handle_order() {
    let mut h = fixtures::Harness::admitted_stream();
    let mut reads = h.fill_reads();
    let first = Token::new(43);
    let second = Token::new(42);
    h.open(first, fixtures::frame_subject());
    h.open(second, fixtures::family_subject());
    assert!(h.out.is_empty());
    for watch in [first, second] {
        h.answer(reads.pop().unwrap(), &fixtures::read_none_answer(w::ReadNone::Unknown));
        assert_eq!(h.out.len(), 2, "one read terminal and one opening per freed credit");
        let (exchange, _) = h.opening();
        assert_eq!(h.link.attempts.get(&exchange).unwrap().token, watch);
        assert_eq!(h.closed_count(), 0);
        h.clear();
    }
}

#[test]
fn closing_a_held_opening_ends_once_without_taking_credit() {
    let mut h = fixtures::Harness::admitted_stream();
    let mut reads = h.fill_reads();
    let watch = Token::new(42);
    h.open(watch, fixtures::frame_subject());
    for _ in 0_u32..2 {
        h.link.step(Time::ZERO, Event::Close { watch }, &mut h.out);
    }
    assert_eq!(h.ended(watch), Some(c::WatchEnd::Closed));
    assert_eq!(h.closed_count(), 1);
    h.answer(reads.pop().unwrap(), &fixtures::read_none_answer(w::ReadNone::Unknown));
    h.assert_no_watch_exchange();
    assert_eq!(h.closed_count(), 1);
    assert_eq!(h.link.in_flight(), fixtures::EXCHANGE_ROOM - 1);
    assert!(h.link.has_exchange_room());
}

#[test]
fn an_admitted_closing_waits_for_exchange_credit() {
    for lost in [false, true] {
        let mut h = fixtures::Harness::admitted_stream();
        let mut reads = h.fill_reads();
        h.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut h.out);
        h.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut h.out);
        assert!(h.out.is_empty(), "a closing waits instead of Dropped");
        let exchange = reads.pop().unwrap();
        if lost {
            h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out);
        } else {
            h.answer(exchange, &[]);
        }
        let (close, _) = h.unwatching();
        assert_eq!(h.closed_count(), 0);
        assert_eq!(h.link.in_flight(), fixtures::EXCHANGE_ROOM);
        h.clear();
        h.answer(close, &fixtures::close_answer());
        h.answer(close, &fixtures::close_answer());
        assert_eq!(h.ended(fixtures::WATCH), Some(c::WatchEnd::Closed));
        assert_eq!(h.closed_count(), 1);
        h.assert_no_watch_exchange();
    }
}

#[test]
fn held_closings_and_openings_share_request_order() {
    let mut h = fixtures::Harness::admitted_stream();
    let mut reads = h.fill_reads();
    h.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut h.out);
    h.open(Token::new(42), fixtures::frame_subject());
    assert!(h.out.is_empty());
    h.answer(reads.pop().unwrap(), &fixtures::read_none_answer(w::ReadNone::Unknown));
    h.unwatching();
    assert_eq!(h.out.len(), 2);
    h.clear();
    h.answer(reads.pop().unwrap(), &fixtures::read_none_answer(w::ReadNone::Unknown));
    h.opening();
    assert_eq!(h.out.len(), 2);
}

#[test]
fn a_stream_end_discards_held_openings_and_closings() {
    let mut h = fixtures::Harness::admitted_stream();
    let reads = h.fill_reads();
    let held = Token::new(42);
    h.open(held, fixtures::frame_subject());
    h.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut h.out);
    h.link.step(Time::ZERO, Event::StreamGone, &mut h.out);
    assert_eq!(h.ended(held), Some(c::WatchEnd::Dropped));
    assert_eq!(h.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
    assert_eq!(h.closed_count(), 2);
    h.clear();
    h.link.step(Time::ZERO, Event::StreamGone, &mut h.out);
    for exchange in &reads {
        h.answer(*exchange, &[]);
    }
    h.assert_no_watch_exchange();
    assert_eq!(h.closed_count(), 0, "late terminals cannot revive a held watch");
}

#[test]
fn signing_out_discards_held_openings_and_closings() {
    let mut h = fixtures::Harness::admitted_stream();
    let reads = h.fill_reads();
    let held = Token::new(42);
    h.open(held, fixtures::frame_subject());
    h.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut h.out);
    h.link.step(Time::ZERO, Event::Streamed { message: Streamed::SignedOut }, &mut h.out);
    assert_eq!(h.ended(held), Some(c::WatchEnd::SignedOut));
    assert_eq!(h.ended(fixtures::WATCH), Some(c::WatchEnd::SignedOut));
    assert_eq!(h.closed_count(), 2);
    h.clear();
    for exchange in &reads {
        h.answer(*exchange, &[]);
    }
    h.assert_no_watch_exchange();
    assert_eq!(h.closed_count(), 0);
}

#[test]
fn a_closing_waits_for_its_opening_answer_even_after_a_delivery() {
    let mut h = fixtures::Harness::streaming(fixtures::family_subject());
    let (opening, name) = h.opening();
    h.clear();
    h.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut h.out);
    assert!(h.out.is_empty());
    h.stream_record(&fixtures::family_delivery(name));
    h.assert_no_watch_exchange();
    assert_eq!(h.link.in_flight(), 1, "the held closing takes no credit");
    h.answer(opening, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    let (close, closing_name) = h.unwatching();
    assert_eq!(closing_name, name);
    h.answer(close, &fixtures::close_answer());
    assert_eq!(h.closed_count(), 1);
    assert_eq!(h.ended(fixtures::WATCH), Some(c::WatchEnd::Closed));
}

#[test]
fn a_refused_opening_never_sends_its_held_closing() {
    let mut h = fixtures::Harness::streaming(fixtures::frame_subject());
    let (opening, _) = h.opening();
    h.clear();
    h.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut h.out);
    let refused = w::OpenAnswer::Refused(
        w::OpenAnswerRefused::new(&w::CEILINGS, w::OpenAnswerRefusedParts { why: w::WatchRefusal::Busy }).unwrap(),
    );
    h.answer(opening, &fixtures::open_answer(&refused));
    assert_eq!(h.closed_count(), 1);
    assert_eq!(h.ended(fixtures::WATCH), Some(c::WatchEnd::Busy));
    h.assert_no_watch_exchange();
}

#[test]
fn held_watches_wait_for_cancelled_transport_credit() {
    for lost in [false, true] {
        let limits = Limits::new(*fixtures::limits().client(), 1).unwrap();
        let mut h = fixtures::Harness::new(&limits);
        h.open(fixtures::WATCH, fixtures::frame_subject());
        h.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
        let (exchange, name) = h.opening();
        h.clear();
        h.open(Token::new(42), fixtures::frame_subject());
        h.stream_record(&fixtures::ending(name, w::WatchEnd::Closed));
        assert!(h.link.has_exchange_room(), "the logical attempt was cancelled");
        assert_eq!(h.link.in_flight(), 1, "HTTP still owns its credit");
        h.assert_no_watch_exchange();
        h.clear();
        if lost {
            h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out);
        } else {
            h.answer(exchange, &[]);
        }
        h.opening();
        assert_eq!(h.closed_count(), 0);
        assert_eq!(h.link.in_flight(), 1);
    }
}

#[test]
fn a_lost_watch_exchange_discards_held_work_with_its_stream() {
    for closing in [false, true] {
        let limits = Limits::new(*fixtures::limits().client(), 1).unwrap();
        let mut h = fixtures::Harness::new(&limits);
        h.open(fixtures::WATCH, fixtures::frame_subject());
        h.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
        let (mut exchange, _) = h.opening();
        if closing {
            h.answer(exchange, &fixtures::open_answer(&w::OpenAnswer::Admitted));
            h.clear();
            exchange = h.close();
        }
        h.clear();
        let held = Token::new(42);
        h.open(held, fixtures::frame_subject());
        h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out);
        assert_eq!(h.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
        assert_eq!(h.ended(held), Some(c::WatchEnd::Dropped));
        assert_eq!(h.closed_count(), 2);
        h.assert_no_watch_exchange();
        assert!(h.has(&Request::CloseStream));
    }
}

#[test]
fn a_full_watch_table_still_refuses_busy_while_openings_wait() {
    let base = fixtures::limits();
    let limits = Limits::new(jig_client_domain::Limits { watches: 1, ..*base.client() }, 8).unwrap();
    let mut h = fixtures::Harness::new(&limits);
    h.fill_reads();
    h.open(fixtures::WATCH, fixtures::frame_subject());
    h.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
    h.clear();
    let refused = Token::new(42);
    h.open(refused, fixtures::frame_subject());
    assert_eq!(h.ended(refused), Some(c::WatchEnd::Busy));
    assert_eq!(h.closed_count(), 1);
    assert_eq!(h.ended(fixtures::WATCH), None);
    h.assert_no_watch_exchange();
}

// The browser reserves one fetch beyond the link's exchange limit for cleanup.
const AUTHORIZATION_ROOM: u32 = 1;
const CLEANUP_ROOM: u32 = 1;
// The fixture's small go record fits inside the wire location allowance.
const ANSWER_BYTES: u32 = w::CEILINGS.authorize_go_location;
#[derive(Clone, Copy)]
enum FinishedExchange {
    Answered,
    Failed,
}
fn authorize_when_room(
    harness: &mut fixtures::Harness,
    fetch: &mut skein_ui_web::fetch::Fetch,
) -> Option<(Token, u64)> {
    if harness.link.in_flight() >= AUTHORIZATION_ROOM {
        return None;
    }
    harness.link.step(Time::ZERO, Event::Authorize { query: Box::from(b"client_id=app".as_slice()) }, &mut harness.out);
    let Some(Request::Exchange { exchange, message: Message::Authorize(_), .. }) = harness.out.pop() else {
        panic!("one authorization exchange");
    };
    match fetch.begin(u32::try_from(exchange.raw()).unwrap(), ANSWER_BYTES, &[]) {
        Some(ticket) => Some((exchange, ticket)),
        None => {
            harness.link.step(Time::ZERO, Event::Lost { exchange }, &mut harness.out);
            None
        }
    }
}
#[test]
fn a_stream_end_keeps_authorization_waiting_for_its_http_exchanges() {
    for finish in [FinishedExchange::Answered, FinishedExchange::Failed] {
        let base = fixtures::limits();
        let limits = Limits::new(*base.client(), AUTHORIZATION_ROOM).unwrap();
        let mut harness = fixtures::Harness::new(&limits);
        let mut fetch = skein_ui_web::fetch::Fetch::new(AUTHORIZATION_ROOM + CLEANUP_ROOM);
        harness.open(fixtures::WATCH, fixtures::frame_subject());
        harness.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
        let (opening, _) = harness.opening();
        let opened = fetch.begin(u32::try_from(opening.raw()).unwrap(), ANSWER_BYTES, &[]).unwrap();
        harness.clear();
        harness.link.step(Time::ZERO, Event::StreamGone, &mut harness.out);
        let mut cleanup = None;
        for request in &harness.out {
            if let Request::Exchange { exchange, message: Message::StreamGone(_), .. } = request {
                cleanup = Some(*exchange);
            }
        }
        let close = cleanup.expect("stream cleanup exchange");
        let closed = fetch.begin(u32::try_from(close.raw()).unwrap(), ANSWER_BYTES, &[]).unwrap();
        assert_eq!(harness.closed_count(), 1, "the stream ends the watch once");
        harness.clear();

        let pending = authorize_when_room(&mut harness, &mut fetch);
        assert!(
            !harness.has(&Request::Authorized(Authorized::Lost)),
            "cancelled watch calls still fill fetch; authorization must wait instead of getting Lost"
        );
        assert!(pending.is_none(), "the first authorization waits for transport credit");
        for (exchange, ticket) in [(opening, opened), (close, closed)] {
            match finish {
                FinishedExchange::Answered => {
                    assert!(fetch.finish(ticket, 200, &[]).is_some());
                    harness.answer(exchange, &[]);
                }
                FinishedExchange::Failed => {
                    assert!(fetch.fail(ticket).is_some());
                    harness.link.step(Time::ZERO, Event::Lost { exchange }, &mut harness.out);
                }
            }
        }
        assert_eq!(harness.closed_count(), 0, "late HTTP terminals do not end the watch twice");
        harness.clear();
        let (exchange, ticket) = authorize_when_room(&mut harness, &mut fetch).expect("the terminal frees credit");
        let location = b"https://app.example.com/callback?code=one";
        let answer = w::AuthorizeAnswer::Go(
            w::AuthorizeGo::new(&w::CEILINGS, w::AuthorizeGoParts { location: Box::from(location.as_slice()) })
                .unwrap(),
        );
        let mut writer = skein_lib::Writer::new(skein_lib::usize_of(answer.measure()));
        answer.encode(&mut writer).unwrap();
        assert!(fetch.piece(ticket, &writer.finish()).is_none());
        let Some(skein_ui_web::fetch::Report::Answered { body, .. }) = fetch.finish(ticket, 200, &[]) else {
            panic!("a whole authorization answer");
        };
        harness.answer(exchange, &body);
        assert_eq!(
            harness.out.pop(),
            Some(Request::Authorized(Authorized::Go { location: Box::from(location.as_slice()) }))
        );
        assert!(harness.out.is_empty());
    }
}
#[test]
fn a_heartbeat_keeps_each_watch_alive() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    harness.clear();
    harness.link.step(Time::from_nanos(150), Event::Streamed { message: Streamed::Heartbeat }, &mut harness.out);
    harness.advance(fixtures::past_two_heartbeats());
    assert_eq!(harness.ended(fixtures::WATCH), None);
    assert!(harness.has(&Request::Watch { watch: fixtures::WATCH, event: c::WatchEvent::Alive }));
}
#[test]
fn the_stream_is_live_at_exactly_twice_the_heartbeat() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    harness.advance(Time::from_nanos(200));
    assert_eq!(harness.ended(fixtures::WATCH), None);
}
#[test]
fn a_duplicate_hello_drops_the_stream() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    harness.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
}
#[test]
fn a_delivery_before_the_hello_drops_the_stream() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.open(fixtures::WATCH, fixtures::family_subject());
    harness.stream_record(&fixtures::family_delivery(1));
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
}
#[test]
fn a_new_opening_never_takes_the_old_wire_name() {
    let mut harness = fixtures::Harness::streaming(fixtures::family_subject());
    let (_, old) = harness.opening();
    harness.stream_record(&fixtures::ending(old, w::WatchEnd::Closed));
    harness.clear();
    harness.open(fixtures::WATCH, fixtures::family_subject());
    let (_, new) = harness.opening();
    assert_ne!(old, new);
    harness.stream_record(&fixtures::family_delivery(old));
    assert!(!harness.has(&Request::FamilyDelivery { watch: fixtures::WATCH, record: fixtures::family_read() }));
}
#[test]
fn a_stream_end_drops_all_watches_once() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    harness.open(Token::new(42), fixtures::family_subject());
    harness.link.step(Time::ZERO, Event::StreamGone, &mut harness.out);
    harness.link.step(Time::ZERO, Event::StreamGone, &mut harness.out);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
    assert_eq!(harness.ended(Token::new(42)), Some(c::WatchEnd::Dropped));
    assert_eq!(harness.closed_count(), 2);
}
#[test]
fn a_stream_refused_signed_out_ends_the_sign_in() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.open(fixtures::WATCH, fixtures::family_subject());
    harness.link.step(Time::ZERO, Event::Streamed { message: Streamed::SignedOut }, &mut harness.out);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::SignedOut));
    assert!(harness.has(&Request::Signing { signing: Signing::SignedOut }));
    assert!(harness.has(&Request::CloseStream));
    assert!(!harness.has(&Request::Fact { fact: Fact::StreamDropped }));
}
#[test]
fn a_stream_drop_tells_the_engine_which_stream_is_gone() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    harness.link.step(Time::ZERO, Event::StreamGone, &mut harness.out);
    assert_eq!(harness.gone_name(), fixtures::STREAM);
}
#[test]
fn a_close_before_the_hello_ends_without_an_exchange() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.open(fixtures::WATCH, fixtures::frame_subject());
    harness.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut harness.out);
    harness.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
    assert_eq!(harness.closed_count(), 1);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Closed));
}
#[test]
fn a_continued_watch_has_a_fresh_name_and_retains_its_run_offset() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    let (_, old) = harness.opening();
    harness.clear();
    harness.link.step(
        Time::ZERO,
        Event::Continue {
            watch: fixtures::WATCH,
            subject: c::Subject::Run {
                task: 7,
                turn: Some(c::ProvisionalTurnName { attempt: 9, number: 2, retry: 3 }),
                offset: 123,
            },
        },
        &mut harness.out,
    );
    let (_, new) = harness.opening();
    assert_ne!(old, new);
    harness.assert_run_subject();
    harness.stream_record(&fixtures::ending(old, w::WatchEnd::Closed));
    assert_eq!(harness.ended(fixtures::WATCH), None);
}
#[test]
fn a_lost_opening_drops_its_watch() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    let (exchange, _) = harness.opening();
    harness.link.step(Time::ZERO, Event::Lost { exchange }, &mut harness.out);
    harness.answer(exchange, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    assert_eq!(harness.closed_count(), 1);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
}
#[test]
fn a_delivery_whose_owner_differs_drops_the_stream() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    let (_, name) = harness.opening();
    harness.stream_record(&fixtures::family_delivery(name));
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
    assert!(harness.has(&Request::Fact { fact: Fact::UndecodableEvent }));
}
#[test]
fn a_run_s_text_is_handed_up_with_its_offsets() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    let (_, name) = harness.opening();
    harness.stream_record(&fixtures::run_event(name));
    assert_eq!(harness.delivered(), Some(&c::Delivery::Run(Box::new(fixtures::run_text_expected()))));
}
#[test]
fn each_jig_delivery_reaches_the_domain_with_its_variant() {
    for (record, expected) in fixtures::delivery_cases().as_ref() {
        let bytes = fixtures::encoded_delivery(record);
        assert_eq!(translate::watches::delivery(&bytes, &fixtures::limits()), Ok(expected.clone()));
    }
}
#[test]
fn a_jig_delivery_before_the_opening_answer_admits_it() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    let (exchange, name) = harness.opening();
    harness.stream_record(&fixtures::jig_delivery(&w::DeliveryValue::Me(fixtures::frame_record()), name));
    harness.answer(exchange, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    assert_eq!(harness.delivered(), Some(&c::Delivery::Me(Box::new(fixtures::frame()))));
}
#[test]
fn a_delivery_over_the_original_document_bound_is_refused() {
    let value = w::DeliveryValue::Me(fixtures::frame_record());
    let bytes = fixtures::encoded_delivery(&value);
    let limits = Limits::new(*fixtures::limits().client(), 4).unwrap();
    let mut client = *limits.client();
    client.document_bytes = u64::from(value.measure()).checked_sub(1).unwrap();
    let limits = Limits::new(client, 4).unwrap();
    assert_eq!(translate::watches::delivery(&bytes, &limits), Err(Undecodable::TooLarge));
}
#[test]
fn an_owner_inbox_delivery_preserves_both_groups_and_its_next_position() {
    let bytes = fixtures::encoded_delivery(&fixtures::inbox_with_rows(1));
    assert_eq!(translate::watches::delivery(&bytes, &fixtures::limits()), Ok(fixtures::inbox_with_rows_expected()));
}
#[test]
fn an_owner_tree_delivery_preserves_its_whole_descendant() {
    let bytes = fixtures::encoded_delivery(&fixtures::tree_with_rows(1));
    assert_eq!(translate::watches::delivery(&bytes, &fixtures::limits()), Ok(fixtures::tree_with_rows_expected()));
}
#[test]
fn a_delivery_piece_that_does_not_split_is_refused() {
    for record in &fixtures::malformed_delivery_pieces() {
        let bytes = fixtures::encoded_delivery(record);
        assert_eq!(translate::watches::delivery(&bytes, &fixtures::limits()), Err(Undecodable::Malformed));
    }
}
#[test]
fn an_owner_inbox_delivery_one_row_over_the_limit_is_refused() {
    let bytes = fixtures::encoded_delivery(&fixtures::inbox_with_rows(3));
    assert_eq!(translate::watches::delivery(&bytes, &fixtures::limits()), Err(Undecodable::TooLarge));
}
#[test]
fn an_owner_tree_delivery_one_row_over_the_limit_is_refused() {
    let bytes = fixtures::encoded_delivery(&fixtures::tree_with_rows(3));
    assert_eq!(translate::watches::delivery(&bytes, &fixtures::limits()), Err(Undecodable::TooLarge));
}

#[test]
fn a_malformed_stream_record_drops_the_stream_and_is_reported() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    harness.stream_record(&[255]);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
    assert!(harness.has(&Request::Fact { fact: Fact::UndecodableEvent }));
    assert!(harness.has(&Request::CloseStream));
}

#[test]
fn an_undecodable_opening_answer_drops_the_stream_and_is_reported() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    let (exchange, _) = harness.opening();
    harness.clear();
    harness.answer(exchange, &[255]);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
    assert!(harness.has(&Request::CloseStream));
    assert!(harness.has(&Request::Fact { fact: Fact::UndecodableWatchAnswer }));
}

#[test]
fn an_undecodable_closing_answer_drops_the_stream_and_is_reported() {
    let mut harness = fixtures::Harness::admitted_stream();
    let exchange = harness.close();
    harness.clear();
    harness.answer(exchange, &[255]);
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
    assert!(harness.has(&Request::CloseStream));
    assert!(harness.has(&Request::Fact { fact: Fact::UndecodableWatchAnswer }));
}

#[test]
fn a_watch_ended_not_served_ends_its_handle_naming_why() {
    let mut h = fixtures::Harness::streaming(fixtures::family_subject());
    let (_, name) = h.opening();
    h.stream_record(&fixtures::ending(name, w::WatchEnd::NotServed(fixtures::not_served())));
    assert_eq!(h.ended(fixtures::WATCH), Some(c::WatchEnd::NotServed));
    assert!(h.has(&Request::Hello { family: Some(19), versions: Some(c::VersionRange { first: 24, last: 24 }) }));
}

#[test]
fn the_stream_s_name_is_saved_at_each_hello() {
    const HELLOS: u32 = 2;
    let mut h = fixtures::Harness::new(&fixtures::limits());
    for _ in 0..HELLOS {
        h.clear();
        h.open(fixtures::WATCH, fixtures::frame_subject());
        h.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
        assert!(h.has(&Request::Save { record: fixtures::kept_stream() }));
        h.link.step(Time::ZERO, Event::StreamGone, &mut h.out);
    }
}
const GONE_TERMINALS: [bool; 2] = [false, true];
#[test]
fn a_kept_stream_name_is_sent_gone_before_the_first_open() {
    for lost in GONE_TERMINALS {
        let mut h = fixtures::Harness::new(&fixtures::limits());
        h.start_store(fixtures::one_store_read(fixtures::kept_stream()));
        h.open(fixtures::WATCH, fixtures::frame_subject());
        assert_eq!(h.gone_name(), fixtures::STREAM, "lost={lost}");
        let Some(Request::Exchange { exchange, .. }) = h.out.iter().next() else { panic!("gone first") };
        let exchange = *exchange;
        h.clear();
        h.open(fixtures::ATTEMPT, fixtures::family_subject());
        assert_eq!(h.out.len(), 0, "lost={lost}");
        let event = if lost { Event::Lost { exchange } } else { Event::Reply { exchange, message: Box::new([]) } };
        h.link.step(Time::ZERO, event, &mut h.out);
        assert_eq!(h.out.len(), 1, "lost={lost}");
        assert!(h.has(&Request::OpenStream { credential: None }), "lost={lost}");
    }
}

#[test]
fn a_full_stream_tells_the_bound_and_drops_its_watches_without_signing_out() {
    let mut harness = fixtures::Harness::streaming(fixtures::frame_subject());
    harness.clear();
    let full = StreamsFull { bound: StreamBound::Component, size: 2 };
    harness.link.step(Time::ZERO, Event::Streamed { message: Streamed::Full(full) }, &mut harness.out);
    assert!(harness.has(&Request::StreamsFull { full }));
    assert!(harness.has(&Request::CloseStream));
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
    assert!(!harness.has(&Request::Signing { signing: Signing::SignedOut }));
}

fn run_delivery(bytes: &[u8]) -> w::RunText {
    let mut parts = fixtures::text_record().into_parts();
    parts.from = 0;
    parts.through = u32::try_from(bytes.len()).unwrap();
    parts.text = Box::from(bytes);
    w::RunText::new(&w::CEILINGS, parts).unwrap()
}

#[test]
fn a_delivery_past_a_fields_text_is_received_within_long_text() {
    for long_text in [64, w::CEILINGS.run_text_text + 1] {
        let base = fixtures::limits();
        let client = jig_client_domain::Limits {
            long_text,
            document_bytes: u64::from(w::CEILINGS.owned_record),
            ..*base.client()
        };
        let limits = Limits::new(client, base.exchanges()).unwrap();
        let length = long_text.min(w::CEILINGS.run_text_text);
        let mut bytes = skein_lib::bytes::zeroed(usize::try_from(length).unwrap());
        bytes.fill(b'x');
        let mut harness = fixtures::Harness::new(&limits);
        harness.open(fixtures::WATCH, fixtures::frame_subject());
        harness.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
        let (_, name) = harness.opening();
        let event = w::StreamEvent::Text(
            w::StreamText::new(&w::CEILINGS, w::StreamTextParts { watch: name, value: run_delivery(&bytes) }).unwrap(),
        );
        harness.stream_record(&fixtures::event_bytes(&event));
        let Some(c::Delivery::Run(value)) = harness.delivered() else {
            panic!("run delivery within long_text");
        };
        assert_eq!(value.text.bytes(), bytes.as_ref());
        assert_eq!(limits.receiving().run_text_text, length);
        assert_eq!(limits.receiving().current_call_input, client.text);
        assert_eq!(limits.receiving().call_input, client.text);
    }
}

#[test]
fn a_delivery_past_long_text_is_refused() {
    let base = fixtures::limits();
    let client = jig_client_domain::Limits { long_text: 64, ..*base.client() };
    let limits = Limits::new(client, base.exchanges()).unwrap();
    assert_eq!(translate::live::run_text(run_delivery(&[b'x'; 65]), &limits), Err(Undecodable::TooLarge));
    let mut parts = fixtures::text_record().into_parts();
    parts.text = skein_lib::bytes::zeroed(usize::try_from(w::CEILINGS.run_text_text + 1).unwrap());
    assert_eq!(w::RunText::new(&w::CEILINGS, parts).unwrap_err().path, w::Path::RunTextText);
}
