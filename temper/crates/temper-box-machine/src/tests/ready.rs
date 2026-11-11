use crate::ready::Ready;
use skein_io::{Event, Request};
use skein_lib::{
    Duration, Queue, Time, Token,
    stream::{Down, Up},
};

#[test]
fn version_checks_retry_refusals_and_bad_status_then_accept_a_split_answer() {
    let mut ready = Ready::new(([127, 0, 0, 1], 3000).into());
    let mut out = Queue::with_capacity(16);
    let mut serial = 10;
    let mut now = Time::ZERO;
    ready.start(now);
    for response in [b"".as_slice(), b"HTTP/1.1 503 Unavailable\r\n", b"HTTP/1.1 200 OK\r\n"] {
        assert_eq!(ready.tick(now, &mut serial, &mut out), None);
        let Some(Request::Connect { owner, .. }) = out.pop() else { panic!("version connection") };
        let socket = Token::new(100);
        ready.event(Event::Connecting { owner, socket }, now, &mut out);
        if !response.is_empty() {
            ready.event(Event::Connected { owner }, now, &mut out);
            let Some(Request::Stream { down: Down::Demand { .. }, .. }) = out.pop() else { panic!("read demand") };
            ready.event(Event::Stream { owner, up: Up::Room }, now, &mut out);
            let Some(Request::Stream { down: Down::Send(request), .. }) = out.pop() else { panic!("HTTP request") };
            assert!(request.starts_with(b"GET /api/v1/version "));
            out.pop();
            for (index, byte) in response.iter().enumerate() {
                ready.event(Event::Stream { owner, up: Up::Bytes(Box::new([*byte])) }, now, &mut out);
                if index + 1 < response.len() {
                    let Some(Request::Stream { down: Down::Demand { .. }, .. }) = out.pop() else {
                        panic!("read demand")
                    };
                }
            }
            assert_eq!(out.pop(), Some(Request::Abort { entity: socket }));
        }
        ready.event(Event::Closed { owner }, now, &mut out);
        now = now.saturating_add(Duration::from_millis(50));
    }
    assert_eq!(ready.tick(now, &mut serial, &mut out), Some(true));
    assert!(ready.settled());
    assert!(out.is_empty());
}

#[test]
fn a_hung_version_check_is_bounded_and_cancellation_drains_before_reuse() {
    for cancel in [false, true] {
        for delayed in [false, true] {
            let mut ready = Ready::new(([127, 0, 0, 1], 3000).into());
            let mut out = Queue::with_capacity(16);
            let mut serial = 0;
            ready.start(Time::ZERO);
            ready.tick(Time::ZERO, &mut serial, &mut out);
            let Some(Request::Connect { owner, .. }) = out.pop() else { panic!("connect") };
            // Cancel before Connecting: its late socket still has to close.
            if cancel {
                ready.cancel(&mut out);
            }
            let socket = Token::new(100);
            let now = Time::ZERO.saturating_add(Duration::from_secs(30));
            if delayed {
                assert_eq!(ready.tick(now, &mut serial, &mut out), None);
                assert!(out.is_empty(), "there is no socket to close yet");
            }
            ready.event(Event::Connecting { owner, socket }, now, &mut out);
            assert_eq!(ready.tick(now, &mut serial, &mut out), None);
            assert_eq!(out.pop(), Some(Request::Abort { entity: socket }));
            assert!(!ready.settled(), "connection must close before the check settles");
            ready.event(Event::Closed { owner }, now, &mut out);
            assert_eq!(ready.tick(now, &mut serial, &mut out), if cancel { None } else { Some(false) });
            assert!(ready.settled());
            assert!(out.is_empty());
            assert_eq!(ready.next_deadline(), None);
        }
    }
}
