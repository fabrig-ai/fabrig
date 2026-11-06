//! People use project roles (jig/docs/design/client/domain/testing.md, 4–6).
use super::a_person::{finish, sign_in_if_needed};
use crate::{
    ClientId, Front, Test, World,
    harness::{Seed, seed::RoleStory},
};
use jig_client_test_domain::Address;
use jig_client_view::address::{Address as JigAddress, ProjectSection, TaskSection};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
const WITHIN: Duration = Duration::from_secs(1);

/// Receiving bounds for role documents and their two reports.
#[must_use]
pub fn limits() -> jig_client_test_domain::Limits {
    let mut limits = crate::test::limits();
    limits.client.domain.text = 128;
    limits.client.domain.long_text = 128;
    limits.client.domain.turn_text = 128;
    limits.client.domain.turn_bytes = 128;
    limits.jig.roles = 4;
    limits.first.text = 128;
    limits
}
fn client(seed: u64) -> (jig_client_test_domain::Domain, jig_client_test_view::View<'static>) {
    crate::test::new(&limits(), seed)
}
fn enter<F: Front<Test>>(world: &mut World<Test, F>, party: u64, address: Address) -> ClientId {
    let client = world.client_with(party, address, limits(), client);
    sign_in_if_needed(world, client);
    client
}
fn ready<F: Front<Test>>(world: &mut World<Test, F>, party: u64) -> ClientId {
    let client = enter(world, party, crate::stories::inbox());
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"decide this goal"), within: WITHIN },
            Step::See { find: Find::exactly_named(Role::Button, b"Accept"), within: WITHIN },
        ],
    );
    finish(world);
    client
}
fn decide<F: Front<Test>>(world: &mut World<Test, F>, client: ClientId, by: u64) {
    let by = format!("Decided by person: {by}").into_bytes();
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Accepted"), within: WITHIN },
            Step::See { find: Find::text(&by), within: WITHIN },
        ],
    );
}

/// A maintainer accepts a goal beyond its proposing member's budget.
pub fn a_maintainer_accepts_a_members_proposal<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::Roles { story: RoleStory::Proposal });
    let maintainer = ready(world, 2);
    world.person(maintainer, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Accept") }]);
    finish(world);
    decide(world, maintainer, 2);
    finish(world);
}

/// Two independently authenticated parties see the same winning decision.
pub fn two_parties_decide_one_proposal<F: Front<Test>>(world: &mut World<Test, F>) {
    let (owner, maintainer) = prepare_two_parties(world);
    confirm_two_parties(world, owner, maintainer);
}
/// Shows the pending proposal to its two authenticated holders.
pub fn prepare_two_parties<F: Front<Test>>(world: &mut World<Test, F>) -> (ClientId, ClientId) {
    world.seed(Seed::Roles { story: RoleStory::Proposal });
    let owner = ready(world, 1);
    let maintainer = ready(world, 2);
    (owner, maintainer)
}
/// Opens both confirmations and submits them before either answer is shown.
pub fn confirm_two_parties<F: Front<Test>>(world: &mut World<Test, F>, owner: ClientId, maintainer: ClientId) {
    for party in [owner, maintainer] {
        world.person(party, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Accept") }]);
    }
    finish(world);
    for party in [owner, maintainer] {
        world.person(
            party,
            vec![
                Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
                Step::See { find: Find::text(b"Accepted"), within: WITHIN },
                Step::See { find: Find::text(b"Decided by person: "), within: WITHIN },
            ],
        );
    }
    finish(world);
    let [owner_by, maintainer_by] = [owner, maintainer].map(|party| {
        [b"Decided by person: 1", b"Decided by person: 2"].map(|by| {
            world
                .tree(party)
                .nodes()
                .iter()
                .flat_map(|node| &node.text)
                .any(|run| run.text.windows(by.len()).any(|part| part == by))
        })
    });
    let [first, second] = owner_by;
    assert_ne!(first, second, "exactly one party won");
    assert_eq!(owner_by, maintainer_by, "both parties see the same winner");
}

/// The member's open conversation changes when its owner accepts in the inbox.
pub fn an_owners_decision_updates_the_members_chat<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::Roles { story: RoleStory::Proposal });
    let member = enter(world, 3, Address::Jig(JigAddress::Task { task: 1, section: TaskSection::Activity }));
    world.person(
        member,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Read earlier"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Read earlier") },
            Step::See { find: Find::text(b"decide this goal"), within: WITHIN },
        ],
    );
    finish(world);
    let owner = ready(world, 1);
    world.person(owner, vec![Step::Press { find: Find::exactly_named(Role::Button, b"Accept") }]);
    finish(world);
    decide(world, owner, 1);
    world.person(member, vec![Step::See { find: Find::text(b"Accepted"), within: WITHIN }]);
    finish(world);
}

/// A maintainer takes a role's person task and answers its choice.
pub fn answering_a_choice_between_reports<F: Front<Test>>(world: &mut World<Test, F>) {
    let maintainer = prepare_choice(world);
    complete_choice(world, maintainer);
}
/// Shows the unclaimed role task before the person takes it.
pub fn prepare_choice<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    world.seed(Seed::Roles { story: RoleStory::Choice });
    let maintainer = enter(world, 2, crate::stories::inbox());
    world.person(
        maintainer,
        vec![
            Step::See { find: Find::text(b"first report or second report"), within: WITHIN },
            Step::See { find: Find::exactly_named(Role::Button, b"Take it"), within: WITHIN },
        ],
    );
    finish(world);
    maintainer
}
/// Takes and answers the task prepared by `prepare_choice`.
pub fn complete_choice<F: Front<Test>>(world: &mut World<Test, F>, maintainer: ClientId) {
    world.person(
        maintainer,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Take it") },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::exactly_named(Role::Button, b"Answer"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Answer") },
            Step::See { find: Find::exactly_named(Role::Choice, b"Choice code: 2"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Choice, b"Choice code: 2") },
            Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"second report".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"answered"), within: WITHIN },
            Step::Gone { find: Find::exactly_named(Role::Button, b"Answer"), within: WITHIN },
        ],
    );
    finish(world);
}

/// A member sees that a goal beyond its allotment will be proposed before confirming.
pub fn a_goal_beyond_a_members_allotment_is_proposed<F: Front<Test>>(world: &mut World<Test, F>) {
    let member = prepare_goal(world);
    confirm_goal(world, member);
}
/// Opens the proposal confirmation before a request is sent.
pub fn prepare_goal<F: Front<Test>>(world: &mut World<Test, F>) -> ClientId {
    world.seed(Seed::Roles { story: RoleStory::Goal });
    let member = enter(world, 3, Address::Jig(JigAddress::Project { project: 1, section: ProjectSection::Goals }));
    world.person(
        member,
        vec![
            Step::See { find: Find::exactly_named(Role::TextArea, b"Goal specification"), within: WITHIN },
            Step::Type {
                find: Find::exactly_named(Role::TextArea, b"Goal specification"),
                words: b"a proposed goal".as_slice().into(),
            },
            Step::Press { find: Find::exactly_named(Role::Button, b"Charter 1") },
            Step::Type { find: Find::exactly_named(Role::TextField, b"Goal budget"), words: b"150".as_slice().into() },
            Step::See { find: Find::text(b"This will be proposed to its holder"), within: WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Set goal") },
            Step::See { find: Find::text(b"This will be proposed"), within: WITHIN },
        ],
    );
    finish(world);
    member
}
/// Confirms the disclosed proposal and shows its answer.
pub fn confirm_goal<F: Front<Test>>(world: &mut World<Test, F>, member: ClientId) {
    world.person(
        member,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Proposed"), within: WITHIN },
        ],
    );
    finish(world);
}

/// An observer's unavailable creation leaves their request in the editor.
pub fn an_observer_sees_why_creation_is_unavailable<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::Roles { story: RoleStory::Observer });
    let observer = enter(world, 4, crate::stories::conversation());
    let mut steps = crate::stories::start_chat_steps();
    steps.insert(0, Step::See { find: Find::named(Role::TextArea, b"First words"), within: WITHIN });
    steps.push(Step::See { find: Find::text(b"This act is not offered"), within: WITHIN });
    world.person(observer, steps);
    finish(world);
    assert!(world.tree(observer).nodes().iter().any(|node| node.role == Role::TextArea
        && node.name.as_deref() == Some(b"First words")
        && node.value.as_ref().is_some_and(|value| value.text.as_ref() == b"Hello")));
}

/// The decision's record names the person who asked, through a delegate.
pub fn a_delegate_s_escalation_record_names_the_person_who_asked<F: Front<Test>>(world: &mut World<Test, F>) {
    const DELEGATE: u64 = 2;
    world.seed(Seed::Roles { story: RoleStory::DelegateHold });
    let client = enter(world, 1, crate::stories::inbox());
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Held delegate"), within: WITHIN },
            Step::See { find: Find::exactly_named(Role::Button, b"Leave it held"), within: WITHIN },
        ],
    );
    finish(world);
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Leave it held") },
            Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"Leave held".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Escalation decided"), within: WITHIN },
        ],
    );
    finish(world);
    let closer = enter(world, 1, Address::Jig(JigAddress::task(DELEGATE)));
    super::b_proposals::open(world, closer, Find::exactly_named(Role::Button, b"Cancel"));
    finish(world);
    world.person(
        closer,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Cancel") },
            Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"Done".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Cancelled"), within: WITHIN },
        ],
    );
    finish(world);
    // The application names the record so the page reads the ended escalation afresh.
    world.event(
        client,
        jig_client_test_domain::Event::Jig(jig_client_view::Event::Named {
            objects: crate::test::list([jig_client_domain::contract::ObjectKey::Escalation(DELEGATE)]),
            sections: crate::test::list([]),
        }),
    );
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Ended escalation"), within: WITHIN },
            Step::See { find: Find::text(b"Requester task: 3"), within: WITHIN },
            Step::See { find: Find::text(b"Decision person: 1"), within: WITHIN },
        ],
    );
    finish(world);
}
