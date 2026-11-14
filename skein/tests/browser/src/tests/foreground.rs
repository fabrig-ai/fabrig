//! Looking brings the page to the front (skein/docs/design/testing/browser.md, 6).
use crate::world::{Fault, World};
use skein_browser::boundary::{Event, Expect, Match, Query, Refusal, Request};
use skein_browser::wire::decode::Document;
use skein_lib::{Duration, Token};

const PAGE: Token = Token::new(3);
const LOOK: Token = Token::new(4);

fn query(within: Option<u64>) -> Query {
    Query {
        role: b"button".as_slice().into(),
        name: b"Save".as_slice().into(),
        matching: Match::Exact,
        within,
        boxes: false,
    }
}
fn world() -> World {
    let mut world = World::new();
    world.start();
    world.open();
    world.sent.clear();
    world
}
fn front_command(world: &World) {
    let bytes = world.sent.first().expect("front command").strip_suffix(&[0]).expect("CDP delimiter");
    let command = Document::parse(bytes, world.env.limits.command).expect("encoded command");
    let root = command.root();
    assert_eq!(root.get(b"method").expect("method").text().expect("method text").as_ref(), b"Page.bringToFront");
    assert_eq!(root.get(b"sessionId").expect("session").text().expect("session text").as_ref(), b"s1");
    assert_eq!(root.get(b"params").expect("parameters").raw(), b"{}");
}

#[test]
fn a_look_waits_for_its_own_page_to_come_to_the_front() {
    for (request, next) in [
        (Request::Find { page: PAGE, op: LOOK, query: query(None) }, b"Runtime.evaluate".as_slice()),
        (Request::Find { page: PAGE, op: LOOK, query: query(Some(7)) }, b"DOM.resolveNode"),
        (Request::Snapshot { page: PAGE, op: LOOK }, b"Accessibility.getFullAXTree"),
        (Request::Screenshot { page: PAGE, op: LOOK }, b"Page.captureScreenshot"),
    ] {
        let mut world = world();
        world.fault(b"Page.bringToFront", Fault::Delay);
        world.ask(request);
        assert_eq!(world.sent.len(), 1, "no read before bringing page to front: {next:?}");
        front_command(&world);
        assert!(world.above.is_empty(), "no terminal before foregrounding");
        world.release_delayed();
        let bytes = world.sent.get(1).expect("read after foregrounding").strip_suffix(&[0]).expect("CDP delimiter");
        let command = Document::parse(bytes, world.env.limits.command).expect("encoded command");
        assert_eq!(command.root().get(b"method").expect("method").text().expect("method text").as_ref(), next);
        world.check_commands_settled().expect("commands settled");
    }
}

#[test]
fn an_await_foregrounds_again_only_at_its_poll_and_keeps_its_deadline() {
    let mut world = world();
    world.ax_visible = false;
    let poll = world.env.limits.poll;
    let within = poll.saturating_add(poll);
    world.ask(Request::Await { page: PAGE, op: LOOK, query: query(None), expect: Expect::Present, within });
    front_command(&world);
    world.sent.clear();
    let before_poll = Duration::from_nanos(poll.as_nanos().checked_sub(1).expect("positive poll"));
    world.tick(world.env.now.saturating_add(before_poll));
    assert!(world.sent.is_empty(), "no front command before the poll");
    world.tick(world.env.now.saturating_add(Duration::from_nanos(1)));
    front_command(&world);
    world.sent.clear();
    world.tick(world.env.now.saturating_add(poll));
    assert!(matches!(world.above.pop(), Some(Event::Missed { op: LOOK, .. })));
    world.sent.clear();
    world.tick(world.env.now.saturating_add(poll));
    assert!(world.sent.is_empty(), "no foregrounding after the await ended");
    world.check_commands_settled().expect("commands settled");
}

#[test]
fn a_refused_foreground_command_ends_the_look_without_reading() {
    let mut world = world();
    world.fault(b"Page.bringToFront", Fault::Error);
    world.ask(Request::Screenshot { page: PAGE, op: LOOK });
    front_command(&world);
    assert_eq!(world.sent.len(), 1, "no capture after foregrounding failed");
    assert!(matches!(world.above.pop(), Some(Event::Refused { op: LOOK, why: Refusal::Gone, .. })));
    assert!(world.above.is_empty(), "one terminal");
    world.check_commands_settled().expect("commands settled");
}
