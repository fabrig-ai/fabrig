#[path = "../stories/goals.rs"]
mod goals;
use jig_client_world::{Until, stories, test};
use skein_fake_person::{Find, Step, Target};
use skein_lib::Duration;
use skein_ui::Role;
const WITHIN: Duration = Duration::from_secs(1);
const CHECK_PARTY: u64 = 7;
const HELD_TASK: u64 = 9;
const RELEASE_ATTEMPT: skein_lib::Token = skein_lib::Token::new(9000);
const RELEASE_KEY: [u8; 16] = [91; 16];
const DELIVERED_HOLD: jig_client::contract::Revision = jig_client::contract::Revision { value: 4 };
const UNDELIVERED_HOLD: jig_client::contract::Revision = jig_client::contract::Revision { value: 99 };
fn raw_release(revision: jig_client::contract::Revision) {
    let mut world = stories::held_chat(11);
    let client = world.client(CHECK_PARTY, stories::chat());
    world.person(client, vec![Step::See { find: Find::named(Role::Button, b"Release"), within: WITHIN }]);
    world.run(Until::Done);
    world.send_raw(
        client,
        jig_client_test_domain::Request::Client {
            request: jig_client::Request::Down {
                down: jig_client::Down::Send {
                    attempt: RELEASE_ATTEMPT,
                    key: RELEASE_KEY,
                    request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Release {
                        project: 1,
                        task: HELD_TASK,
                        revision,
                        again: None,
                    })),
                    value: None,
                },
            },
        },
    );
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
}
#[test]
#[should_panic(expected = "jig's check: a decision names a revision never delivered")]
fn jig_s_check_fails_a_release_naming_a_revision_never_delivered() {
    raw_release(UNDELIVERED_HOLD);
}
#[test]
fn jig_s_check_passes_a_release_naming_the_delivered_revision() {
    raw_release(DELIVERED_HOLD);
}

fn plan() -> Target {
    Target::Role { role: Role::Region, name: Box::from(*b"Plan") }
}
#[test]
fn a_proposal_accepted_in_an_arranged_plan_leaves_while_its_chat_shows() {
    let mut w = stories::arranged_proposal_world(15);
    let c = w.client_with(7, stories::chat(), test::limits(), test::arranged);
    w.run(Until::Time(w.now().saturating_add(Duration::from_millis(80))));
    w.person(
        c,
        vec![
            Step::See { find: Find::named(Role::Log, b"Conversation"), within: WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Accept").within(Box::new([plan()])) },
            Step::Press { find: Find::named(Role::Button, b"Confirm") },
            Step::Gone { find: Find::named(Role::Button, b"Accept").within(Box::new([plan()])), within: WITHIN },
            Step::See { find: Find::named(Role::Log, b"Conversation"), within: WITHIN },
        ],
    );
    w.run(Until::Done);
}

#[test]
fn a_family_page_shows_its_tasks_plan_and_not_a_second_one() {
    let mut w = stories::two_plans_world(16);
    let c = w.client(7, stories::incident());
    w.person(
        c,
        vec![
            Step::See { find: Find::named(Role::Region, b"Plan"), within: WITHIN },
            Step::See { find: Find::text(b"Delegate ten"), within: WITHIN },
        ],
    );
    w.run(Until::Done);
}

const QUERY_PROJECT: u32 = 1;
const QUERY_PERSON: u64 = 7;
#[test]
fn a_project_s_tasks_section_watches_one_query_by_last_change() {
    use jig_client_domain::contract::{QueryOrder, Subject, TaskQuery};
    use jig_client_view::address::{Address, ProjectSection};
    let mut world = stories::chat_ready(18);
    world
        .front()
        .engine
        .act(
            skein_lib::Time::ZERO,
            skein_lib::Wall::EPOCH,
            jig_client_scripted_engine::Change::Project(Box::new(stories::project())),
        )
        .expect("query story fixture and delivery");
    let person = world.client(
        QUERY_PERSON,
        jig_client_test_domain::Address::Jig(Address::Project {
            project: QUERY_PROJECT,
            section: ProjectSection::Tasks,
        }),
    );
    world.person(person, vec![Step::See { find: Find::text(b"Chat nine"), within: WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed);
    let subjects: Vec<_> =
        world.front().opened.iter().filter(|s| matches!(s, Subject::Tasks { .. })).cloned().collect();
    assert_eq!(
        subjects,
        vec![Subject::Tasks {
            query: TaskQuery {
                project: Some(QUERY_PROJECT),
                requester: None,
                root: None,
                parent: None,
                label: None,
                phase: None,
                executor: None,
                tracked: None,
                order: QueryOrder::Changed
            },
            window: test::limits().client.domain.rows
        }]
    );
}

#[test]
fn the_test_application_s_own_proposal_view_is_held_to_jig_s_check() {
    let mut world = stories::own_proposal::world(31);
    let client = world.client(stories::own_proposal::PERSON, stories::own_proposal::address());
    let mut person = stories::own_proposal::person();
    person.insert(3, Step::See { find: Find::text(b"After"), within: WITHIN });
    world.person(client, person);
    assert_eq!(world.run(Until::Done).verdict, skein_world::domain::Verdict::Passed);
}
#[test]
#[should_panic(expected = "jig's check: a deciding act did not show its facts")]
fn an_own_proposal_view_hiding_its_effect_fails_jig_s_check() {
    let mut world = stories::own_proposal::world(31);
    let client = world.client_with(
        stories::own_proposal::PERSON,
        stories::own_proposal::address(),
        test::limits(),
        stories::own_proposal::hidden,
    );
    world.person(client, stories::own_proposal::person());
    world.run(Until::Done);
}

#[path = "../stories/projects.rs"]
mod projects;
