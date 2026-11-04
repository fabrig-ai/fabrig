#![expect(clippy::disallowed_macros, reason = "canaries check formatted values and bytes")]
use crate::{Event, Request, tests::fixtures};
use skein_channel::{Lower, LowerEvent};
use skein_lib::{Queue, Time, Token, stream};

fn hidden(debug: &str, canary: &str) {
    assert!(!debug.contains(canary) && !debug.contains(&format!("{:?}", canary.as_bytes())), "{debug}");
}
#[test]
fn a_queued_write_shows_no_bytes() {
    let canary = "canary-workers-grant";
    let connection = Token::new(1);
    let write = || Lower::Write(stream::OutputDown::Send { right: connection, bytes: Box::from(canary.as_bytes()) });
    let request = Request::Lower { connection, lower: write() };
    let event = Event::Lower { connection, event: LowerEvent::Read(stream::Up::Bytes(Box::from(canary.as_bytes()))) };
    let mut component = fixtures::component();
    component.down(Time::ZERO, Event::Accepted { connection }, &mut Queue::uncapped());
    component.connections.get_mut(&connection).expect("accepted connection").below.push(write());
    for debug in [format!("{request:?}"), format!("{event:?}"), format!("{component:?}")] {
        hidden(&debug, canary);
    }
}
