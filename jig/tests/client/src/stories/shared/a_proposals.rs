//! One proposal is shown across inboxes and conversations (jig/docs/design/client/domain/testing.md, 4–6).
use super::a_person::{finish, sign_in, sign_in_if_needed};
use crate::{ClientId, Fault, Front, Test, World, harness::Seed};
use jig_client_test_domain::Address as RootAddress;
use jig_client_view::address::Address;
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_lib::Duration;
use skein_ui::Role;
const WITHIN: Duration = Duration::from_secs(1);

fn pending<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    world.seed(Seed::ChatProject { party: 1, project: 1 });
    world.seed(Seed::PendingGoal { words: b"decide this goal".as_slice().into(), budget: 50 });
    let owner = world.client(1, crate::stories::inbox());
    let mut steps = sign_in();
    steps.extend([
        Step::See { find: Find::text(b"decide this goal"), within: WITHIN },
        Step::See { find: Find::exactly_named(Role::Button, b"Accept"), within: WITHIN },
    ]);
    world.person(owner, steps);
    finish(world);
    owner
}

/// Accepting a proposal names the decision and its maker.
pub fn accepting_a_proposed_goal<F: Front<Test>>(world: &mut World<Test, F>) {
    let owner = pending(world);
    world.person(
        owner,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Accept") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Accepted"), within: WITHIN },
            Step::See { find: Find::text(b"Decided by person: 1"), within: WITHIN },
        ],
    );
    finish(world);
}

/// Two restored tabs see the same first decision when both confirmations send.
pub fn two_tabs_decide_one_proposal<F: Front<Test>>(world: &mut World<Test, F>) {
    let owner = pending(world);
    let other = world.tab(owner);
    sign_in_if_needed(world, other);
    world.person(other, vec![Step::See { find: Find::text(b"decide this goal"), within: WITHIN }]);
    finish(world);
    for tab in [owner, other] {
        world.person(tab, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Accept") }]);
    }
    finish(world);
    let confirm = |tab| match TreeFace::new(world.tree(tab)).find(&Find::exactly_named(Role::Button, b"Confirm")) {
        Found::Node(node) => node,
        Found::Absent | Found::Pending => panic!("both confirmations are ready"),
    };
    let first_node = confirm(owner);
    let second_node = confirm(other);
    world.fault(Fault::TwoTabs { first: owner, first_node, second: other, second_node });
    for tab in [owner, other] {
        world.person(
            tab,
            vec![
                Step::See { find: Find::text(b"Accepted"), within: WITHIN },
                Step::See { find: Find::text(b"Decided by person: 1"), within: WITHIN },
            ],
        );
    }
    finish(world);
}

/// A decision in the inbox also updates the open conversation's proposal.
pub fn an_inbox_decision_updates_its_conversation<F: Front<Test>>(world: &mut World<Test, F>) {
    let owner = pending(world);
    let chat = world.client(
        1,
        RootAddress::Jig(Address::Task { task: 1, section: jig_client_view::address::TaskSection::Activity }),
    );
    sign_in_if_needed(world, chat);
    let steps = vec![
        Step::See { find: Find::exactly_named(Role::Button, b"Read earlier"), within: WITHIN },
        Step::Press { find: Find::exactly_named(Role::Button, b"Read earlier") },
        Step::See { find: Find::text(b"decide this goal"), within: WITHIN },
        Step::See { find: Find::text(b"Proposal"), within: WITHIN },
    ];
    world.person(chat, steps);
    finish(world);
    world.person(
        owner,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Accept") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Accepted"), within: WITHIN },
        ],
    );
    world.person(
        chat,
        vec![
            Step::See { find: Find::text(b"decide this goal"), within: WITHIN },
            Step::See { find: Find::text(b"Accepted"), within: WITHIN },
        ],
    );
    finish(world);
}
