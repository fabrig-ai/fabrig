//! Two open tabs leave a fresh client its room (temper/docs/design/protocol/configuration.md, 7).
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::{app::Temper, fixtures, front::Joint};

#[test]
#[should_panic(expected = "agent ready within world time")]
fn two_open_tabs_leave_a_fresh_tools_client_room_for_its_read() {
    let (mut world, chat, _, _) = two_open_tabs();
    let agent = fixtures::agent(&mut world);
    let chat = chat.to_string();
    let number = fixtures::give(&mut world, agent, &[b"task", b"show", chat.as_bytes()]);
    let printed = fixtures::answered(&mut world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
}

#[test]
#[should_panic(expected = "reloaded chat whole within world time")]
fn two_open_tabs_leave_room_for_a_persons_reload() {
    let (mut world, chat, _, long) = two_open_tabs();
    world.reload(long);
    world.front().resume_watch_room_page(long);
    let start = world.now();
    let deadline = start.saturating_add(fixtures::WITHIN);
    while world.now() < deadline && !chat_whole(&world, long, chat) {
        world.run(Until::Steps(2));
    }
    assert!(world.now() <= deadline && chat_whole(&world, long, chat), "reloaded chat whole within world time");
    see_live(&mut world, long);
}

#[test]
#[should_panic(expected = "third chat whole within world time")]
fn two_open_tabs_leave_room_for_a_third_page() {
    let (mut world, chat, _, _) = two_open_tabs();
    let third = fixtures::watch_room_client(&mut world, fixtures::chat_page(chat));
    let start = world.now();
    let deadline = start.saturating_add(fixtures::WITHIN);
    while world.now() < deadline && !chat_whole(&world, third, chat) {
        world.run(Until::Steps(2));
    }
    assert!(world.now() <= deadline && chat_whole(&world, third, chat), "third chat whole within world time");
    see_live(&mut world, third);
}

fn chat_whole(world: &World<Temper, Joint>, client: ClientId, task: u64) -> bool {
    let root = world.root(client).person();
    let Some(chat) = root.pages().chat() else { return false };
    let Some(document) = chat.task(root.client()) else { return false };
    root.client().link().state == jig_client::Link::Live
        && document.chip.task == task
        && root.client().document(chat.document).is_some_and(|value| value.state == jig_client::State::Live)
        && !document.delegates.is_empty()
        && chat.delegations.len() == document.delegates.len()
        && chat.delegations.iter().all(|delegate| {
            root.client().document(delegate.handle).is_some_and(|value| value.state == jig_client::State::Live)
        })
        && chat.run.is_some_and(|handle| {
            root.client().document(handle).is_some_and(|value| value.state == jig_client::State::Live)
        })
}

fn two_open_tabs() -> (World<Temper, Joint>, u64, ClientId, ClientId) {
    let mut world = fixtures::watch_room_world(fixtures::SEED);
    let chat = fixtures::watch_room_chat(world.front());
    let home = fixtures::watch_room_client(&mut world, fixtures::project());
    see_live(&mut world, home);
    assert_eq!(world.root(home).person().pages().address(), &fixtures::project());
    let long = fixtures::watch_room_client(&mut world, fixtures::chat_page(chat));
    see_live(&mut world, long);
    assert_eq!(world.root(long).person().pages().address(), &fixtures::chat_page(chat));
    let held = world.front().open_watches();
    let room = usize::try_from(temper_engine_domain::VIEWS.watchers).expect("watch room fits usize");
    assert!(held >= room, "the two tabs hold the old room: {held}");
    (world, chat, home, long)
}

fn see_live(world: &mut World<Temper, Joint>, client: ClientId) {
    world
        .person(client, vec![Step::See { find: Find::exactly_named(Role::Status, b"Live"), within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "tab live within world time: {outcome:?}");
}
