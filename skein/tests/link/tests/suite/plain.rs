use skein_io::Request;
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Queue, bytes};
use skein_link::{Event, Link, Moved, Start};
use skein_link_world::{SOCKET, env};

#[test]
fn a_link_shows_its_queues_lengths_and_no_byte() {
    let mut link = Link::new(None);
    link.down(&env(), Down::Send(bytes::copy_of(b"secret-token")));
    link.up(&env(), Up::Bytes(bytes::copy_of(b"secret-token")));
    assert_eq!(format!("{link:?}"), "Link { socket: None, encrypted: false, events: 1, cipher: 1, ending: Open }");
}

#[test]
fn without_tls_a_close_is_at_once() {
    for (abort, expected) in [(false, Request::Close { entity: SOCKET }), (true, Request::Abort { entity: SOCKET })] {
        let mut link = Link::new(None);
        let mut below = Queue::uncapped();
        link.connecting(SOCKET, &mut below);
        link.down(&env(), Down::Send(bytes::copy_of(b"queued")));
        link.up(&env(), Up::End);
        link.close(&env(), abort, &mut below);
        assert_eq!(below.pop(), Some(expected));
        assert_eq!(link.connected(&env()), Start::Closing);
        assert_eq!(link.pump(&mut below), Moved::Nothing);
        assert_eq!(link.next_deadline(), None);
        link.close(&env(), false, &mut below);
        assert!(below.is_empty());
    }
}

#[test]
fn without_tls_the_stream_passes_through() {
    let mut link = Link::new(None);
    let mut below = Queue::uncapped();
    link.connecting(SOCKET, &mut below);
    assert!(!link.encrypted());
    assert_eq!(link.connected(&env()), Start::Ready);
    for (down, moved) in [
        (Down::Demand { read: Read::Nothing, room: 1 }, Moved::Within),
        (Down::Send(bytes::copy_of(b"x")), Moved::Sent),
        (Down::Send(Box::new([])), Moved::Within),
        (Down::Finish, Moved::Within),
    ] {
        // Down has owned bytes: the expected request keeps a copy in the test.
        let expected = match &down {
            Down::Demand { read, room } => Down::Demand { read: *read, room: *room },
            Down::Send(bytes) => Down::Send(bytes.clone()),
            Down::Finish => Down::Finish,
        };
        link.down(&env(), down);
        assert!(link.has_work(), "queued output is work");
        assert_eq!(link.pump(&mut below), moved);
        assert_eq!(below.pop(), Some(Request::Stream { stream: SOCKET, down: expected }));
        assert!(!link.has_work());
    }
    for up in [Up::Room, Up::Bytes(bytes::copy_of(b"reply")), Up::End] {
        let expected = match &up {
            Up::Room => Up::Room,
            Up::Bytes(bytes) => Up::Bytes(bytes.clone()),
            Up::End => Up::End,
            Up::Failed(fault) => Up::Failed(*fault),
        };
        link.up(&env(), up);
        assert!(link.has_work(), "queued input is work");
        assert_eq!(link.pump(&mut below), Moved::Above(Event::Stream(expected)));
        assert!(!link.has_work());
    }
    assert_eq!(link.pump(&mut below), Moved::Nothing);
}
