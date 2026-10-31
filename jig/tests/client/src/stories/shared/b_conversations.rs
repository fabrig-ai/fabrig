//! Conversation actions and visible results (jig/docs/design/client/domain/testing.md, 4–6).
use crate::{ClientId, Front, FrontFault, Test, Until, World, harness::Seed};
use jig_client_test_domain::Address;
use jig_client_view::address::{Address as Page, TaskSection};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
use skein_world::domain::Verdict;

pub const WAIT: Duration = Duration::from_secs(2);
pub const PARTY: u64 = 1;
pub const CHAT: u64 = 1;

pub fn finish<F: Front<Test>>(world: &mut World<Test, F>) {
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(outcome.judged.0 > 0);
}
pub fn ready<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    world.seed(Seed::ChatProject { party: PARTY, project: 1 });
    world.seed(Seed::Conversation { party: PARTY, project: 1, words: b"Hello".as_slice().into() });
    let client = world.client(PARTY, Address::Jig(Page::Task { task: CHAT, section: TaskSection::Activity }));
    world.person(client, sign_in(vec![Step::See { find: Find::text(b"Hello"), within: WAIT }]));
    finish(world);
    client
}
#[must_use]
pub fn sign_in(mut steps: Vec<Step<Address>>) -> Vec<Step<Address>> {
    let mut before = vec![
        Step::See { find: Find::named(Role::Button, b"Sign in with Test"), within: WAIT },
        Step::Press { find: Find::named(Role::Button, b"Sign in with Test") },
    ];
    before.append(&mut steps);
    before
}
pub fn a_draft_is_kept_across_a_reload<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    world.person(
        client,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Write"), within: WAIT },
            Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
            Step::Type { find: Find::named(Role::TextArea, b"Your answer"), words: b"Unsent draft".as_slice().into() },
            Step::Pause { span: Duration::from_millis(30) },
            Step::Reload,
            Step::See { find: Find::exactly_named(Role::Button, b"Write"), within: WAIT },
            Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
        ],
    );
    finish(world);
    assert!(
        world
            .tree(client)
            .nodes()
            .iter()
            .any(|node| node.value.as_ref().is_some_and(|value| value.text.as_ref() == b"Unsent draft"))
    );
}
pub fn start_chat<F: Front<Test>>(world: &mut World<Test, F>, client: ClientId, words: &[u8]) {
    world.person(client, start_chat_steps(words));
    finish(world);
}
#[must_use]
pub fn start_chat_steps(words: &[u8]) -> Vec<Step<Address>> {
    vec![
        Step::Go { address: crate::stories::conversation() },
        Step::See { find: Find::named(Role::TextArea, b"First words"), within: WAIT },
        Step::Type { find: Find::named(Role::TextArea, b"First words"), words: words.into() },
        Step::Press { find: Find::named(Role::Button, b"Start a conversation") },
        Step::See { find: Find::text(words), within: WAIT },
    ]
}
pub fn a_reload_with_requests_pending_sends_them_again_under_their_keys<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    world.fault(FrontFault::NoAnswer);
    start_chat(world, client, b"Second chat");
    world.reload(client);
    world.person(client, vec![Step::See { find: Find::text(b"Second chat"), within: WAIT }]);
    finish(world);
    world.go(client, crate::stories::conversation());
    world.person(client, vec![Step::See { find: Find::text(b"Second chat"), within: WAIT }]);
    finish(world);
    assert_eq!(
        world
            .tree(client)
            .nodes()
            .iter()
            .filter(|node| node.key.is_some_and(|key| key.kind == 20 && key.a == 2))
            .count(),
        1
    );
}
pub fn a_dropped_watch_shows_the_page_behind_until_it_reopens<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    world.fault(FrontFault::DropWatch);
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Behind"), within: WAIT },
            Step::Gone { find: Find::text(b"Behind"), within: WAIT },
            Step::See { find: Find::text(b"Hello"), within: WAIT },
        ],
    );
    finish(world);
}
pub fn a_page_left_with_reads_in_flight_drops_their_answers<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    world.fault(FrontFault::Slow { span: Duration::from_millis(50) });
    world.go(client, crate::stories::inbox());
    world.go(client, crate::stories::conversation());
    world.person(client, vec![Step::See { find: Find::named(Role::TextArea, b"First words"), within: WAIT }]);
    finish(world);
    assert!(
        !world
            .tree(client)
            .nodes()
            .iter()
            .any(|node| node.role == Role::Log && node.name.as_deref() == Some(b"Conversation"))
    );
}
pub fn the_conversations_section_lists_live_chats_by_last_activity<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    start_chat(world, client, b"Second chat");
    world.go(client, crate::stories::conversation());
    world.person(
        client,
        vec![
            Step::See { find: Find::named(Role::Link, b"Second chat"), within: WAIT },
            Step::See { find: Find::named(Role::Link, b"Hello"), within: WAIT },
        ],
    );
    finish(world);
    let tasks: Vec<_> = world
        .tree(client)
        .nodes()
        .iter()
        .filter_map(|node| node.key.filter(|key| key.kind == 20).map(|key| key.a))
        .collect();
    assert_eq!(tasks, vec![2, CHAT]);
}
pub fn a_person_s_chats_list_live_then_ended_by_last_activity<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    start_chat(world, client, b"Second chat");
    world.person(
        client,
        vec![
            Step::Go { address: Address::Jig(Page::Task { task: CHAT, section: TaskSection::Activity }) },
            Step::See { find: Find::text(b"Hello"), within: WAIT },
            Step::Press { find: Find::exactly_named(Role::Button, b"Cancel") },
            Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"Done chatting".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Ended"), within: WAIT },
            Step::Go { address: crate::stories::conversation() },
            Step::See { find: Find::text(b"Second chat"), within: WAIT },
            Step::Press { find: Find::named(Role::Button, b"Read earlier") },
            Step::See { find: Find::text(b"Hello"), within: WAIT },
        ],
    );
    finish(world);
}
pub fn an_engine_outside_the_client_s_versions_shows_out_of_date<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    world.fault(FrontFault::Versions);
    world.person(client, vec![Step::See { find: Find::text(b"Out of date"), within: WAIT }]);
    finish(world);
}
pub fn starting_a_chat_at_the_text_shell<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::ChatProject { party: PARTY, project: 1 });
    let client = world.client(PARTY, crate::stories::conversation());
    world.at_text_shell(client);
    world.person(client, sign_in(vec![Step::See { find: Find::named(Role::TextArea, b"First words"), within: WAIT }]));
    finish(world);
    start_chat(world, client, b"Hello");
}
