//! Words and held conversations through the public tree (jig/docs/design/client/domain/testing.md, 4–6).
use super::b_conversations::{CHAT, WAIT, finish, ready, start_chat};
use crate::{ClientId, Front, FrontFault, Test, World};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;

fn held<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    use crate::harness::Seed;
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Project(super::b_seed::Facts::HeldConversation));
    let client = world.client(
        1,
        jig_client_test_domain::Address::Jig(jig_client_view::address::Address::Task {
            task: 1,
            section: jig_client_view::address::TaskSection::Activity,
        }),
    );
    world.person(
        client,
        super::b_conversations::sign_in(vec![Step::See {
            find: Find::exactly_named(Role::Button, b"Release"),
            within: WAIT,
        }]),
    );
    finish(world);
    client
}
pub fn a_held_chat_is_released<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = held(world);
    release(world, client);
}
fn release<F: Front<Test>>(world: &mut World<Test, F>, client: ClientId) {
    world.person(
        client,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Release"), within: WAIT },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release") },
            Step::See { find: Find::text(b"Being decided"), within: WAIT },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Released"), within: WAIT },
            Step::Gone { find: Find::exactly_named(Role::Button, b"Release"), within: WAIT },
        ],
    );
    finish(world);
}
pub fn a_held_chat_is_released_at_the_text_shell<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = held(world);
    world.at_text_shell(client);
    release(world, client);
}
pub fn a_person_s_words_are_shown_as_sent<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    send_words(world, client, false);
}
fn send_words<F: Front<Test>>(world: &mut World<Test, F>, client: ClientId, release: bool) {
    let mut steps = words_steps(b"Please continue", release);
    steps.push(Step::See { find: Find::text(b"Sent"), within: WAIT });
    if release {
        steps.push(Step::Gone { find: Find::exactly_named(Role::Button, b"Release"), within: WAIT });
    }
    world.person(client, steps);
    finish(world);
}
#[must_use]
pub fn words_steps(words: &[u8], release: bool) -> Vec<Step<jig_client_test_domain::Address>> {
    let mut steps = vec![
        Step::See { find: Find::exactly_named(Role::Button, b"Write"), within: WAIT },
        Step::Press { find: Find::exactly_named(Role::Button, b"Write") },
        Step::Type { find: Find::named(Role::TextArea, b"Your answer"), words: words.into() },
    ];
    if release {
        steps.push(Step::Press { find: Find::named(Role::Button, b"Write and release together") });
    }
    steps.extend([
        Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
        Step::See { find: Find::text(words), within: WAIT },
    ]);
    steps
}
pub fn words_to_a_stopped_task_release_it_together<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = held(world);
    send_words(world, client, true);
}
pub fn the_engine_restarting_under_a_pending_request_answers_it_once<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world);
    world.fault(FrontFault::NoAnswer);
    start_chat(world, client, b"Second chat");
    world.fault(FrontFault::Restart { offline: Duration::from_millis(120) });
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
            .filter(|node| node.key.is_some_and(|key| key.kind == 20 && key.a != CHAT))
            .count(),
        1
    );
}
