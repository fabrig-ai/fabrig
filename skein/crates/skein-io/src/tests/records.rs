//! Event routing and terminals (skein/docs/design/io.md, 2 and 3.1).

use super::addr;
use crate::kernel::{Exit, ServiceSignal};
use crate::{Error, Event, Measured, Window};
use skein_lib::{Token, stream};

fn owned(owner: Token) -> [Event; 12] {
    let entity = Token::new(99);
    [
        Event::Listening { owner, listener: entity, addr: addr(100) },
        Event::Accepted { owner, socket: entity, peer: addr(101) },
        Event::Connecting { owner, socket: entity },
        Event::Connected { owner },
        Event::Stream { owner, up: stream::Up::Bytes(Box::from(&b"payload"[..])) },
        Event::Output {
            owner,
            up: stream::OutputUp::Settled { right: entity, outcome: stream::OutputOutcome::Granted },
        },
        Event::Spawned { owner, child: entity, pipes: Box::new([Token::new(10), Token::new(11)]) },
        Event::Exited { owner, exit: Exit::Code(17) },
        Event::Usage { owner, usage: Measured::Read(crate::kernel::Usage::ZERO) },
        Event::Window { owner, window: Window::Gone },
        Event::Failed { owner, error: Error::Refused },
        Event::Closed { owner },
    ]
}

#[test]
fn routing_preserves_every_owned_payload_and_identifies_the_terminal() {
    let original = Token::new(1);
    let destination = Token::new(2);
    for (event, expected) in owned(original).into_iter().zip(owned(destination)) {
        assert_eq!(event.owner(), Some(original));
        assert_eq!(event.is_closed(), event == Event::Closed { owner: original });
        let remapped = event.with_owner(destination);
        assert_eq!(remapped.owner(), Some(destination));
        assert_eq!(remapped, expected);
    }
}

fn notifications() -> [Event; 3] {
    [
        Event::Resized,
        Event::Shutdown { signal: ServiceSignal::Interrupt, from: None },
        Event::Shutdown { signal: ServiceSignal::Terminate, from: Some(Token::new(9)) },
    ]
}

#[test]
fn service_notifications_remain_ownerless_and_keep_their_sender() {
    for (notification, expected) in notifications().into_iter().zip(notifications()) {
        assert_eq!(notification.owner(), None);
        assert!(!notification.is_closed());
        let remapped = notification.with_owner(Token::new(2));
        assert_eq!(remapped.owner(), None);
        assert_eq!(remapped, expected);
    }
}

#[test]
fn the_test_preset_is_usable_and_allows_entrance_overrides() {
    assert!(crate::valid(&crate::Limits::TEST));
    let changed = crate::Limits { sockets: 2, intake: 8, ..crate::Limits::TEST };
    assert_eq!(changed.largest_read(), 8);
    assert!(crate::valid(&changed));
}
