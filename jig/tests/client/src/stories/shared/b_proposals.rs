//! People decide a proposal from their inbox (jig/docs/design/client/domain/testing.md, 4–6).
use super::{
    b_conversations::{WAIT, finish, sign_in},
    b_seed::Facts,
};
use crate::{ClientId, Front, Test, World, harness::Seed};
use skein_fake_person::{Find, Step};
use skein_ui::Role;

fn ready<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Project(Facts::ProposedGoal));
    let client = world.client(1, crate::stories::inbox());
    world.person(client, sign_in(vec![Step::See { find: Find::named(Role::Button, b"Accept"), within: WAIT }]));
    finish(world);
    client
}
pub fn a_proposal_accepted_in_the_inbox_leaves_its_chat_too<F: Front<Test>>(world: &mut World<Test, F>) {
    let inbox = ready(world);
    let chat = world.client(
        1,
        jig_client_test_domain::Address::Jig(jig_client_view::address::Address::Task {
            task: 1,
            section: jig_client_view::address::TaskSection::Activity,
        }),
    );
    open(world, chat, Find::named(Role::Button, b"Accept"));
    finish(world);
    world.person(
        inbox,
        vec![
            Step::Press { find: Find::named(Role::Button, b"Accept") },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Accepted"), within: WAIT },
        ],
    );
    world.person(chat, vec![Step::See { find: Find::text(b"Accepted"), within: WAIT }]);
    finish(world);
}
pub fn two_tabs_deciding_one_thing_are_told_who_decided_first<F: Front<Test>>(world: &mut World<Test, F>) {
    let first = ready(world);
    let second = world.tab(first);
    open(world, second, Find::named(Role::Button, b"Accept"));
    finish(world);
    let accept = |client| {
        world
            .tree(client)
            .nodes()
            .iter()
            .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Accept"))
            .expect("delivered proposal")
            .id
    };
    let first_node = accept(first);
    let second_node = accept(second);
    world.fault(crate::Fault::TwoTabs { first, first_node, second, second_node });
    for client in [first, second] {
        world.person(
            client,
            vec![
                Step::Press { find: Find::named(Role::Button, b"Confirm") },
                Step::See { find: Find::text(b"Accepted"), within: WAIT },
            ],
        );
    }
    finish(world);
    assert!([first, second].into_iter().any(|client| {
        world
            .tree(client)
            .nodes()
            .iter()
            .flat_map(|node| &node.text)
            .any(|run| run.text.windows(b"Decided by person: 1".len()).any(|part| part == b"Decided by person: 1"))
    }));
}
pub fn closing_a_chat_cancels_one_delegate_and_moves_the_other_to_the_person<F: Front<Test>>(
    world: &mut World<Test, F>,
) {
    use jig_client_test_domain::Address;
    use jig_client_view::address::{Address as Page, TaskSection};
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Project(Facts::Delegates));
    let client = world.client(1, Address::Jig(Page::Task { task: 1, section: TaskSection::Activity }));
    world.person(
        client,
        sign_in(vec![
            Step::Pause { span: skein_lib::Duration::from_millis(100) },
            Step::See { find: Find::named(Role::Button, b"Close"), within: WAIT },
            Step::Press { find: Find::named(Role::Button, b"Close") },
            Step::Press { find: Find::named(Role::Button, b"Cancel delegate 2 and its subtree") },
            Step::Press { find: Find::named(Role::Button, b"Take over delegate 3, reserving 1") },
            Step::Type { find: Find::named(Role::TextArea, b"Reason"), words: b"finished".as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::Go { address: Address::Jig(Page::Task { task: 2, section: TaskSection::Overview }) },
            Step::See { find: Find::text(b"Cancelled delegate"), within: WAIT },
            Step::See { find: Find::text(b"Cancelled with reason"), within: WAIT },
            Step::See { find: Find::text(b"finished"), within: WAIT },
            Step::Go { address: Address::Jig(Page::Task { task: 3, section: TaskSection::Overview }) },
            Step::See { find: Find::text(b"Moved delegate"), within: WAIT },
            Step::See { find: Find::text(b"Requested by person 1"), within: WAIT },
        ]),
    );
    finish(world);
}

fn open<F: Front<Test>>(world: &mut World<Test, F>, client: ClientId, find: Find) {
    world.person(client, vec![Step::Pause { span: skein_lib::Duration::from_millis(20) }]);
    finish(world);
    let steps = vec![Step::See { find, within: WAIT }];
    let steps = if world
        .tree(client)
        .nodes()
        .iter()
        .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Sign in with Test"))
    {
        sign_in(steps)
    } else {
        steps
    };
    world.person(client, steps);
}
