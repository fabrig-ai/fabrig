//! Closing a project is one durable decision (draft 5; people.md, 5.2).
use jig_core_world::{
    effects::World,
    fixtures::{ask_as, chat, delegate, live_row, member, people_member, second_party},
    reads::{Parties, ask, latest},
};
use jig_domain_core::{self as core, documents, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const PROJECT: u32 = 1;
const OWNER: u64 = 1;
const CONNECTOR: u16 = 1;
const KEY: u8 = 91;
const SEED: u64 = 1870;
const REASON: &[u8] = b"project deleted";

fn world() -> World {
    let (mut config, mut limits) = jig_core_world::faults::plan_fixture(SEED);
    config.core.grants =
        Box::new([people::ActGrant { connector: CONNECTOR, may: Box::new([people::ActKind::RemoveProject]) }]);
    limits.core.people.operations = 4; // A root and its delegate fit the compound.
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::configured(SEED, false, config, limits);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
    })));
    world
}
fn request(world: &mut World, key: u8, value: people::Ask) {
    ask_as(world, world.sign_in.expect("owner signed in"), key, value);
}
fn remove(world: &mut World, key: u8) {
    request(world, key, people::Ask::RemoveProject { project: PROJECT });
}
fn removed() -> people::Reply {
    people::Reply::Outcome(people::Outcome::ProjectRemoved { project: PROJECT })
}
fn unknown(world: &World) {
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown)))
    );
}
fn cancelled(world: &World, task: u64) {
    let row = match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) => row,
        _ => live_row(world, task),
    };
    let ending = match &row.phase {
        tasks::Phase::Closing(closing) => &closing.ending,
        tasks::Phase::Ended(ending) => ending,
        phase @ (tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. }) => {
            panic!("task {task} cancelled: {phase:?}")
        }
    };
    assert!(matches!(ending, tasks::Ending::Cancelled {
        by: tasks::Party::Deployment { project: PROJECT }, reason, ..
    } if reason.as_ref() == REASON));
}

#[test]
fn an_owner_removes_a_project_and_its_live_chat_is_cancelled_by_the_deployment() {
    let mut world = world();
    let run = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    let child = delegate(&mut world, run, 1, child);
    let answers = world.people_answers.len();
    remove(&mut world, KEY);
    assert_eq!(world.people_answers.len(), answers + 1);
    assert_eq!(world.people_answers.last(), Some(&removed()));
    cancelled(&world, run.0);
    cancelled(&world, child);
    assert_eq!(
        world.store.rows.get(&root::Key::Core(core::Key::Core(core::CoreKey::Removal(PROJECT)))),
        Some(&root::Record::Core(core::Record::Core(core::CoreRecord::Removal(core::Removal {
            project: PROJECT,
            stage: core::RemovalStage::Ending,
        }))))
    );
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Policy(PROJECT)))));
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Roles(PROJECT)))));
}

#[test]
fn a_member_who_is_no_owner_is_refused() {
    let mut world = world();
    let signed = second_party(&mut world, b"member", people::Role::Member);
    ask_as(&mut world, signed, KEY, people::Ask::RemoveProject { project: PROJECT });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role {
            held: Some(people::Role::Member)
        },)))
    );
    assert!(matches!(live_row(&world, world.assignments[0].0).phase, tasks::Phase::Active(_)));
}

#[test]
fn a_removed_project_refuses_a_new_task_a_message_and_a_rename_as_unknown() {
    let mut world = world();
    let task = world.assignments[0].0;
    let create = chat(&world, PROJECT, b"new chat".as_slice().into());
    remove(&mut world, KEY);
    for (key, value) in [
        (KEY + 1, create),
        (KEY + 2, people::Ask::Say { project: PROJECT, task, words: b"hello".as_slice().into() }),
        (KEY + 3, people::Ask::Rename { project: PROJECT, name: "Renamed".into() }),
        (
            KEY + 4,
            people::Ask::Adopt {
                project: PROJECT,
                adoption: people::Adoption {
                    resource: people::ResourceName { connector: CONNECTOR, path: Box::new([b"own".as_slice().into()]) },
                    role: people::ResourceRole::Owned,
                    options: Box::new([]),
                },
            },
        ),
        (KEY + 5, people::Ask::SetRoles { project: PROJECT, holdings: Box::new([]) }),
    ] {
        request(&mut world, key, value);
        unknown(&world);
    }
}

#[test]
fn a_removed_project_leaves_me_and_the_deployments_document() {
    let mut world = world();
    world.reads = Some(Parties::new(world.store.applied));
    remove(&mut world, KEY);
    ask(&mut world, 1000, reads::Read::Frame);
    let reads::Answer::Document(document) = latest(&world) else { panic!("me answer") };
    let documents::Document::Frame(frame) = &**document else { panic!("me") };
    assert!(frame.projects.is_empty());
    ask(&mut world, 1001, reads::Read::Deployment);
    let reads::Answer::Document(document) = latest(&world) else { panic!("deployment answer") };
    let documents::Document::Deployment(deployment) = &**document else { panic!("deployment") };
    assert!(deployment.projects.is_empty());
    ask(&mut world, 1002, reads::Read::Tasks { query: jig_core_world::fixtures::OWN_TASKS, live: None, ended: None });
    let reads::Answer::Tasks(page) = latest(&world) else { panic!("task query") };
    assert!(page.live.is_empty());
    let reads::EndedGroup::Rows { rows, .. } = &page.ended else { panic!("requester's ended tasks") };
    assert!(rows.is_empty());
}

#[test]
fn a_granted_connector_removes_a_project() {
    let mut world = world();
    let task = world.assignments[0].0;
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(1000)),
        connector: CONNECTOR,
        key: [KEY; 16],
        act: Box::new(people::Act::RemoveProject { project: PROJECT }),
        carried: None,
    }));
    assert_eq!(world.act_answers.last().expect("act answered").1, people::Outcome::ProjectRemoved { project: PROJECT });
    cancelled(&world, task);
}

#[test]
fn a_project_removed_twice_answers_the_saved_answer_then_unknown() {
    let mut world = world();
    remove(&mut world, KEY);
    let commits = world.commits.len();
    remove(&mut world, KEY);
    assert_eq!(world.people_answers.last(), Some(&removed()));
    assert_eq!(world.commits.len(), commits);
    remove(&mut world, KEY + 1);
    unknown(&world);
}

#[test]
fn a_waiting_and_a_held_task_end_at_once() {
    let mut world = world();
    let chat = world.assignments[0].0;
    let mut waiting = people_member(b"waiting", 0);
    waiting.dependencies = Box::new([people::Dependency::Existing(chat)]);
    request(&mut world, KEY, people::Ask::Create { project: PROJECT, batch: Box::new([waiting.clone()]) });
    let Some(people::Reply::Outcome(people::Outcome::Created { first, .. })) = world.people_answers.last() else {
        panic!("waiting task made: {:?}", world.people_answers.last())
    };
    let waiting_number = *first;
    request(&mut world, KEY + 1, people::Ask::Create { project: PROJECT, batch: Box::new([waiting]) });
    let Some(people::Reply::Outcome(people::Outcome::Created { first, .. })) = world.people_answers.last() else {
        panic!("held task made: {:?}", world.people_answers.last())
    };
    let (waiting, held) = (waiting_number, *first);
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task: held, why: tasks::Hold::Stopped })));
    remove(&mut world, KEY + 2);
    assert_eq!(world.people_answers.last(), Some(&removed()));
    cancelled(&world, held);
    cancelled(&world, waiting);
    for task in [held, waiting] {
        assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))));
    }
}

#[test]
fn a_closed_project_makes_no_repair_even_from_a_seed_admitted_before_removal() {
    let mut world = world();
    let seed = world.domain.core_mut().repair_seed(PROJECT).expect("repair admitted while open");
    let number = seed.number();
    remove(&mut world, KEY);
    assert!(world.domain.core_mut().repair_seed(PROJECT).is_none());
    world.send(root::Event::Core(core::Event::ConnectorRepair {
        seed: Box::new(seed),
        executor: tasks::Executor::Agent { charter: 1 },
        spec: tasks::Spec {
            asks: false,
            words: b"repair".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 128 },
        holdings: Box::new([]),
    }));
    for key in [tasks::Key::Live(number), tasks::Key::Ended(number)] {
        assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(key))));
    }
}

#[test]
fn a_closing_root_is_cancelled_by_the_deployment_on_removal() {
    let mut world = world();
    let task = world.assignments[0].0;
    request(
        &mut world,
        KEY,
        people::Ask::Cancel { project: PROJECT, task, reason: b"owner cancelled".as_slice().into(), grace: None },
    );
    remove(&mut world, KEY + 1);
    assert_eq!(world.people_answers.last(), Some(&removed()));
    cancelled(&world, task);
}
