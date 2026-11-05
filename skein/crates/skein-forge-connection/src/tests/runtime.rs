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

#[test]
fn removal_discards_queued_notifications_and_listener_hints() {
    let limits = f::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut component = crate::Component::new(empty(), &limits, &f::shared()).expect("fixture");
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    let mut added = forge(f::settings());
    added.access = None;
    component.down(&env, Request::Add(added), &mut up, &mut io);
    up.push(Event::Connected { forge: f::FORGE });
    up.push(Event::Connected { forge: f::OTHER_FORGE });
    component.down(&env, Request::Remove { forge: f::FORGE }, &mut up, &mut io);
    let Some(Event::Connected { forge }) = up.pop() else { panic!("other forge's notification expected") };
    assert_eq!(forge, f::OTHER_FORGE);
    assert!(up.is_empty());

    let mut hook = crate::hooks::connection::Hooked::new(Token::new(100), &env);
    hook.refused(api::Hint {
        repository: f::REPO,
        change: api::Change::Branch(Box::from(b"main".as_slice())),
        key: None,
    });
    hook.offer();
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    hook.pump(Token::new(101), &env, &List::with_capacity(0), &empty().word, &mut up, &mut io);
    assert!(up.is_empty(), "listener drops a removed forge's hint when popped");
}
