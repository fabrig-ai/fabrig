use super::*;

fn compound(count: u32) -> Ask {
    let mut operations = List::with_capacity(count);
    for task in 1..=count {
        operations
            .push(Operation::Stop { task: u64::from(task), reason: Box::from(b"Paused".as_slice()) })
            .expect("test operation room");
    }
    Ask::Compound { project: 1, operations }
}

#[test]
fn a_compound_past_operations_per_request_is_refused() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    let refused = test.request(10, 1, compound(3));
    assert_eq!(reply(&refused), Reply::Refused(Refusal::Limit { which: LimitKind::Operations }));
}

#[test]
fn an_empty_compound_is_refused() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    let refused = test.request(10, 1, compound(0));
    assert_eq!(reply(&refused), Reply::Refused(Refusal::Limit { which: LimitKind::Operations }));
}

#[test]
fn a_compound_sent_twice_under_one_key_is_routed_once() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    test.send(Event::Roles { project: 1, holdings: Box::new([]) });
    let request = route(&test.request(10, 1, compound(2)));
    assert_eq!(test.request(10, 1, compound(2)).len(), 0);
    let outcome = Outcome::Compound { project: 1, operations: 2, first_message: 1 };
    let replies = test.send(Event::Decided { request, outcome: outcome.clone() });
    let mut answers = 0_u32;
    for row in replies {
        if let Request::Reply { reply, .. } = row {
            assert_eq!(reply, Reply::Outcome(outcome.clone()));
            answers += 1;
        }
    }
    assert_eq!(answers, 2);
    assert_eq!(reply(&test.request(10, 1, compound(2))), Reply::Outcome(outcome));
}

const INVALID: &[(u64, &[u8], Refusal)] = &[
    (0, b"words", Refusal::Unknown),
    (1, b"", Refusal::Limit { which: LimitKind::Empty }),
    (1, b"too many words", Refusal::Limit { which: LimitKind::Words }),
];
#[test]
fn a_compounds_invalid_operation_keeps_its_standalone_refusal() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    for &(task, words, ref refusal) in INVALID {
        let mut operations = List::with_capacity(1);
        operations.push(Operation::Message { task, words: words.into() }).expect("one operation");
        let rows = test.request(10, 1, Ask::Compound { project: 1, operations });
        assert_eq!(reply(&rows), Reply::Refused(refusal.clone()), "task {task}, words {words:?}");
    }
}

#[test]
fn stops_require_nonempty_bounded_utf8_reasons_before_routing() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    test.send(Event::Roles { project: 1, holdings: Box::new([]) });
    for (reason, which) in [
        (b"".as_slice(), LimitKind::Empty),
        (b"Too many words".as_slice(), LimitKind::Words),
        (b"\xff".as_slice(), LimitKind::Words),
    ] {
        let expected = Reply::Refused(Refusal::Limit { which });
        let stop = Ask::Stop { project: 1, task: 8, reason: reason.into() };
        let rows = test.request(10, 1, stop);
        let [Request::Reply { .. }] = rows.as_slice() else { panic!("only a refusal: {rows:?}") };
        assert_eq!(reply(&rows), expected);

        let mut operations = List::with_capacity(2);
        operations.push(Operation::Stop { task: 7, reason: b"Paused".as_slice().into() }).unwrap();
        operations.push(Operation::Stop { task: 8, reason: reason.into() }).unwrap();
        let rows = test.request(10, 1, Ask::Compound { project: 1, operations });
        let [Request::Reply { .. }] = rows.as_slice() else { panic!("only a refusal: {rows:?}") };
        assert_eq!(reply(&rows), expected);
    }
    route(&test.request(10, 1, Ask::Stop { project: 1, task: 8, reason: b"Paused".as_slice().into() }));
}
