//! The sender's admission, terminal and deadline rules (attention.md, 7.2–7.3).
#![expect(clippy::disallowed_macros, clippy::disallowed_methods, reason = "ordinary tests drive bounded push streams")]
use super::*;
const PEM: &[u8] = include_bytes!("../../../../../tests/one-process/src/fixtures/vapid.pem");
fn runner() -> Runner {
    let mut runner = Runner::new();
    runner.configure(Some(crate::settings::Push {
        key: skein_push::Vapid::from_pkcs8_pem(PEM).unwrap(),
        subject: b"mailto:operator@example.test".as_slice().into(),
        services: Box::new([Box::from(b"http://127.0.0.1:31999".as_slice())]),
    }));
    runner
}
fn push() -> root::Push {
    let key = skein_lib::base64::decode(
        skein_lib::base64::Alphabet::Url,
        b"BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4",
        65,
    )
    .unwrap();
    root::Push {
        party: 7,
        device: 1,
        endpoint: b"http://127.0.0.1:31999/push/alice".as_slice().into(),
        p256dh: key.as_ref().try_into().unwrap(),
        auth: [1; 16],
        topic: b"p7".as_slice().into(),
        payload: b"count".as_slice().into(),
    }
}
#[test]
fn departed_origins_and_the_seventeenth_push_are_dropped_without_a_queue() {
    let mut runner = runner();
    let mut io = Queue::uncapped();
    let mut departed = push();
    departed.endpoint = b"http://127.0.0.1:32000/push/alice".as_slice().into();
    runner.send(departed, Time::ZERO, Wall::EPOCH, &mut io);
    assert!(runner.closed() && io.is_empty());
    for _ in 0_u32..17 {
        runner.send(push(), Time::ZERO, Wall::EPOCH, &mut io);
    }
    assert_eq!(runner.sending.len(), 16);
    assert_eq!(io.len(), 16);
    let mut events = Queue::uncapped();
    runner.advance(Time::ZERO, Wall::EPOCH, false, &mut events, &mut io);
    assert_eq!(io.len(), 16, "overflow was not queued");
}
fn answer(events: &mut Queue<root::Event>) -> PushAnswer {
    let Some(root::Event::Forge(event)) = events.pop() else { panic!("Pushed") };
    let forge::Event::Pushed { party, device, endpoint, answer } = *event else { panic!("Pushed") };
    assert_eq!((party, device, endpoint), (7, 1, push().endpoint));
    answer
}
#[test]
fn each_status_is_told_once_and_never_retried() {
    for (status, expected) in [
        (201_u16, PushAnswer::Made),
        (404, PushAnswer::Gone),
        (410, PushAnswer::Gone),
        (413, PushAnswer::Failed),
        (429, PushAnswer::Failed),
        (500, PushAnswer::Failed),
        (403, PushAnswer::Failed),
        (302, PushAnswer::Failed),
    ] {
        let mut runner = runner();
        let mut io = Queue::uncapped();
        let mut events = Queue::uncapped();
        runner.send(push(), Time::ZERO, Wall::EPOCH, &mut io);
        let Some(Request::Connect { owner, .. }) = io.pop() else { panic!("connect") };
        let socket = Token::new(99);
        runner.up(Event::Connecting { owner, socket }, Time::ZERO, Wall::EPOCH, &mut io);
        runner.up(Event::Connected { owner }, Time::ZERO, Wall::EPOCH, &mut io);
        let mut wire = skein_world::stream::Wire::new(8192);
        let mut replied = false;
        for _ in 0_u32..128 {
            runner.advance(Time::ZERO, Wall::EPOCH, false, &mut events, &mut io);
            while let Some(request) = io.pop() {
                match request {
                    Request::Stream { down, .. } => wire.take(down),
                    Request::Abort { entity } => {
                        assert_eq!(entity, socket);
                        runner.up(Event::Closed { owner }, Time::ZERO, Wall::EPOCH, &mut io);
                    }
                    other @ (Request::Listen { .. }
                    | Request::Connect { .. }
                    | Request::Bind { .. }
                    | Request::Reject { .. }
                    | Request::Output { .. }
                    | Request::Spawn { .. }
                    | Request::SpawnAlone { .. }
                    | Request::Signal { .. }
                    | Request::Usage { .. }
                    | Request::Window { .. }
                    | Request::Close { .. }) => panic!("no second connection: {other:?}"),
                }
            }
            if !replied && wire.received.windows(4).any(|part| part == b"\r\n\r\n") {
                // The response header alone settles a push; an unbounded body is not read.
                wire.write(
                    format!("HTTP/1.1 {status} Answer\r\nRetry-After: 3\r\nContent-Length: 999999\r\n\r\n").as_bytes(),
                );
                replied = true;
            }
            if let Some(up) = wire.answer() {
                runner.up(Event::Stream { owner, up }, Time::ZERO, Wall::EPOCH, &mut io);
            }
        }
        assert_eq!(answer(&mut events), expected);
        assert!(events.is_empty() && runner.closed() && io.is_empty());
        let written = &wire.received;
        assert!(written.windows(b"POST /push/alice HTTP/1.1".len()).any(|part| part == b"POST /push/alice HTTP/1.1"));
    }
}
#[test]
fn ten_seconds_abandons_an_exchange_and_late_events_cannot_send_it_again() {
    let mut runner = runner();
    let mut io = Queue::uncapped();
    let mut events = Queue::uncapped();
    let start = Time::from_nanos(7);
    runner.send(push(), start, Wall::EPOCH, &mut io);
    let Some(Request::Connect { owner, .. }) = io.pop() else { panic!("connect") };
    let socket = Token::new(99);
    runner.up(Event::Connecting { owner, socket }, start, Wall::EPOCH, &mut io);
    let due = start.saturating_add(Duration::from_secs(10));
    assert_eq!(runner.next_deadline(), Some(due));
    runner.advance(Time::from_nanos(due.as_nanos() - 1), Wall::EPOCH, false, &mut events, &mut io);
    assert!(events.is_empty());
    runner.advance(due, Wall::EPOCH, false, &mut events, &mut io);
    assert_eq!(answer(&mut events), PushAnswer::Failed);
    let Some(Request::Abort { entity }) = io.pop() else { panic!("abort timed-out socket") };
    assert_eq!(entity, socket);
    runner.up(Event::Connected { owner }, due, Wall::EPOCH, &mut io);
    runner.up(Event::Closed { owner }, due, Wall::EPOCH, &mut io);
    runner.advance(due, Wall::EPOCH, false, &mut events, &mut io);
    assert!(runner.closed() && events.is_empty() && io.is_empty());
}
