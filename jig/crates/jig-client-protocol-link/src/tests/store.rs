use super::*;
fn restored(expected: &Saved) -> fixtures::Harness {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.start_store(fixtures::one_store_read(fixtures::store_read(expected.clone())));
    harness
}
fn pending_write(record: Saved) -> fixtures::Harness {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.save(record);
    harness.send_kept(fixtures::ATTEMPT);
    harness
}
#[test]
fn a_table_entry_restores_its_key_and_ask() {
    for ask in fixtures::store_asks().into_boxed() {
        let expected = fixtures::saved_table(ask);
        assert!(restored(&expected).restored(&expected));
    }
}
#[test]
fn a_record_of_another_store_version_is_dropped_and_told() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.start_store(fixtures::one_store_read(fixtures::other_store_version(fixtures::store_read(
        fixtures::saved_table(fixtures::message()),
    ))));
    assert!(harness.has(&Request::Dropped));
    assert_eq!(harness.out.len(), 1);
}
#[test]
fn a_request_past_64_kib_is_saved_before_it_is_sent() {
    const REQUEST_BYTES: usize = 85_000;
    let base = fixtures::limits();
    let client = jig_client_domain::Limits { document_bytes: 1_048_576, ask_bytes: 131_328, ..*base.client() };
    let limits = Limits::new(client, base.exchanges()).unwrap();
    let mut harness = fixtures::Harness::new(&limits);
    let ask = Ask::Family {
        scope: jig_client_domain::contract::Scope::Project(1),
        record: Family { family: 19, version: 23, bytes: skein_lib::bytes::zeroed(REQUEST_BYTES) },
    };
    let mut saved = fixtures::saved_table(fixtures::message());
    saved.record.request.request = ask.clone();
    harness.save(saved);
    harness.link.step(
        Time::ZERO,
        Event::Send { attempt: fixtures::ATTEMPT, key: fixtures::KEY, ask, value: None },
        &mut harness.out,
    );
    assert_eq!(harness.out.len(), 1, "one Save before sending");
    let Some(Request::Save { record }) = harness.out.iter().next() else { panic!("one Save") };
    assert!(record.bytes.len() > 65_536);
    assert!(record.bytes.len() < 86_000);
    assert_eq!(harness.exchanges(), 0);
    harness.written();
    assert_eq!(harness.exchanges(), 1);
    assert_eq!(harness.out.len(), 2, "one Save and one exchange");
}

#[test]
fn a_step_s_writes_leave_before_its_exchanges() {
    let mut harness = pending_write(fixtures::saved_table(fixtures::message()));
    assert_eq!(harness.exchanges(), 0);
    harness.written();
    let Some(Request::Save { .. }) = harness.out.iter().next() else { panic!("write leaves first") };
    assert_eq!(harness.exchanges(), 1);
}
#[test]
fn a_write_gathers_exchanges_already_in_the_step_s_output_queue() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.send_kept(fixtures::ATTEMPT);
    harness.save(fixtures::saved_table(fixtures::message()));
    assert_eq!(harness.exchanges(), 0);
    harness.written();
    let Some(Request::Save { .. }) = harness.out.iter().next() else { panic!("write leaves first") };
    assert_eq!(harness.exchanges(), 1);
}
#[test]
fn a_failed_write_withholds_the_step_s_exchanges() {
    let mut harness = pending_write(fixtures::saved_table(fixtures::message()));
    harness.write_failed();
    harness.written();
    assert_eq!(harness.exchanges(), 0);
    assert!(harness.has(&Request::NotKept { attempt: fixtures::ATTEMPT }));
    assert_eq!(harness.terminals(), 1);
}
#[test]
fn an_unwritable_record_retires_only_its_key_and_preserves_the_pending_write() {
    let first = fixtures::saved_table(fixtures::message());
    let base = fixtures::limits();
    let client = jig_client_domain::Limits {
        document_bytes: u64::try_from(fixtures::store_read(first.clone()).bytes.len()).unwrap(),
        ..*base.client()
    };
    let limits = Limits::new(client, base.exchanges()).unwrap();
    let mut harness = fixtures::Harness::new(&limits);
    harness.save(first);
    harness.send_kept(fixtures::ATTEMPT);
    assert_eq!(harness.exchanges(), 0);
    harness.clear();
    let ask = c::Ask::Message {
        project: 7,
        task: 8,
        words: c::Text::new(Box::from(b"longer than hello".as_slice()), 32).unwrap(),
    };
    let mut second = fixtures::saved_table(ask.clone());
    second.record.key = [8; 16];
    harness.save(second);
    assert!(!harness.has(&Request::NotKept { attempt: fixtures::ATTEMPT }), "the encoded request stays held");
    assert!(!harness.has(&Request::Fact { fact: Fact::UndecodableStore }));
    assert_eq!(harness.out.len(), 1, "only the unwritten key, no save");
    assert!(harness.has(&Request::NotWritten { key: [8; 16] }));
    harness.link.step(
        Time::ZERO,
        Event::Send { attempt: Token::new(18), key: [8; 16], ask: Ask::Jig(Box::new(ask)), value: None },
        &mut harness.out,
    );
    assert_eq!(harness.exchanges(), 0);
    assert_eq!(harness.out.len(), 1, "the following exchange retires silently");
    assert_eq!(harness.link.attempts.len(), 1, "only A still owns exchange credit");
    harness.written();
    assert_eq!(harness.exchanges(), 1);
    assert_eq!(harness.out.len(), 2, "B is named once and A leaves once");
    for request in &harness.out {
        if let Request::Exchange { message: Message::Request(bytes), .. } = request {
            let keyed = w::Keyed::decode(limits.wire(), &mut Reader::new(bytes)).unwrap();
            assert_eq!(keyed.key(), &fixtures::KEY);
        }
    }
    assert!(!harness.has(&Request::NotKept { attempt: fixtures::ATTEMPT }));
    harness.clear();
    harness.written();
    assert_eq!(harness.out.len(), 0, "nothing leaves twice");
}
#[test]
fn a_table_entry_restores_family_bytes_and_their_wire_version() {
    let expected = Saved {
        version: fixtures::TABLE_VERSION,
        record: skein_lib::RequestRecord {
            key: fixtures::KEY,
            scope: u64::MAX,
            first_sent: None,
            request: jig_client_domain::Held {
                key: fixtures::APPLICATION_KEY,
                request: Ask::Family {
                    scope: jig_client_domain::contract::Scope::Project(1),
                    record: fixtures::family_read(),
                },
            },
        },
    };
    assert!(restored(&expected).restored(&expected));
}
#[test]
fn an_erased_key_waits_for_durability_before_the_step_s_exchange() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.link.step(Time::ZERO, Event::Erase { key: fixtures::KEY }, &mut harness.out);
    harness.link.settle(&mut harness.out);
    harness.send_kept(fixtures::ATTEMPT);
    assert_eq!(harness.exchanges(), 0);
    harness.written();
    let Some(Request::Erase { .. }) = harness.out.iter().next() else { panic!("erase leaves first") };
    assert_eq!(harness.exchanges(), 1);
}
#[test]
fn malformed_kept_bytes_drop_the_owner_and_are_reported() {
    let mut entry = fixtures::store_read(fixtures::saved_table(fixtures::message()));
    entry.bytes = Box::from([255]);
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.start_store(fixtures::one_store_read(entry));
    assert!(harness.has(&Request::Dropped));
    assert!(harness.has(&Request::Fact { fact: Fact::UndecodableStore }));
    assert_eq!(harness.out.len(), 2);
}
#[test]
fn a_mismatched_table_storage_key_drops_its_record() {
    let mut entry = fixtures::store_read(fixtures::saved_table(fixtures::message()));
    let length = entry.key.len();
    *entry.key.get_mut(length - 1).unwrap() = 0;
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.start_store(fixtures::one_store_read(entry));
    assert!(harness.has(&Request::Dropped));
    assert!(harness.has(&Request::Fact { fact: Fact::UndecodableStore }));
}
#[test]
fn a_cancelled_opening_already_in_the_step_queue_is_never_sent_after_writing() {
    let mut harness = fixtures::Harness::streaming(fixtures::family_subject());
    let (exchange, _) = harness.opening();
    harness.clear();
    harness.link.step(Time::ZERO, Event::Lost { exchange }, &mut harness.out);
    harness.link.settle(&mut harness.out);
    harness.save(fixtures::saved_table(fixtures::message()));
    harness.written();
    assert_eq!(harness.exchanges(), 0, "no stream gone is sent");
    harness.assert_no_watch_exchange();
    assert_eq!(harness.ended(fixtures::WATCH), Some(c::WatchEnd::Dropped));
}

#[test]
fn a_full_table_and_the_stream_s_name_start() {
    const TABLE_RECORDS: u32 = 4;
    const KEPT_RECORDS: u32 = TABLE_RECORDS + 1;
    let mut h = fixtures::Harness::new(&fixtures::limits());
    let mut kept = skein_lib::List::with_capacity(KEPT_RECORDS);
    assert_eq!(fixtures::limits().client().requests.requests, TABLE_RECORDS);
    for number in 0..TABLE_RECORDS {
        let mut record = fixtures::saved_table(fixtures::message());
        record.record.key = [u8::try_from(number).unwrap(); 16];
        kept.push(fixtures::store_read(record)).unwrap();
    }
    kept.push(fixtures::kept_stream()).unwrap();
    h.start_store(kept);
    assert_eq!(h.out.len(), TABLE_RECORDS);
    for request in &h.out {
        let Request::Kept { .. } = request else { panic!("restored table entry: {request:?}") };
    }
    h.clear();
    h.open(fixtures::WATCH, fixtures::application_subject());
    assert_eq!(h.exchanges(), 0);
    assert!(h.has(&Request::OpenStream {
        credential: None,
        replaces: Some(fixtures::STREAM),
        version: jig_client_protocol_wire::VERSION
    }));
}

#[test]
fn construction_refuses_unrepresentable_record_limits() {
    let base = fixtures::limits();
    assert!(crate::store::valid(&base));
    let client = jig_client_domain::Limits { document_bytes: u64::MAX, ..*base.client() };
    assert_eq!(Limits::new(client, base.exchanges()), Err(Unusable::Limits));
}

/// O4: a completed write releases its admitted exchanges in arrival order.
#[test]
fn held_exchanges_leave_in_order_after_the_write() {
    let mut harness = fixtures::Harness::new(&fixtures::limits());
    harness.save(fixtures::saved_table(fixtures::message()));
    for number in [1, 2, 3] {
        harness.link.step(
            Time::ZERO,
            Event::Send {
                attempt: Token::new(number),
                key: [u8::try_from(number).unwrap(); 16],
                ask: Ask::Jig(Box::new(fixtures::message())),
                value: None,
            },
            &mut harness.out,
        );
        harness.link.settle(&mut harness.out);
    }
    assert_eq!(harness.exchanges(), 0, "all exchanges await the write");
    harness.clear();
    harness.written();
    let mut keys = skein_lib::List::with_capacity(3);
    while let Some(request) = harness.out.pop() {
        let Request::Exchange { message: Message::Request(bytes), .. } = request else {
            panic!("only held keyed exchanges leave: {request:?}")
        };
        let keyed = w::Keyed::decode(fixtures::limits().wire(), &mut Reader::new(&bytes)).unwrap();
        keys.push(*keyed.key()).unwrap();
    }
    assert_eq!(keys.as_slice(), &[[1; 16], [2; 16], [3; 16]]);
    harness.written();
    assert_eq!(harness.exchanges(), 0, "a repeated answer releases nothing twice");
}
