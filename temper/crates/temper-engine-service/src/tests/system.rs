use super::fixtures;
use crate::End;
use skein_lib::{Time, Token, Wall};








#[test]
fn a_socket_workers_bind_announces_its_bound_address_once() {
    let mut service = fixtures::started();
    for _ in 0..service.notes().capacity() {
        service.notes().push(crate::Note::ForgeLacking { number: 1, names: Box::new([]) });
    }
    assert_eq!(service.notes().room(), 0, "forge diagnostics occupy their whole queue");
    let addr = ([127, 0, 0, 1], 41733).into();
    service.process.told().push(skein_io::Event::Listening {
        owner: Token::new(crate::tokens::WORKER_LISTENERS),
        listener: Token::new(77),
        addr: skein_io::Addr::Ip(addr),
    });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.listening().pop(), Some(crate::Note::Listening { listener: crate::Listening::Workers, addr }));
    assert!(service.listening().is_empty());
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.listening().is_empty());
    assert_eq!(service.notes().pop(), Some(crate::Note::ForgeLacking { number: 1, names: Box::new([]) }));
}

#[test]
fn an_in_process_workers_bind_has_no_listening_note() {
    let mut service = fixtures::started();
    service.process.told().push(skein_io::Event::Listening {
        owner: Token::new(crate::tokens::WORKER_LISTENERS),
        listener: Token::new(77),
        addr: skein_io::Addr::InProcess(skein_io::Name::new(b"workers").expect("configured worker listener")),
    });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.listening().is_empty());
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.listening().is_empty());
}

#[test]
fn a_clients_bind_announces_the_servers_bound_address_once() {
    let mut service = fixtures::started();
    let addr = ([127, 0, 0, 1], 41734).into();
    service.process.told().push(skein_io::Event::Listening {
        owner: Token::new(crate::tokens::SERVER),
        listener: Token::new(77),
        addr: skein_io::Addr::Ip(addr),
    });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.listening().pop(), Some(crate::Note::Listening { listener: crate::Listening::Clients, addr }));
    assert!(service.listening().is_empty());
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert!(service.listening().is_empty());
}


#[test]
fn a_new_client_after_idle_gets_its_whole_stall_period() {
    use skein_lib::Duration;
    const STALL: Duration = Duration::from_secs(30);
    const BEGAN: Time = Time::from_nanos(1_000_000_000);
    for idle in [29, 30, 60, 100] {
        let mut service = fixtures::started();
        fixtures::listening(&mut service);
        crate::iterate(&mut service, BEGAN, Wall::EPOCH);
        let accepted = BEGAN.saturating_add(Duration::from_secs(idle));
        service.process.told().push(skein_io::Event::Accepted {
            owner: Token::new(crate::tokens::SERVER),
            socket: fixtures::http::SOCKET,
            peer: fixtures::http::address(),
        });
        crate::iterate(&mut service, accepted, Wall::EPOCH);
        assert_eq!(
            service.engine.as_ref().unwrap().clients.next_deadline(),
            Some(accepted.saturating_add(STALL)),
            "new connection after {idle} seconds idle gets its deadline at accept"
        );
    }
}

#[test]
fn a_new_webhook_after_idle_gets_its_whole_delivery_period() {
    use skein_lib::Duration;
    const BEGAN: Time = Time::from_nanos(1_000_000_000);
    const IDLE: Duration = Duration::from_secs(60);
    const LISTENER: Token = Token::new(1_u64 << 59);
    let mut service = fixtures::ready();
    let config = fixtures::engine();
    let mut found = jig_protocol_config::Problems::empty();
    let settings = crate::forge::settings(
        &fixtures::forges(),
        fixtures::forge_runtime(),
        Some(skein_forge_connection::hooks::Webhooks { listener: fixtures::http::address() }),
        &mut found,
    )
    .unwrap();
    let layer = &mut service.engine.as_mut().unwrap().forge;
    layer.component = skein_forge_connection::Component::new(
        settings,
        &layer.env.limits,
        &crate::forge::shared(&config.limits, &config.jig.limits.io),
    )
    .unwrap();
    layer.component.begin(&layer.env, &mut layer.above, &mut layer.io);
    layer.up(skein_io::Event::Listening { owner: LISTENER, listener: Token::new(88), addr: fixtures::http::address() });
    while layer.io.pop().is_some() {}
    let delivery = layer.env.limits.hook_deadline;
    crate::iterate(&mut service, BEGAN, Wall::EPOCH);
    let accepted = BEGAN.saturating_add(IDLE);
    service.process.told().push(skein_io::Event::Accepted {
        owner: Token::new(crate::tokens::FORGE | LISTENER.raw()),
        socket: Token::new(89),
        peer: fixtures::http::address(),
    });
    crate::iterate(&mut service, accepted, Wall::EPOCH);
    assert_eq!(
        service.engine.as_ref().unwrap().forge.component.next_deadline(),
        Some(accepted.saturating_add(delivery)),
        "the webhook starts its delivery period at accept"
    );
}

#[test]
fn a_source_is_marked_closing_only_once_its_close_is_queued() {
    use skein_io::kernel::{Fd, Op, Submit};
    use skein_lib::Queue;
    let mut service = fixtures::ready();
    let fd = Fd::new(90);
    service.adopt_listener(fd, ([127, 0, 0, 1], 41736).into()).unwrap();
    let capacity = service.completions().capacity();
    // Stage a complete raw batch through the store's existing kernel entrance.
    let store = &mut service.engine.as_mut().unwrap().store;
    store.submitted = Queue::with_capacity(capacity);
    for index in 0..capacity {
        store.submitted.push(Submit { op: Token::new(u64::from(index) + 1000), kind: Op::Close { fd: Fd::new(91) } });
    }
    *service.submissions() = Queue::with_capacity(capacity);
    for index in 0..capacity {
        service
            .submissions()
            .push(Submit { op: Token::new(u64::from(index) + 2000), kind: Op::Close { fd: Fd::new(92) } });
    }
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.pending(Time::ZERO).raw_submissions, capacity, "raw kernel staging is at its cap");
    service.engine.as_mut().unwrap().end = Some(End::Aborted);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.wait_state().listener_closes, 0, "the source stays open without close-output credit");
    while service.submissions().pop().is_some() {}
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.wait_state().listener_closes, 0, "publication returns credit after the close stage");
    reap_staged_closes(&mut service);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    assert_eq!(service.wait_state().listener_closes, 1);
    let mut closes = 0_u32;
    for submit in service.submissions().iter() {
        if let Op::Close { fd: found } = submit.kind
            && found == fd
        {
            closes += 1;
        }
    }
    assert_eq!(closes, 1, "the kernel observes the close when the source is marked closing");
}

#[test]
fn queued_completions_are_taken_before_a_due_io_timer() {
    use skein_io::kernel::{Complete, Error, Fd, Op};
    use skein_lib::Queue;
    let mut service = fixtures::ready();
    service.adopt_signals(Fd::new(95)).unwrap();
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let mut accept = None;
    let mut read = None;
    for _ in 0..service.submissions().len() {
        let submit = service.submissions().pop().unwrap();
        if let Op::Accept { .. } = submit.kind
            && accept.is_none()
        {
            accept = Some(submit);
        } else if let Op::ReadSignal { fd } = submit.kind
            && fd == Fd::new(95)
        {
            read = Some(submit);
        } else {
            service.submissions().push(submit);
        }
    }
    let accept = accept.expect("the clients listener's accept");
    let Op::Accept { fd: listener } = accept.kind else { panic!("accept") };
    service.completions().push(Complete { op: accept.op, kind: accept.kind, result: Err(Error::TooManyOpenFiles) });
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let due = service.next_deadline().expect("the listener retries its exhausted descriptor budget");
    assert!(service.pending(due).io_due, "the retry is an IO timer");
    let read = read.expect("signal read");
    // A one-entry shell completion queue makes this one completion a full reap batch.
    *service.completions() = Queue::with_capacity(1);
    service.completions().push(Complete { op: read.op, kind: read.kind, result: Err(Error::BrokenPipe) });
    assert_eq!(service.completions().room(), 0);
    crate::iterate(&mut service, due, Wall::EPOCH);
    let mut close_at = None;
    let mut retry_at = None;
    for (position, submit) in service.submissions().iter().enumerate() {
        if let Op::Close { fd } = submit.kind
            && fd == Fd::new(95)
        {
            close_at = Some(position);
        }
        if let Op::Accept { fd } = submit.kind
            && fd == listener
        {
            retry_at = Some(position);
        }
    }
    let close_at = close_at.expect("the completion's signal source closes its failed input");
    let retry_at = retry_at.expect("the due listener timer retries accept");
    assert!(close_at < retry_at, "the kernel sees the completion owner's output before the timer owner's output");
    assert!(service.completions().is_empty());
}

// Synthetic closes fill staging without an io owner. Reaping their published
// records returns route credit just as a shell completion does.
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
