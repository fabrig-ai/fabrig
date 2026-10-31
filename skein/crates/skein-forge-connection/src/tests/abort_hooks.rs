//! Abort escalates a webhook's graceful HTTP close (programming-model.md, 5.2).
use super::*;
use crate::tests::fixtures;

#[test]
fn abort_escalates_a_webhook_already_closing_after_its_response() {
    let limits = fixtures::limits();
    let env = Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut hooks = Hooks::new(&limits);
    let socket = Token::new(100);
    let owner = Token::new(OWNER.raw().checked_add(1).expect("hook owner"));
    let mut connection = Hooked::new(socket, &env);
    connection.closing = Closing::Sent;
    let id = hooks.slab.insert(connection).expect("hook room");
    let previous = hooks.connections.insert(owner, id).expect("index room");
    assert!(previous.is_none());
    let mut io = Queue::uncapped();
    let mut up = Queue::uncapped();
    hooks.abort();
    hooks.drive(&env, &fixtures::hook_settings(), &mut up, &mut io);
    let Some(skein_io::Request::Abort { entity }) = io.pop() else { unreachable!("graceful close escalated") };
    assert_eq!(entity, socket);
    hooks.drive(&env, &fixtures::hook_settings(), &mut up, &mut io);
    assert!(io.is_empty(), "one abort per hook");
    assert!(up.is_empty());
    assert!(!hooks.closed(), "waits for the socket's Closed");
    hooks.up(&env, skein_io::Event::Closed { owner }, &mut up);
    assert!(hooks.closed());
}
