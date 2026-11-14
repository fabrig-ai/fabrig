use crate::world::World;
use skein_browser::boundary::{Event, Refusal, Request};
use skein_browser::wire::decode::Document;
use skein_lib::Token;
const PAGE: Token = Token::new(3);
const READ: Token = Token::new(4);
#[test]
fn tabs_read_the_browser_and_end_once_before_page_close() {
    let mut world = World::new();
    world.start();
    world.open();
    world.sent.clear();
    world.ask(Request::Tabs { page: PAGE, op: READ });
    let Some(Event::Tabs { op: READ, tabs, more: 0 }) = world.above.pop() else { panic!("tab list") };
    assert!(tabs.is_empty());
    let mut referee = crate::referee::Referee::new();
    referee.event(&Event::Tabs { op: READ, tabs, more: 0 }, 0).expect("browser contract");
    assert_eq!(world.sent.len(), 1);
    let document = Document::parse(world.sent[0].strip_suffix(&[0]).unwrap(), world.env.limits.command).unwrap();
    assert_eq!(document.root().get(b"method").unwrap().text().unwrap().as_ref(), b"Target.getTargets");
    assert!(document.root().get(b"sessionId").is_none());
    world.ask(Request::Close { entity: PAGE });
    world.above.clear();
    world.sent.clear();
    world.ask(Request::Tabs { page: PAGE, op: Token::new(5) });
    assert!(matches!(world.above.pop(), Some(Event::Refused { op, why: Refusal::Gone, .. }) if op == Token::new(5)));
    assert!(world.sent.is_empty(), "closed page sends no command");
    world.check_commands_settled().expect("commands settled");
}
