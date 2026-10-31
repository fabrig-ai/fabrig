//! A person's project board (jig/docs/design/client/domain/testing.md, 4–6).
use super::{
    b_conversations::{WAIT, finish, sign_in},
    b_seed::Facts,
};
use crate::{ClientId, Front, Test, World, harness::Seed};
use jig_client_test_domain::Address;
use jig_client_view::address::{Address as Page, ProjectSection};
use skein_fake_person::{Find, Step};
use skein_ui::Role;

fn ready<F: Front<Test>>(world: &mut World<Test, F>, section: ProjectSection) -> ClientId {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Project(Facts::Goals));
    let client = world.client(1, Address::Jig(Page::Project { project: 1, section }));
    world.person(client, sign_in(vec![Step::See { find: Find::text(b"Second goal"), within: WAIT }]));
    finish(world);
    client
}
fn goal_order<F: Front<Test>>(world: &World<Test, F>, client: ClientId) -> Vec<u64> {
    let mut tasks: Vec<_> = world
        .tree(client)
        .nodes()
        .iter()
        .filter_map(|node| node.key.filter(|key| node.role == Role::Item && key.kind == 0).map(|key| key.a))
        .collect();
    tasks.dedup();
    tasks
}
pub fn moving_a_goal_up_reorders_the_board<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world, ProjectSection::Goals);
    assert_eq!(goal_order(world, client), vec![1, 2]);
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Move up") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
        ],
    );
    finish(world);
    assert_eq!(goal_order(world, client), vec![2, 1]);
}
pub fn the_project_overview_shows_the_board_s_first_goals<F: Front<Test>>(world: &mut World<Test, F>) {
    let client = ready(world, ProjectSection::Overview);
    world.person(client, vec![Step::See { find: Find::text(b"First goal"), within: WAIT }]);
    finish(world);
    assert_eq!(goal_order(world, client), vec![1, 2]);
}
pub fn the_frame_shows_no_conversations_entry_without_a_chat_template<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Project(Facts::NoChatTemplate));
    let client = world.client(1, crate::stories::inbox());
    world.person(client, sign_in(vec![Step::See { find: Find::text(b"Signed in as"), within: WAIT }]));
    finish(world);
    assert!(
        !world
            .tree(client)
            .nodes()
            .iter()
            .any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Conversations"))
    );
}
fn propose<F: Front<Test>>(world: &mut World<Test, F>, procedure: bool) {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::Project(Facts::GoalOffer { procedure }));
    let client = world.client(1, Address::Jig(Page::Project { project: 1, section: ProjectSection::Goals }));
    let executor = if procedure { b"Connector 2, procedure 7".as_slice() } else { b"Charter 1".as_slice() };
    world.person(
        client,
        sign_in(vec![
            Step::See { find: Find::named(Role::TextArea, b"Goal specification"), within: WAIT },
            Step::Type {
                find: Find::named(Role::TextArea, b"Goal specification"),
                words: b"Resolve incident".as_slice().into(),
            },
            Step::Press { find: Find::named(Role::Button, executor) },
            Step::Type { find: Find::named(Role::TextField, b"Goal budget"), words: b"2".as_slice().into() },
            Step::See { find: Find::text(b"Beyond your pool, this will be proposed"), within: WAIT },
            Step::Press { find: Find::named(Role::Button, b"Set goal") },
            Step::See { find: Find::text(executor), within: WAIT },
            Step::See { find: Find::text(b"proposed"), within: WAIT },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Proposed"), within: WAIT },
        ]),
    );
    finish(world);
}
pub fn a_goal_beyond_the_allotment_is_proposed_as_the_confirmation_said<F: Front<Test>>(world: &mut World<Test, F>) {
    propose(world, false);
}
pub fn a_procedure_only_goal_form_confirms_the_supplied_executor<F: Front<Test>>(world: &mut World<Test, F>) {
    propose(world, true);
}
