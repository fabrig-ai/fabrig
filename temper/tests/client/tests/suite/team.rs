//! Two people read Team through the client and engine roots
//! (temper/docs/design/client/pages/README.md, Team).
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures, front::Joint};

type Clients = World<Temper, Joint>;

const PROJECT: u32 = 1;
// The owner's identity, fixture sign-in and client sign-in occupy the first three identifiers.
const MEMBER: u64 = 4;
const MEMBER_SUBJECT: u64 = 8;
const QUESTION: &[u8] = b"Export the table as CSV?";
const ANSWER: &[u8] = b"Go ahead.";

fn settle(world: &mut Clients) {
    let until = world.now().saturating_add(fixtures::WITHIN);
    let outcome = world.run(Until::Time(until));
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn people(world: &Clients, who: ClientId, expected: &[[&[u8]; 3]]) {
    let tree = world.tree(who);
    assert!(
        matches!(TreeFace::new(tree).find(&Find::exactly_named(Role::Link, b"Team")), Found::Node(_)),
        "{who:?}: Team in the rail"
    );
    let Found::Node(main) = TreeFace::new(tree).find(&Find::named(Role::Main, b"")) else {
        panic!("Team's main column missing");
    };
    let mut face = TreeFace::subtree(tree, main).expect("main column");
    for (role, name) in [
        (Role::Heading { level: 1 }, b"Who owns what on Project 1".as_slice()),
        (Role::Paragraph, b"Two of you.".as_slice()),
        (Role::Heading { level: 2 }, b"People".as_slice()),
    ] {
        assert!(matches!(face.find(&Find::exactly_named(role, name)), Found::Node(_)), "missing {name:?}");
    }
    let start = tree.nodes().iter().position(|node| node.id == main).expect("main column's position");
    let size = usize::try_from(tree.nodes()[start].size).expect("main column's size");
    let nodes = &tree.nodes()[start..start + size];
    let cards: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter(|(index, node)| {
            node.role == Role::Slot && nodes.get(index + 1).is_some_and(|child| child.role == Role::Text)
        })
        .map(|(_, node)| node.id)
        .collect();
    assert_eq!(cards.len(), expected.len(), "{who:?}: one box for each person");
    for (card, words) in cards.into_iter().zip(expected) {
        let mut face = TreeFace::subtree(tree, card).expect("person's box");
        for (role, name) in [Role::Text, Role::Paragraph, Role::Paragraph].into_iter().zip(words) {
            assert!(
                matches!(face.find(&Find::exactly_named(role, name)), Found::Node(_)),
                "{who:?}: missing {role:?} {name:?} in {words:?}, reader first"
            );
        }
    }
}

#[test]
fn two_people_read_each_other_on_team() {
    let mut world = fixtures::world(fixtures::SEED);
    world.randomness(0, 0, 0);
    fixtures::team_question(world.front(), QUESTION);
    let owner = world.client(fixtures::PARTY, Address::Team { project: PROJECT, role: None });
    settle(&mut world);
    // Establish membership through the owner before the member opens its watches.
    let member = ClientId(1);
    world.front().bind_dismissal_person(member, MEMBER, MEMBER_SUBJECT);
    world.front().add_dismissal_member(owner, MEMBER, PROJECT);
    assert_eq!(world.client(MEMBER, Address::Team { project: PROJECT, role: None }), member);
    settle(&mut world);
    people(
        &world,
        owner,
        &[[b"You", b"Owner", b"One question waits on you."], [b"Other", b"Member", b"Nothing waits on Other."]],
    );
    people(
        &world,
        member,
        &[[b"You", b"Member", b"Nothing waits on you."], [b"Person", b"Owner", b"One question waits on Person."]],
    );
    world.person(
        owner,
        vec![
            Step::Go { address: fixtures::inbox() },
            Step::See { find: Find::text(QUESTION), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Answer words"), words: ANSWER.into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Answer") },
            Step::See { find: Find::exactly_named(Role::Status, b"Answered"), within: fixtures::WITHIN },
            Step::Go { address: Address::Team { project: PROJECT, role: None } },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    settle(&mut world);
    people(
        &world,
        owner,
        &[[b"You", b"Owner", b"Nothing waits on you."], [b"Other", b"Member", b"Nothing waits on Other."]],
    );
    people(
        &world,
        member,
        &[[b"You", b"Member", b"Nothing waits on you."], [b"Person", b"Owner", b"Nothing waits on Person."]],
    );
}

#[test]
fn a_person_reads_pauses_and_resumes_a_standing_role_on_team() {
    use jig_client_kit::harness::Settings;
    use jig_domain_tasks as tasks;
    let (front, task) = super::person_task_tools::parked_role();
    let mut world: Clients =
        World::new(Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() }, front);
    world.randomness(0, 0, 0);
    let owner = world.client(fixtures::PARTY, Address::Team { project: PROJECT, role: Some(task) });
    settle(&mut world);
    let facts = temper_client_domain::pages::team::shown(world.root(owner).person(), PROJECT);
    let temper_client_domain::pages::team::Shown::Read(facts) = facts else { panic!("Team read") };
    assert_eq!(facts.people.len(), 1, "the requester is alone in the project");
    assert_eq!(facts.roles.len(), 1);
    assert_eq!(facts.roles.get(0).unwrap().task, task);
    assert_eq!(
        facts.card.as_ref().unwrap().now.as_ref().unwrap().body.as_ref(),
        super::person_task_tools::ROLE_NOW.as_bytes()
    );
    world.person(
        owner,
        vec![
            Step::See { find: Find::exactly_named(Role::Link, b"Team"), within: fixtures::WITHIN },
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 2 }, b"Standing roles"),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 2 }, b"Owns the tests"),
                within: fixtures::WITHIN,
            },
            Step::See { find: Find::text(b"Waiting for a change, then check its tests."), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Pause") },
            Step::See { find: Find::exactly_named(Role::Button, b"Resume"), within: fixtures::WITHIN },
            Step::See {
                find: Find::text(b"Paused: running tasks stop, nothing new starts."),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "Pause must send Stop: {outcome:?}");
    assert!(matches!(world.front().engine.task(task).unwrap().phase, tasks::Phase::Held { .. }));
    world.person(
        owner,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Resume") },
            Step::See { find: Find::exactly_named(Role::Button, b"Pause"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Resumed."), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "Resume must release the hold: {outcome:?}");
    assert!(!matches!(world.front().engine.task(task).unwrap().phase, tasks::Phase::Held { .. }));
}
