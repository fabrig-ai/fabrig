//! A settling's members and their independent outcomes (protocol/client/testing.md, 1).
use super::*;
use skein_lib::{List, Writer};

fn gather(h: &mut fixtures::Harness, number: u64) {
    h.link.step(
        Time::ZERO,
        Event::Read {
            read: Token::new(number),
            what: Readable::Jig(c::Read::Project { project: u32::try_from(number).unwrap() }),
        },
        &mut h.out,
    );
}
fn batch(h: &fixtures::Harness) -> (Token, w::Batch) {
    assert_eq!(h.exchanges(), 1);
    for request in &h.out {
        match request {
            Request::Exchange { exchange, message: Message::Batch(body), .. } => {
                return (*exchange, w::Batch::decode(h.link.limits.wire(), &mut Reader::new(body)).unwrap());
            }
            Request::Exchange { .. }
            | Request::Read { .. }
            | Request::StreamsFull { .. }
            | Request::Authorized(_)
            | Request::AskedLink(_)
            | Request::Approval(_)
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Kept { .. }
            | Request::Dropped
            | Request::NotKept { .. }
            | Request::NotWritten { .. }
            | Request::ReadAgain { .. }
            | Request::Answer { .. }
            | Request::ServiceCredential { .. }
            | Request::OpenStream { .. }
            | Request::Signing { .. }
            | Request::CloseStream
            | Request::Watch { .. }
            | Request::FamilyDelivery { .. }
            | Request::Ended { .. }
            | Request::Hello { .. }
            | Request::Fact { .. } => {}
        }
    }
    panic!("one batch: {:?}", h.out)
}

fn part(h: &mut fixtures::Harness, exchange: Token, member: u16, reply: Option<Box<[u8]>>) {
    let record = w::BatchReply { member, reply };
    let mut writer = Writer::new(
        usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
    h.link.step(Time::ZERO, Event::Part { exchange, message: writer.finish() }, &mut h.out);
}
fn result(h: &fixtures::Harness, token: u64) -> Option<&Readout> {
    for request in &h.out {
        match request {
            Request::Read { read, result } if *read == Token::new(token) => return Some(result),
            Request::Exchange { .. }
            | Request::Read { .. }
            | Request::StreamsFull { .. }
            | Request::Authorized(_)
            | Request::AskedLink(_)
            | Request::Approval(_)
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Kept { .. }
            | Request::Dropped
            | Request::NotKept { .. }
            | Request::NotWritten { .. }
            | Request::ReadAgain { .. }
            | Request::Answer { .. }
            | Request::ServiceCredential { .. }
            | Request::OpenStream { .. }
            | Request::Signing { .. }
            | Request::CloseStream
            | Request::Watch { .. }
            | Request::FamilyDelivery { .. }
            | Request::Ended { .. }
            | Request::Hello { .. }
            | Request::Fact { .. } => {}
        }
    }
    None
}
// The composing client retries a current lost read once before reporting unreachable.
fn exhaust_lost_reads(h: &mut fixtures::Harness) -> u32 {
    let mut pending = core::mem::replace(&mut h.out, Queue::uncapped());
    let mut count = 0_u32;
    while let Some(request) = pending.pop() {
        if let Request::ReadAgain { read, what } = request {
            count = count.saturating_add(1);
            h.link.step(Time::ZERO, Event::ReadAgain { read, what }, &mut h.out);
        } else {
            h.out.push(request);
        }
    }
    h.link.settle(&mut pending);
    while let Some(request) = pending.pop() {
        if let Request::Exchange { exchange, .. } = request {
            h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out);
        } else {
            h.out.push(request);
        }
    }
    count
}
fn read_count(h: &fixtures::Harness) -> u32 {
    let mut count = 0_u32;
    for request in &h.out {
        match request {
            Request::Read { .. } => count = count.saturating_add(1),
            Request::Exchange { .. }
            | Request::StreamsFull { .. }
            | Request::Authorized(_)
            | Request::AskedLink(_)
            | Request::Approval(_)
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Kept { .. }
            | Request::Dropped
            | Request::NotKept { .. }
            | Request::NotWritten { .. }
            | Request::ReadAgain { .. }
            | Request::Answer { .. }
            | Request::ServiceCredential { .. }
            | Request::OpenStream { .. }
            | Request::Signing { .. }
            | Request::CloseStream
            | Request::Watch { .. }
            | Request::FamilyDelivery { .. }
            | Request::Ended { .. }
            | Request::Hello { .. }
            | Request::Fact { .. } => {}
        }
    }
    count
}

#[test]
fn a_continued_watch_waits_for_its_opening_and_cleanup_before_reusing_watch_room() {
    let limits = Limits::new(jig_client_domain::Limits { watches: 2, ..*fixtures::limits().client() }, 4).unwrap();
    let mut h = fixtures::Harness::new(&limits);
    h.open(fixtures::WATCH, fixtures::application_subject());
    h.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
    let (opening, old) = h.opening();
    h.clear();
    h.link.step(Time::ZERO, Event::Continue { watch: fixtures::WATCH, subject: c::Subject::Me }, &mut h.out);
    h.link.settle(&mut h.out);
    assert_eq!(h.exchanges(), 0, "neither the cleanup nor its replacement may overtake admission");
    h.answer(opening, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    let (closing, name) = h.unwatching();
    assert_eq!(name, old);
    assert_eq!(h.exchanges(), 1, "only the admitted watch's closing leaves");
    h.clear();
    h.answer(closing, &fixtures::close_answer());
    let (_, new) = h.opening();
    assert_ne!(new, old);
    assert_eq!(h.closed_count(), 0, "the old closing does not end its replacement");
}

#[test]
fn cancelling_a_held_read_keeps_later_reads_openings_and_closings_in_asked_order() {
    let limits = Limits::new(*fixtures::limits().client(), 1).unwrap();
    let mut h = fixtures::Harness::new(&limits);
    h.open(fixtures::WATCH, fixtures::application_subject());
    h.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
    let (opening, _) = h.opening();
    h.answer(opening, &fixtures::open_answer(&w::OpenAnswer::Admitted));
    h.clear();
    gather(&mut h, 100);
    h.link.settle(&mut h.out);
    let Some(Request::Exchange { exchange: blocker, .. }) = h.out.pop() else { panic!("blocker") };
    gather(&mut h, 1);
    h.link.step(Time::ZERO, Event::Open { watch: Token::new(90), subject: fixtures::family_subject() }, &mut h.out);
    h.link.step(Time::ZERO, Event::Close { watch: fixtures::WATCH }, &mut h.out);
    gather(&mut h, 3);
    h.link.cancel_read(Token::new(1));
    h.link.cancel_read(Token::new(u64::MAX));
    h.link.settle(&mut h.out);
    assert!(h.out.is_empty());
    h.link.step(Time::ZERO, Event::Lost { exchange: blocker }, &mut h.out);
    h.link.settle(&mut h.out);
    let (exchange, record) = batch(&h);
    let [w::Member::Open(_), w::Member::Close(_), w::Member::Read(read)] = record.members.as_slice() else {
        panic!("remaining asked order")
    };
    let w::Read::Project(read) = w::Read::decode(limits.wire(), &mut Reader::new(read.record.as_ref())).unwrap() else {
        panic!("project read")
    };
    assert_eq!(read.project, 3);
    h.clear();
    h.link.cancel_read(Token::new(3));
    assert_eq!(h.link.in_flight(), 1, "a sent member keeps credit");
    part(&mut h, exchange, 2, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
    assert_eq!(result(&h, 3), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Busy))));
    assert!(result(&h, 1).is_none(), "a cancelled held read has no output");
}

#[test]
fn three_reads_and_an_opening_leave_as_one_ordered_batch() {
    let mut h = fixtures::Harness::admitted_stream();
    for n in 1..=3 {
        gather(&mut h, n);
    }
    h.link.step(Time::ZERO, Event::Open { watch: Token::new(90), subject: fixtures::family_subject() }, &mut h.out);
    assert_eq!(h.exchanges(), 0);
    h.link.settle(&mut h.out);
    let (_, b) = batch(&h);
    let [w::Member::Read(_), w::Member::Read(_), w::Member::Read(_), w::Member::Open(_)] = b.members.as_slice() else {
        panic!("ordered members")
    };
    for (place, member) in b.members.as_slice().iter().take(3).enumerate() {
        let w::Member::Read(record) = member else { panic!("read prefix") };
        let w::Read::Project(read) =
            w::Read::decode(h.link.limits.wire(), &mut Reader::new(record.record.as_ref())).unwrap()
        else {
            panic!("project read")
        };
        assert_eq!(read.project, u32::try_from(place).unwrap() + 1);
    }
    assert_eq!(h.link.in_flight(), 1);
    h.clear();
    gather(&mut h, 4);
    h.link.settle(&mut h.out);
    let Some(Request::Exchange { message: Message::Read(_), .. }) = h.out.pop() else { panic!("single read") };
}

#[test]
fn reads_wait_during_starting_then_leave_alone_or_batched() {
    for count in [1, 2] {
        let mut h = fixtures::Harness::new(&fixtures::limits());
        h.link.step(Time::ZERO, Event::OpenStream, &mut h.out);
        for number in 1..=count {
            gather(&mut h, number);
        }
        h.link.settle(&mut h.out);
        assert!(h.has(&Request::OpenStream { credential: None, version: w::VERSION, replaces: None }));
        assert_eq!(h.exchanges(), 0);
        assert_eq!(read_count(&h), 0);
        h.clear();

        h.stream_record(&fixtures::hello_event(fixtures::HEARTBEAT));
        assert_eq!(h.exchanges(), 1);
        let mut members = None;
        let mut batched = false;
        for request in &h.out {
            if let Request::Exchange { message: Message::Read(body), .. } = request {
                let owned = w::Owned::decode(h.link.limits.wire(), &mut Reader::new(body)).unwrap();
                members = Some(List::from_boxed(Box::new([w::Member::Read(owned)])));
            }
            if let Request::Exchange { message: Message::Batch(body), .. } = request {
                members = Some(w::Batch::decode(h.link.limits.wire(), &mut Reader::new(body)).unwrap().members);
                batched = true;
            }
        }
        assert_eq!(batched, count > 1);
        let members = members.expect("reads released by matching Opened");
        assert_eq!(u64::from(members.len()), count);
        for (number, member) in (1..=count).zip(&members) {
            let w::Member::Read(owned) = member else { panic!("read member") };
            let value = w::Read::decode(h.link.limits.wire(), &mut Reader::new(&owned.record)).unwrap();
            assert_eq!(value, w::Read::Project(w::ReadProject { project: u32::try_from(number).unwrap() }));
        }
    }
}

#[test]
fn sixty_four_members_take_one_credit_and_the_rest_wait_in_order() {
    let base = fixtures::limits();
    let limits = Limits::new(jig_client_domain::Limits { reads: 70, ..*base.client() }, 1).unwrap();
    let mut h = fixtures::Harness::with_hello(&limits);
    for n in 1..=67 {
        gather(&mut h, n);
    }
    h.link.settle(&mut h.out);
    let (exchange, b) = batch(&h);
    assert_eq!(b.members.len(), 64);
    assert_eq!(h.link.in_flight(), 1);
    h.clear();
    part(&mut h, exchange, 0, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
    assert_eq!(h.link.in_flight(), 1, "a part is not a transport terminal");
    h.link.settle(&mut h.out);
    assert_eq!(h.exchanges(), 0);
    h.link.step(Time::ZERO, Event::Done { exchange }, &mut h.out);
    h.clear();
    h.link.settle(&mut h.out);
    assert_eq!(batch(&h).1.members.len(), 3);
    assert_eq!(h.link.in_flight(), 1);
}

#[test]
fn batch_bytes_split_before_the_bound_and_an_oversize_read_leaves_alone() {
    let base = fixtures::limits();
    let limits =
        Limits::new(jig_client_domain::Limits { reads: 8, document_bytes: 262_144, ..*base.client() }, 4).unwrap();
    let mut h = fixtures::Harness::with_hello(&limits);
    for (n, size) in [60_000_usize, 60_000, 20_000, 140_000].into_iter().enumerate() {
        h.link.step(
            Time::ZERO,
            Event::Read {
                read: Token::new(u64::try_from(n).unwrap()),
                what: Readable::Family {
                    scope: c::Scope::Project(1),
                    window: c::Window { rows: 1, after: None },
                    record: Family { family: 19, version: 23, bytes: skein_lib::bytes::zeroed(size) },
                },
            },
            &mut h.out,
        );
    }
    h.link.settle(&mut h.out);
    let mut messages = List::with_capacity(h.out.len());
    for _ in 0..h.out.len() {
        let request = h.out.pop().unwrap();
        messages.push(request).unwrap();
    }
    let [
        Request::Exchange { message: Message::Batch(b), .. },
        Request::Exchange { message: Message::Read(small), .. },
        Request::Exchange { message: Message::Read(large), .. },
    ] = messages.as_slice()
    else {
        panic!("split: {messages:?}")
    };
    assert_eq!(w::Batch::decode(limits.wire(), &mut Reader::new(b)).unwrap().members.len(), 2);
    assert!(b.len() <= usize::try_from(BATCH_BYTES).unwrap());
    assert!(small.len() < usize::try_from(BATCH_BYTES).unwrap());
    assert!(large.len() > usize::try_from(BATCH_BYTES).unwrap());
}

#[test]
fn parts_route_by_member_and_a_cut_batch_keeps_its_answered_reads() {
    for done in [false, true] {
        let mut h = fixtures::Harness::with_hello(&fixtures::limits());
        for n in 1..=3 {
            gather(&mut h, n);
        }
        h.link.settle(&mut h.out);
        let (exchange, _) = batch(&h);
        h.clear();
        part(&mut h, exchange, 1, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
        assert_eq!(result(&h, 2), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Busy))));
        assert!(result(&h, 1).is_none());
        part(&mut h, exchange, 2, None);
        h.link.step(Time::ZERO, if done { Event::Done { exchange } } else { Event::Lost { exchange } }, &mut h.out);
        assert_eq!(exhaust_lost_reads(&mut h), 2, "only unanswered members retry");
        assert_eq!(result(&h, 1), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable))));
        assert_eq!(result(&h, 3), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable))));
        assert_eq!(read_count(&h), 3);
        assert_eq!(h.link.in_flight(), 0);
    }
}

#[test]
fn a_family_members_busy_or_lost_reply_keeps_the_transient_reason() {
    for ending in 0_u8..4 {
        let mut h = fixtures::Harness::with_hello(&fixtures::limits());
        gather(&mut h, 1);
        h.link.step(
            Time::ZERO,
            Event::Read {
                read: Token::new(2),
                what: Readable::Family {
                    scope: c::Scope::Project(1),
                    window: c::Window { rows: 1, after: None },
                    record: fixtures::family_read(),
                },
            },
            &mut h.out,
        );
        h.link.settle(&mut h.out);
        let (exchange, _) = batch(&h);
        h.clear();
        part(&mut h, exchange, 0, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
        match ending {
            0 => {
                part(&mut h, exchange, 1, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
                h.link.step(Time::ZERO, Event::Done { exchange }, &mut h.out);
            }
            1 => {
                part(&mut h, exchange, 1, None);
                h.link.step(Time::ZERO, Event::Done { exchange }, &mut h.out);
            }
            2 => h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out),
            _ => h.link.step(Time::ZERO, Event::Done { exchange }, &mut h.out),
        }
        assert_eq!(exhaust_lost_reads(&mut h), u32::from(ending != 0));
        let why = if ending == 0 { c::ReadNone::Busy } else { c::ReadNone::Unreachable };
        assert_eq!(result(&h, 2), Some(&Readout::Jig(c::ReadResult::None(why))));
        assert_eq!(result(&h, 1), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Busy))));
        assert_eq!(read_count(&h), 2);
    }
}

#[test]
fn an_invalid_or_repeated_part_loses_unanswered_members_until_the_terminal() {
    for bad in 0_u8..3 {
        let mut h = fixtures::Harness::with_hello(&fixtures::limits());
        for n in 1..=3 {
            gather(&mut h, n);
        }
        h.link.settle(&mut h.out);
        let (exchange, _) = batch(&h);
        h.clear();
        part(&mut h, exchange, 1, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
        match bad {
            0 => part(&mut h, exchange, 1, Some(fixtures::read_none_answer(w::ReadNone::Busy))),
            1 => part(&mut h, exchange, 64, None),
            _ => h.link.step(Time::ZERO, Event::Part { exchange, message: Box::from([255]) }, &mut h.out),
        }
        assert_eq!(exhaust_lost_reads(&mut h), 2, "answered members do not retry");
        assert_eq!(result(&h, 1), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable))));
        assert_eq!(result(&h, 2), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Busy))));
        part(&mut h, exchange, 0, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
        assert_eq!(read_count(&h), 3);
        assert_eq!(h.link.in_flight(), 1);
        h.link.step(Time::ZERO, Event::Done { exchange }, &mut h.out);
        assert_eq!(h.link.in_flight(), 0);
    }
}

#[test]
fn an_unanswered_opening_or_closing_drops_only_its_stream_not_answered_reads() {
    for closing in [false, true] {
        let mut h = fixtures::Harness::admitted_stream();
        gather(&mut h, 1);
        let watch = if closing { fixtures::WATCH } else { Token::new(90) };
        h.link.step(
            Time::ZERO,
            if closing {
                Event::Close { watch }
            } else {
                Event::Open { watch, subject: fixtures::application_subject() }
            },
            &mut h.out,
        );
        h.link.settle(&mut h.out);
        let (exchange, _) = batch(&h);
        h.clear();
        part(&mut h, exchange, 0, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
        h.link.step(Time::ZERO, Event::Done { exchange }, &mut h.out);
        assert_eq!(result(&h, 1), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Busy))));
        assert_eq!(h.ended(watch), Some(c::WatchEnd::Dropped));
        assert!(h.has(&Request::Fact { fact: Fact::StreamDropped }));
    }
}

#[test]
fn a_batched_opening_refused_or_not_served_is_that_watchs_alone() {
    for (answer, expected) in [
        (w::OpenAnswer::Refused(w::OpenAnswerRefused { why: w::WatchRefusal::NotAllowed }), c::WatchEnd::NotAllowed),
        (w::OpenAnswer::NotServed(fixtures::not_served()), c::WatchEnd::NotServed),
    ] {
        let mut h = fixtures::Harness::admitted_stream();
        gather(&mut h, 1);
        let watch = Token::new(90);
        h.link.step(Time::ZERO, Event::Open { watch, subject: fixtures::family_subject() }, &mut h.out);
        h.link.settle(&mut h.out);
        let (exchange, _) = batch(&h);
        h.clear();
        part(&mut h, exchange, 1, Some(fixtures::open_answer(&answer)));
        assert_eq!(h.ended(watch), Some(expected));
        assert_eq!(h.ended(fixtures::WATCH), None);
        assert!(!h.has(&Request::Fact { fact: Fact::StreamDropped }));
        part(&mut h, exchange, 0, Some(fixtures::read_none_answer(w::ReadNone::Busy)));
        assert_eq!(result(&h, 1), Some(&Readout::Jig(c::ReadResult::None(c::ReadNone::Busy))));
    }
}

#[test]
fn every_stream_opening_names_the_last_known_stream_without_a_gone_exchange() {
    let limits = Limits::new(*fixtures::limits().client(), 1).unwrap();
    let mut h = fixtures::Harness::new(&limits);
    let mut kept = List::with_capacity(1);
    kept.push(fixtures::kept_stream()).unwrap();
    h.link.step(Time::ZERO, Event::Start { kept }, &mut h.out);
    gather(&mut h, 1);
    h.link.settle(&mut h.out);
    h.clear();
    gather(&mut h, 2);
    h.open(fixtures::WATCH, fixtures::application_subject());
    assert!(h.has(&Request::OpenStream {
        credential: None,
        replaces: Some(fixtures::STREAM),
        version: jig_client_protocol_wire::VERSION
    }));
    assert_eq!(h.exchanges(), 0);
    let hello =
        w::StreamEvent::decode(h.link.limits.wire(), &mut Reader::new(&fixtures::hello_event(fixtures::HEARTBEAT)))
            .unwrap();
    let w::StreamEvent::Opened(opened) = hello else { panic!("opened") };
    let mut fields = opened;
    let mut hello = fields.hello;
    hello.stream = [9; 16];
    fields.hello = hello;
    h.stream_record(&fixtures::event_bytes(&w::StreamEvent::Opened(fields)));
    h.clear();
    h.link.step(Time::ZERO, Event::StreamGone, &mut h.out);
    h.open(Token::new(99), fixtures::application_subject());
    assert!(h.has(&Request::OpenStream {
        credential: None,
        replaces: Some([9; 16]),
        version: jig_client_protocol_wire::VERSION
    }));
    assert_eq!(h.exchanges(), 0);
}
