//! Runtime membership (skein/docs/design/forge-connection.md, 3.1).
use super::fixtures as f;
use crate::{Event, Request, Settings, settings::Forge};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{List, Queue, Token};

fn empty() -> Settings {
    let mut settings = f::settings();
    settings.forges = List::with_capacity(0);
    settings
}

fn forge(settings: Settings) -> Forge {
    settings.forges.into_boxed().into_iter().next().expect("fixture forge")
}

fn harness() -> f::Harness {
    let mut h = f::Harness::new(empty());
    h.endpoints = List::with_capacity(2);
    h.endpoints.push((forge(f::settings()).address, f::FORGE)).expect("fixture endpoint");
    h.git_tokens = Some(List::with_capacity(32));
    h.installation_events = Some(List::with_capacity(32));
    h.facts = Some(List::with_capacity(32));
    h
}

#[test]
fn removal_discards_queued_notifications_and_listener_hints() {
    let mut h = harness();
    let mut added = forge(f::settings());
    added.access = None;
    h.send(Request::Add(added));
    h.up.push(Event::Connected { forge: f::FORGE });
    h.up.push(Event::Connected { forge: f::OTHER_FORGE });
    h.send(Request::Remove { forge: f::FORGE });
    let Some(Event::Connected { forge }) = h.up.pop() else { panic!("other forge's notification expected") };
    assert_eq!(forge, f::OTHER_FORGE);
    assert!(h.up.is_empty());

    let mut hook = crate::hooks::connection::Hooked::new(Token::new(100), &h.env);
    hook.refused(api::Hint {
        repository: f::REPO,
        change: api::Change::Branch(Box::from(b"main".as_slice())),
        key: None,
    });
    hook.offer();
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    hook.pump(Token::new(101), &h.env, &List::with_capacity(0), &empty().word, &mut up, &mut io);
    assert!(up.is_empty(), "listener drops a removed forge's hint when popped");
}
