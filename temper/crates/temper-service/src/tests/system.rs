use super::fixtures;
use crate::{End, EngineEnd};
use skein_lib::{Time, Token, Wall};

#[test]
fn the_worker_dials_workers_and_its_hello_reaches_the_engine() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    let hello = fixtures::hello(&stand.service).unwrap();
    assert_eq!(hello.0, Token::new(1));
    assert_eq!(hello.1.slots, 1);
    assert_eq!(hello.1.hosting.as_ref(), &[]);
    assert_eq!(stand.kernel.spawned, 1);
}

#[test]
fn the_worker_iterates_only_once_the_engine_listens() {
    let mut stand = fixtures::one_process();
    let opened = stand.service.engine_component().exchange_store(fixtures::unopened_store());
    crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
    assert!(!stand.service.worker.as_ref().unwrap().pending(Time::ZERO).started, "store still closed");
    stand.service.engine_component().replace_store(opened);
    crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
    assert!(!stand.service.worker.as_ref().unwrap().pending(Time::ZERO).started, "listener still opening");
    for _ in 0_u8..4 {
        stand.turn();
    }
    assert!(stand.service.engine.as_ref().expect("engine").workers_opened());
    assert!(stand.service.worker.as_ref().unwrap().pending(Time::ZERO).started);
}

#[test]
fn a_signal_drains_the_engine_then_ends_the_worker_at_once() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    stand.service.shutdown();
    assert!(!stand.service.worker.as_ref().unwrap().pending(Time::ZERO).aborted);
    for _ in 0_u16..256 {
        stand.turn();
        if stand.service.is_empty() {
            break;
        }
    }
    assert_eq!(stand.service.end(), Some(&End::Engine(EngineEnd::Drained)));
    assert!(stand.service.is_empty(), "the worker ended with the engine");
    assert!(stand.kernel.root_closed);
}

#[test]
fn a_second_signal_aborts_both_layers() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    stand.service.shutdown();
    stand.service.shutdown();
    assert_eq!(stand.service.end(), Some(&End::Engine(EngineEnd::Aborted)));
    assert!(stand.service.worker.as_ref().unwrap().pending(Time::ZERO).aborted);
    for _ in 0_u16..256 {
        stand.turn();
        if stand.service.is_empty() {
            break;
        }
    }
    assert!(stand.service.is_empty());
}

#[test]
fn io_events_and_completions_reach_the_layer_whose_tokens_they_carry() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    assert!(stand.service.worker.as_ref().unwrap().git_ready(), "git completions reached the worker");
    assert_eq!(stand.kernel.files, 9, "layout opens making while keeping stores");
    stand.service.process.told().push(skein_io::Event::Closed { owner: Token::new(crate::tokens::SERVER) });
    crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
    assert_eq!(
        stand.service.end(),
        Some(&End::Engine(EngineEnd::Stopped(crate::EngineStop::Listener {
            listener: crate::Listening::Clients,
            address: Some(skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into())),
            error: None
        }))),
        "clients event reached engine"
    );
    assert!(stand.service.worker.as_ref().unwrap().pending(Time::ZERO).aborted);
}

#[test]
fn an_unusable_git_stops_the_service_naming_its_program() {
    let mut stand = fixtures::one_process();
    stand.kernel.unusable = true;
    for _ in 0_u16..256 {
        stand.turn();
        if stand.service.is_empty() {
            break;
        }
    }
    let Some(End::Worker(crate::worker_alone::End::Stopped(crate::worker_alone::Stop::Start(problems)))) =
        stand.service.end()
    else {
        panic!("git refuses the start")
    };
    assert_eq!(
        stand.service.engine.as_ref().unwrap().end(),
        Some(&EngineEnd::Drained),
        "worker failure does not replace the engine end"
    );
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(
        problems.problems.get(0).unwrap().path,
        jig_protocol_config::Path::root()
            .member(b"application")
            .member(b"workers")
            .child(jig_protocol_config::Segment::Index(0))
            .member(b"git")
    );
    assert!(stand.service.is_empty());
}

#[test]
fn standalone_announces_clients_without_announcing_in_process_workers() {
    let mut stand = fixtures::one_process();
    fixtures::until_hosting(&mut stand);
    let addr = fixtures::engine().clients.listener;
    let jig_protocol_config::Address::Addr(addr) = addr else { panic!("clients socket") };
    let addr = addr.socket().unwrap();
    assert_eq!(
        stand.service.listening().pop(),
        Some(crate::Note::Engine(crate::EngineNote::Listening { listener: crate::Listening::Clients, addr }))
    );
    assert!(stand.service.listening().is_empty());
    stand.turn();
    assert!(stand.service.listening().is_empty());
}

#[test]
fn the_box_pipes_carry_no_service_data_and_close_on_drain_or_abort() {
    use skein_io::kernel::Fd;
    for abort in [false, true] {
        let mut stand = fixtures::one_process();
        stand.service.adopt_box(Fd::new(3), Fd::new(4), 128).unwrap();
        assert_eq!(stand.service.adopt_box(Fd::new(5), Fd::new(6), 128), Err(crate::system::BoxRefusal::Adopted));
        fixtures::until_hosting(&mut stand);
        let mut pending_reads = 0_u32;
        for submit in stand.service.submissions().iter() {
            if let skein_io::kernel::Op::PipeRead { fd, .. } = &submit.kind
                && *fd == Fd::new(4)
            {
                pending_reads = pending_reads.checked_add(1).unwrap();
            }
        }
        assert_eq!(pending_reads, 1, "io's read is still pending; nothing reached the service");
        assert!(!stand.kernel.box_read_cancelled);
        assert_eq!(stand.kernel.box_writes, 0);
        assert!(!stand.kernel.closed.as_slice().contains(&Fd::new(3)));
        assert!(!stand.kernel.closed.as_slice().contains(&Fd::new(4)));
        stand.service.shutdown();
        if abort {
            stand.service.shutdown();
        }
        for _ in 0_u16..256 {
            stand.turn();
            if stand.service.is_empty() {
                break;
            }
        }
        assert_eq!(
            stand.service.end(),
            Some(if abort { &End::Engine(EngineEnd::Aborted) } else { &End::Engine(EngineEnd::Drained) })
        );
        assert!(stand.service.is_empty());
        assert!(stand.kernel.box_read_cancelled, "close cancelled io's pending read");
        for note in stand.service.notes().iter() {
            if let crate::Note::Engine(crate::EngineNote::BoxLink { line }) = note {
                panic!("our close is not the box closing: {line:?}");
            }
        }
        assert_eq!(stand.kernel.box_writes, 0);
        for fd in [Fd::new(3), Fd::new(4)] {
            let mut count = 0_u32;
            for closed in &stand.kernel.closed {
                if *closed == fd {
                    count = count.checked_add(1).unwrap();
                }
            }
            assert_eq!(count, 1);
        }
    }
}

#[test]
fn a_scripted_box_answers_keyed_setup_requests_through_the_clients_family() {
    use temper_client_protocol_wire::v1 as w;
    use temper_engine_domain_forge::{Connection, SetUpStep};
    const OWNED: u16 = 1;
    const OTHER: u16 = 2;
    let mut stand = scripted_box();
    // SystemStand has fixed membership and an in-memory store. Startup's
    // absent-file state is tested separately; inject its root inputs here.
    for forge in [OWNED, OTHER] {
        stand.service.engine_component().setup_state(forge, Connection::NotSetUp);
    }
    turns(&mut stand);
    while stand.service.notes().pop().is_some() {}
    let bearer = box_sign_in(&mut stand);
    assert_eq!(box_setup(&mut stand, &bearer, OWNED, 1), w::Outcome::SettingUp);
    turns(&mut stand);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 1 owned\n");
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain().forge_connection(OWNED),
        Connection::SettingUp { step: None }
    );

    for byte in b"step 1 prepare\n" {
        stand.kernel.box_answers.push(*byte);
    }
    turns(&mut stand);
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain().forge_connection(OWNED),
        Connection::SettingUp { step: Some(SetUpStep::Prepare) }
    );
    assert_eq!(box_setup(&mut stand, &bearer, OWNED, 1), w::Outcome::SettingUp);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 1 owned\n");

    for byte in b"failed 1 prepare 1 exited 1\n" {
        stand.kernel.box_answers.push(*byte);
    }
    turns(&mut stand);
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain().forge_connection(OWNED),
        Connection::SetUpFailed { why: b"prepare 1 exited 1".as_slice().into() }
    );
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 1 owned\n");

    assert_eq!(box_setup(&mut stand, &bearer, OWNED, 2), w::Outcome::SettingUp);
    turns(&mut stand);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 1 owned\nstart 1 owned\n");
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain().forge_connection(OWNED),
        Connection::SettingUp { step: None }
    );

    let w::Outcome::SetUpRefused(refused) = box_setup(&mut stand, &bearer, OTHER, 3) else { panic!("busy answer") };
    assert_eq!(*refused.why(), w::SetUpRefusal::Busy);
    turns(&mut stand);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 1 owned\nstart 1 owned\n");
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain().forge_connection(OWNED),
        Connection::SettingUp { step: None }
    );
    assert_eq!(stand.service.engine.as_ref().unwrap().domain().forge_connection(OTHER), Connection::NotSetUp);
    assert!(stand.service.notes().is_empty());

    for byte in b"started 9\n" {
        stand.kernel.box_answers.push(*byte);
    }
    turns(&mut stand);
    assert_eq!(stand.kernel.box_written.as_slice(), b"start 1 owned\nstart 1 owned\n");
    assert_eq!(
        stand.service.engine.as_ref().unwrap().domain().forge_connection(OWNED),
        Connection::SettingUp { step: None }
    );
    assert_eq!(
        stand.service.notes().pop(),
        Some(crate::Note::Engine(crate::EngineNote::BoxLink {
            line: b"box link: dropped started for forge 9, not asked".as_slice().into()
        }))
    );
    assert!(stand.service.notes().is_empty());
}

#[test]
fn the_row_carries_the_latest_fetched_line_until_the_next_step() {
    use temper_client_protocol_wire::v1 as w;
    use temper_engine_domain_forge::{Connection, SetUpBinary, SetUpFetched, SetUpStep};
    const OWNED: u16 = 1;
    // Allow root continuations, memory-store handoffs, output credit and IO.
    fn line(stand: &mut fixtures::system::SystemStand, line: &[u8]) {
        for byte in line {
            stand.kernel.box_answers.push(*byte);
        }
        turns(stand);
    }
    fn fetched(stand: &fixtures::system::SystemStand) -> Option<SetUpFetched> {
        let rows = stand.service.engine.as_ref().unwrap().domain().forges();
        for row in &rows.rows {
            if row.number == OWNED {
                return row.fetched;
            }
        }
        panic!("owned forge row");
    }
    for (ending, expected) in [
        (b"step 1 prepare\n".as_slice(), Connection::SettingUp { step: Some(SetUpStep::Prepare) }),
        (b"started 1\n", Connection::Connecting),
        (b"failed 1 fetch exited 1\n", Connection::SetUpFailed { why: b"fetch exited 1".as_slice().into() }),
    ] {
        let mut stand = scripted_box();
        turns(&mut stand);
        while let Some(read) = stand.service.take_handover_read() {
            stand.service.handover_read(read.forge, read.run, Ok(None));
        }
        turns(&mut stand);
        stand.service.engine_component().setup_state(OWNED, Connection::NotSetUp);
        line(&mut stand, b"");
        while stand.service.notes().pop().is_some() {}
        let bearer = box_sign_in(&mut stand);
        assert_eq!(box_setup(&mut stand, &bearer, OWNED, 1), w::Outcome::SettingUp);
        line(&mut stand, b"step 1 fetch\n");
        assert_eq!(stand.kernel.box_written.as_slice(), b"start 1 owned\n");
        assert_eq!(fetched(&stand), None);
        for (bytes, expected) in [
            (
                b"fetched 1 forgejo 10 100\n".as_slice(),
                SetUpFetched { binary: SetUpBinary::Forgejo, done: 10, total: Some(100) },
            ),
            (
                b"fetched 1 forgejo 100 100\n",
                SetUpFetched { binary: SetUpBinary::Forgejo, done: 100, total: Some(100) },
            ),
            (b"fetched 1 runner 20 none\n", SetUpFetched { binary: SetUpBinary::Runner, done: 20, total: None }),
        ] {
            line(&mut stand, bytes);
            assert_eq!(fetched(&stand), Some(expected), "{bytes:?}, ending {ending:?}");
            line(&mut stand, b"");
            assert_eq!(fetched(&stand), Some(expected), "progress remains between lines");
        }
        line(&mut stand, ending);
        assert_eq!(stand.service.engine.as_ref().unwrap().domain().forge_connection(OWNED), expected, "{ending:?}");
        assert_eq!(fetched(&stand), None, "{ending:?}");
    }
}

fn scripted_box() -> fixtures::system::SystemStand {
    use skein_io::kernel::Fd;
    let mut stand = fixtures::system::one_process_under_box(true);
    stand.service.adopt_box(Fd::new(3), Fd::new(4), fixtures::system::BOX_BYTES).unwrap();
    fixtures::until_hosting(&mut stand);
    stand
}

// Allow root continuations and memory-store handoffs, then output credit and IO.
fn turns(stand: &mut fixtures::system::SystemStand) {
    const TURNS: u8 = 64;
    for _ in 0..TURNS {
        stand.turn();
    }
}

// The system story uses the actual clients component: an HTTP family request
// crosses the core's key admission, and FromApplication::Answered returns its bytes.
fn box_sign_in(stand: &mut fixtures::system::SystemStand) -> Box<[u8]> {
    use fixtures::http as peer;
    let link = stand.service.links().pop().expect("sign-in link");
    let engine = stand.service.engine.as_mut().unwrap();
    let body = peer::link_body(engine, &link);
    let mut sign_in = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    sign_in.pump(engine, true);
    let jig_client_protocol_wire::v22::SignInAnswer::SignedIn(answer) = sign_in.sign_in_answer(engine) else {
        panic!("deployment owner signed in")
    };
    peer::bearer(answer.token().as_ref().expect("native credential").expose())
}

fn box_setup(
    stand: &mut fixtures::system::SystemStand,
    bearer: &[u8],
    forge: u16,
    key: u8,
) -> temper_client_protocol_wire::v1::Outcome {
    use fixtures::http as peer;
    use jig_client_protocol_wire::v22 as j;
    use skein_lib::{Reader, Writer};
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let engine = stand.service.engine.as_mut().unwrap();
    let limits = engine.client_wire();
    let request = w::Requesting::new(
        &w::CEILINGS,
        w::RequestingParts {
            request: w::Request::SetUp(
                w::SetUp::new(
                    &w::CEILINGS,
                    w::SetUpParts {
                        forge: w::Making::Kept(w::Kept::new(&w::CEILINGS, w::KeptParts { forge }).unwrap()),
                        first: w::SetUpFirst::Empty(
                            w::FirstEmpty::new(
                                &w::CEILINGS,
                                w::FirstEmptyParts {
                                    playground: false,
                                    org: b"work".as_slice().into(),
                                    repository: b"hello".as_slice().into(),
                                },
                            )
                            .expect("setup names fit wire ceilings"),
                        ),
                    },
                )
                .unwrap(),
            ),
        },
    )
    .unwrap();
    let family = j::FamilyRequest::new(
        limits,
        j::FamilyRequestParts { scope: j::Scope::Deployment, record: wire::requesting(&request) },
    )
    .unwrap();
    let mut bytes = Writer::new(usize::try_from(family.measure()).unwrap());
    family.encode(&mut bytes).unwrap();
    let request = j::Keyed::new(
        limits,
        j::KeyedParts {
            key: [key; 16],
            owner: j::Owner::Family(j::Family::new(limits, j::FamilyParts { number: wire::FORGE }).unwrap()),
            version: wire::CLIENT_VERSION,
            request: bytes.finish(),
            value: None,
        },
    )
    .unwrap();
    let mut bytes = Writer::new(usize::try_from(request.measure()).unwrap());
    request.encode(&mut bytes).unwrap();
    let mut client = peer::Peer::new(engine, b"POST", b"/jig/v1/request", &bytes.finish(), Some(bearer));
    client.pump(engine, true);
    let at = skein_lib::bytes::find(client.received(), b"\r\n\r\n").unwrap().checked_add(4).unwrap();
    let mut reader = Reader::new(client.received().get(at..).unwrap());
    let j::KeyedAnswer::FinalAnswer(answer) = j::KeyedAnswer::decode(engine.client_wire(), &mut reader).unwrap() else {
        panic!("one terminal family answer")
    };
    assert!(reader.is_empty());
    assert_eq!(
        answer.version(),
        wire::CLIENT_VERSION,
        "core terminal for setup: {:?}",
        j::Final::decode(engine.client_wire(), &mut Reader::new(answer.record()))
    );
    assert_eq!(
        *answer.owner(),
        j::Owner::Family(j::Family::new(engine.client_wire(), j::FamilyParts { number: wire::FORGE }).unwrap())
    );
    let mut reader = Reader::new(answer.record());
    let answer = w::Answer::decode(&w::CEILINGS, &mut reader).unwrap();
    assert!(reader.is_empty());
    answer.into_parts().outcome
}

#[test]
fn clients_claim_a_listener_adopted_before_the_first_down() {
    use skein_io::kernel::{Fd, Op};
    let mut stand = fixtures::one_process();
    let jig_protocol_config::Address::Addr(addr) = fixtures::engine().clients.listener else { panic!("clients IP") };
    let addr = addr.socket().unwrap();
    let fd = Fd::new(8);
    stand.service.adopt_listener(fd, addr).unwrap();
    assert_eq!(stand.service.adopt_listener(Fd::new(7), addr), Err(Fd::new(7)), "duplicate stays the caller's");
    let mut accepted = false;
    for _ in 0_u16..256 {
        crate::iterate(&mut stand.service, Time::ZERO, Wall::EPOCH);
        for submit in stand.service.submissions().iter() {
            if let Op::Bind { addr: bound, .. } = &submit.kind {
                assert_ne!(*bound, addr, "clients must not bind again");
            }
            if let Op::Listen { fd: listening, .. } = &submit.kind {
                assert_ne!(*listening, fd, "already listening");
            }
            if let Op::Accept { fd: listening, .. } = &submit.kind {
                accepted |= *listening == fd;
            }
        }
        stand.kernel.answer(&mut stand.service);
        if accepted {
            break;
        }
    }
    assert!(accepted, "clients accept on the inherited descriptor");
    assert_eq!(
        stand.service.listening().pop(),
        Some(crate::Note::Engine(crate::EngineNote::Listening { listener: crate::Listening::Clients, addr }))
    );
    stand.service.shutdown();
    for _ in 0_u16..256 {
        stand.turn();
        if stand.service.is_empty() {
            break;
        }
    }
    assert!(stand.service.is_empty());
    let mut closed = 0_u32;
    for descriptor in &stand.kernel.closed {
        if *descriptor == fd {
            closed = closed.checked_add(1).unwrap();
        }
    }
    assert_eq!(closed, 1);
}

#[test]
fn a_blocked_told_head_holds_later_events_for_other_destinations() {
    use skein_io::kernel::{Complete, Done, Fd, Op};
    let mut service = fixtures::ready();
    while service.listening().pop().is_some() {}
    service.adopt_box(Fd::new(3), Fd::new(4), fixtures::system::BOX_BYTES).unwrap();
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let mut read = None;
    for _ in 0..service.submissions().len() {
        let mut submit = service.submissions().pop().unwrap();
        if let Op::PipeRead { fd, buf } = &mut submit.kind
            && *fd == Fd::new(4)
        {
            for (target, source) in buf.iter_mut().zip(b"bad\n") {
                *target = *source;
            }
            read = Some(submit);
        } else {
            service.submissions().push(submit);
        }
    }
    let read = read.expect("box read published to the kernel");
    for _ in 0..service.notes().capacity() {
        service.notes().push(crate::Note::Engine(crate::EngineNote::ForgeLacking { number: 1, names: Box::new([]) }));
    }
    service.completions().push(Complete { op: read.op, kind: read.kind, result: Ok(Done::Count(4)) });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.process.told().len(), 1, "the box event waits at its full diagnostics sink");
    let addr = ([127, 0, 0, 1], 41735).into();
    service.process.told().push(skein_io::Event::Listening {
        owner: Token::new(crate::tokens::SERVER),
        listener: Token::new(77),
        addr: skein_io::Addr::Ip(addr),
    });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.notes().room(), 0);
    assert!(service.listening().is_empty(), "the later clients event cannot pass the box head");
    assert_eq!(service.process.told().len(), 2);
    while service.notes().pop().is_some() {}
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let Some(crate::Note::Engine(crate::EngineNote::BoxLink { .. })) = service.notes().pop() else {
        panic!("the box owner receives its line");
    };
    assert_eq!(
        service.listening().pop(),
        Some(crate::Note::Engine(crate::EngineNote::Listening { listener: crate::Listening::Clients, addr }))
    );
    assert!(service.process.told().is_empty());
}

#[test]
fn an_io_continuation_reaches_its_owner_before_a_later_completion() {
    use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
    use skein_lib::Queue;
    let mut service = fixtures::ready();
    service.adopt_box(Fd::new(3), Fd::new(4), fixtures::system::BOX_BYTES).unwrap();
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let mut read = None;
    for _ in 0..service.submissions().len() {
        let mut submit = service.submissions().pop().unwrap();
        if let Op::PipeRead { fd, buf } = &mut submit.kind
            && *fd == Fd::new(4)
        {
            for (target, source) in buf.iter_mut().zip(b"bad\n") {
                *target = *source;
            }
            read = Some(submit);
        } else {
            service.submissions().push(submit);
        }
    }
    let read = read.expect("box read");
    while service.notes().pop().is_some() {}
    let capacity = service.completions().capacity();
    let store = service.engine_component().store();
    *store.submissions() = Queue::with_capacity(capacity);
    for index in 0..capacity {
        store
            .submissions()
            .push(Submit { op: Token::new(u64::from(index) + 1000), kind: Op::Close { fd: Fd::new(91) } });
    }
    while service.submissions().room() > 0 {
        service.submissions().push(Submit { op: Token::new(9000), kind: Op::Close { fd: Fd::new(92) } });
    }
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.pending(Time::ZERO).raw_submissions, capacity);
    service.adopt_signals(Fd::new(95)).unwrap();
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.pending(Time::ZERO).io_ready, "the signal source has a ready continuation");
    service.completions().push(Complete { op: read.op, kind: read.kind, result: Ok(Done::Count(4)) });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.completions().len(), 1);
    // Publication at the end of this iteration returns exactly one raw slot.
    let _submission = service.submissions().pop().unwrap();
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.pending(Time::ZERO).raw_submissions, capacity - 1);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(!service.pending(Time::ZERO).io_ready, "the continuation spends the returned credit first");
    assert_eq!(service.completions().len(), 1, "the later completion cannot spend that continuation's credit");
    assert!(service.notes().is_empty(), "the box owner has not received the later line");
    // Return route credit for the synthetic full batch, then publish the
    // continuation before letting the service take the later completion.
    reap_staged_closes(&mut service);
    service.publish_staged();
    reap_staged_closes(&mut service);
    service.publish_staged();
    let mut signal_reads = 0_u32;
    for submit in service.submissions().iter() {
        if let Op::ReadSignal { fd } = submit.kind
            && fd == Fd::new(95)
        {
            signal_reads += 1;
        }
    }
    assert_eq!(signal_reads, 1, "the signal source reaches the kernel before the box completion is routed");
    assert!(service.notes().is_empty());
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let Some(crate::Note::Engine(crate::EngineNote::BoxLink { .. })) = service.notes().pop() else {
        panic!("the later completion reaches the box owner");
    };
    assert!(service.completions().is_empty());
}

fn reap_staged_closes(service: &mut crate::Service) {
    use skein_io::kernel::{Complete, Done, Fd, Op};
    for _ in 0..service.submissions().len() {
        let submit = service.submissions().pop().unwrap();
        if let Op::Close { fd } = submit.kind
            && fd == Fd::new(91)
        {
            let _complete =
                service.complete_staged(Complete { op: submit.op, kind: submit.kind, result: Ok(Done::Nothing) });
        }
    }
}
