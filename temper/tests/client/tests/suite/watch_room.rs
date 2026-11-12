//! Two open tabs leave a fresh client its room (temper/docs/design/protocol/configuration/limits.md).
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::{app::Temper, fixtures, front::Joint};

#[test]
fn two_open_tabs_leave_a_fresh_tools_client_room_for_its_read() {
    let (mut world, chat, _, _) = two_open_tabs();
    let agent = fixtures::agent(&mut world);
    let chat = chat.to_string();
    let number = fixtures::give(&mut world, agent, &[b"task", b"show", chat.as_bytes()]);
    let printed = fixtures::answered(&mut world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
}

#[test]
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

#[test]
fn a_client_past_the_room_is_told_and_answered_when_a_page_closes() {
    let mut world = fixtures::watch_room_world_streams(fixtures::SEED, 1);
    let (chat, _, long) = open_tabs(&mut world);
    let agent = fixtures::agent(&mut world);
    let deadline = world.now().saturating_add(fixtures::WITHIN);
    world.run(Until::Time(deadline));
    assert!(!world.root(agent).agent().ready(), "the full room keeps the tools client out");
    assert!(world.root(agent).agent().watches_full(), "the client knows the room is full");
    let held = world.front().open_watches();
    world.front().close_client(long);
    assert!(world.front().open_watches() < held, "closing the tab gives its watches back");
    let backoff = temper_client_world::agent::limits().domain.client.domain.backoff.most;
    let deadline = world.now().saturating_add(backoff).saturating_add(fixtures::WITHIN);
    while world.now() < deadline && !world.root(agent).agent().ready() {
        world.run(Until::Steps(2));
    }
    assert!(world.now() <= deadline && world.root(agent).agent().ready(), "ready after the tab closes");
    assert!(!world.root(agent).agent().watches_full());
    let chat = chat.to_string();
    let number = fixtures::give(&mut world, agent, &[b"task", b"show", chat.as_bytes()]);
    let printed = fixtures::answered(&mut world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
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
    let (chat, home, long) = open_tabs(&mut world);
    (world, chat, home, long)
}

fn open_tabs(world: &mut World<Temper, Joint>) -> (u64, ClientId, ClientId) {
    let chat = fixtures::watch_room_chat(world.front());
    let home = fixtures::watch_room_client(world, fixtures::project());
    see_live(world, home);
    assert_eq!(world.root(home).person().pages().address(), &fixtures::project());
    let long = fixtures::watch_room_client(world, fixtures::chat_page(chat));
    see_live(world, long);
    assert_eq!(world.root(long).person().pages().address(), &fixtures::chat_page(chat));
    // The task window bounds the delegated batch. A second read of the live root
    // from Home occupies the remaining place without growing that batch.
    world.event(
        home,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Watch {
                handle: jig_client::Handle {
                    asker: temper_client_domain::owner::Owner::Pages.asker(),
                    number: 4_294_967_296,
                },
                subject: jig_client::Watchable::Jig(jig_client::contract::Subject::Task { task: chat }),
            },
        }
        .into(),
    );
    see_live(world, home);
    let held = world.front().open_watches();
    let room = usize::try_from(temper_engine_domain::VIEWS.watchers).expect("watch room fits usize");
    assert!(held >= room, "the two tabs hold the old room: {held}");
    (chat, home, long)
}

fn see_live(world: &mut World<Temper, Joint>, client: ClientId) {
    world
        .person(client, vec![Step::See { find: Find::exactly_named(Role::Status, b"Live"), within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "tab live within world time: {outcome:?}");
}
