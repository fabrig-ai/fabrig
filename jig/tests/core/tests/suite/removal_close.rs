//! Closed-at-once deletion (temper/docs/design/draft/playground-reset.md, 5).
use jig_core_world::{
    effects::{Cut, World},
    fixtures::live_row,
    stories::delegates::delegate_pair,
};
use jig_domain_core::{self as core, documents};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Env, ReplyTo, Time, Token, Wall};

const PROJECT: u32 = 1;
const CONNECTOR: u16 = 1;
const KEY: u8 = 91;
fn configuration(seed: u64) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_core_world::faults::plan_fixture(seed);
    limits.core.people.operations = 2;
    config.core.grants = Box::new([people::ActGrant { connector: CONNECTOR, may: people::ActKind::ALL.into() }]);
    jig_core_world::peers::journal(&mut limits);
    (config, limits)
}
fn world(seed: u64) -> World {
    let (config, limits) = configuration(seed);
    World::configured(seed, false, config, limits)
}
fn act(world: &mut World, key: u8, act: people::Act) {
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        connector: CONNECTOR,
        key: [key; 16],
        act: Box::new(act),
        carried: None,
    }));
}
fn remove(world: &mut World) {
    act(world, KEY, people::Act::RemoveProject { project: PROJECT });
}
fn answered(world: &World) -> &people::Outcome {
    &world.act_answers.last().expect("act answered").1
}
fn stages(world: &World) -> Vec<core::RemovalStage> {
    world
        .commits
        .iter()
        .flatten()
        .filter_map(|write| match write {
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Removal(row)))) => {
                Some(row.stage)
            }
            root::Write::CopySettingsRevision(_)
            | root::Write::Wires
            | root::Write::Save(_)
            | root::Write::Erase(_) => None,
        })
        .collect()
}
fn cancelled(world: &World, task: u64) {
    let row = match world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))) {
        Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) => row,
        _ => live_row(world, task),
    };
    let ending = match &row.phase {
        tasks::Phase::Closing(closing) => &closing.ending,
        tasks::Phase::Ended(ending) => ending,
        other @ (tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. }) => {
            panic!("task cancelled: {other:?}")
        }
    };
    assert!(matches!(ending, tasks::Ending::Cancelled { by: tasks::Party::Deployment { project: PROJECT }, .. }));
}
#[test]
fn the_deployments_remove_project_answers_project_removed() {
    let mut world = world(1900);
    let commit = world.store.applied + 1;
    world.cut = Some(Cut::Submitted(commit));
    remove(&mut world);
    assert_eq!(world.reached_cut, Some(Cut::Submitted(commit)));
    assert!(world.act_answers.is_empty(), "answer waits for the commit");
    assert!(stages(&world).contains(&core::RemovalStage::Ending));
    world.cut = None;
    world.drain();
    assert_eq!(answered(&world), &people::Outcome::ProjectRemoved { project: PROJECT });
}
#[test]
fn an_owners_remove_project_answers_project_removed() {
    let mut world = world(1901);
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(1901)),
        sign_in: world.sign_in.expect("signed in"),
        key: [KEY; 16],
        ask: people::Ask::RemoveProject { project: PROJECT },
    })));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::ProjectRemoved { project: PROJECT }))
    );
}
#[test]
fn a_project_gone_or_being_removed_is_unknown() {
    let mut world = world(1902);
    remove(&mut world);
    for (key, project) in [(KEY + 1, PROJECT), (KEY + 2, u32::MAX)] {
        act(&mut world, key, people::Act::RemoveProject { project });
        assert_eq!(answered(&world), &people::Outcome::Refused(people::Refusal::Unknown));
    }
}
#[test]
fn a_removing_project_takes_no_new_work() {
    let mut world = world(1903);
    let task = world.assignments[0].0;
    let (config, limits) = configuration(1903);
    let creating = jig_core_world::fixtures::Creating::new(&config.core, limits.core.tasks.tree_tasks);
    let new_chat = jig_core_world::fixtures::create_chat(&creating, PROJECT, 0, b"new chat".as_slice().into());
    remove(&mut world);
    for (key, act_value) in [
        (KEY + 1, people::Act::Close { project: PROJECT, task, grace: None, reason: Box::new([]) }),
        (KEY + 2, people::Act::Cancel { project: PROJECT, task, grace: None, reason: Box::new([]) }),
        (
            KEY + 3,
            people::Act::Adopt {
                project: PROJECT,
                adoption: people::Adoption {
                    resource: people::ResourceName { connector: CONNECTOR, path: Box::new([Box::from(&b"own"[..])]) },
                    role: people::ResourceRole::Owned,
                    options: Box::new([]),
                },
            },
        ),
    ] {
        act(&mut world, key, act_value);
        assert_eq!(answered(&world), &people::Outcome::Refused(people::Refusal::Unknown));
    }
    for (key, ask) in [
        (KEY + 4, people::Ask::Rename { project: PROJECT, name: "Gone".into() }),
        (KEY + 5, people::Ask::Say { project: PROJECT, task, words: b"hello".as_slice().into() }),
        (
            KEY + 6,
            people::Ask::SetRoles {
                project: PROJECT,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            },
        ),
        (KEY + 7, new_chat),
        (
            KEY + 8,
            people::Ask::ChangePolicy {
                project: PROJECT,
                change: people::PolicyChange::ProjectSpend { period_spend: 100 },
            },
        ),
        (
            KEY + 9,
            people::Ask::Application {
                scope: people::ApplicationScope::Project(PROJECT),
                connector: CONNECTOR,
                digest: [0; 32],
            },
        ),
    ] {
        world.send(root::Event::Core(core::Event::People(people::Event::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: world.sign_in.expect("signed in"),
            key: [key; 16],
            ask,
        })));
        assert_eq!(
            world.people_answers.last(),
            Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown)))
        );
    }
}
#[test]
fn a_removing_project_leaves_every_list() {
    let mut world = world(1904);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
    })));
    remove(&mut world);
    let core = world.domain.core_mut();
    let limits = &configuration(1904).1.core;
    let document = documents::make(
        core,
        limits,
        1,
        &documents::Subject::Frame { party: 1 },
        &documents::Window { rows: 10, after: None },
        Wall::EPOCH,
    )
    .expect("frame");
    let documents::Document::Frame(frame) = document else { panic!("frame") };
    assert!(frame.projects.is_empty());
    let document = documents::make(
        core,
        limits,
        1,
        &documents::Subject::Deployment,
        &documents::Window { rows: 10, after: None },
        Wall::EPOCH,
    )
    .expect("deployment");
    let documents::Document::Deployment(deployment) = document else { panic!("deployment") };
    assert!(deployment.projects.is_empty());
}
#[test]
fn its_live_tasks_are_cancelled() {
    let mut world = world(1905);
    let root = world.assignments[0].0;
    let (first, second) = delegate_pair(&mut world);
    let (period_root, period_child) = recurring(&mut world);
    assert_eq!(live_row(&world, period_root).requester, tasks::Party::Deployment { project: PROJECT });
    let answers = world.people_answers.len();
    remove(&mut world);
    for task in [root, first, second, period_root, period_child] {
        cancelled(&world, task);
    }
    assert!(world.cancellations.iter().any(|(task, _)| *task == root));
    assert!(live_row(&world, root).grace.is_none(), "Close with grace None cancels immediately");
    assert_eq!(world.act_answers.len(), 1);
    assert_eq!(world.people_answers.len(), answers, "compound answers are swallowed");
}
#[test]
fn the_last_live_task_ending_moves_the_row_to_erasing() {
    for assigned in [false, true] {
        let (config, limits) = configuration(1906);
        let mut world =
            if assigned { World::configured(1906, false, config, limits) } else { World::first_start(config, limits) };
        remove(&mut world);
        if assigned {
            assert_eq!(world.domain.core_mut().removal(PROJECT).expect("ending row").stage, core::RemovalStage::Ending);
            let before = world.commits.len();
            world.end(tasks::End::Parked, 0);
            assert!(
                world.commits.iter().skip(before).any(|writes| {
                    writes.iter().any(|write| {
                        matches!(
                            write,
                            root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(_))))
                        )
                    }) && writes.iter().any(|write| {
                        matches!(
                            write,
                            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Removal(
                                core::ProjectRemoval { stage: core::RemovalStage::Rows, .. }
                            ))))
                        )
                    })
                }),
                "the last task and advanced removal share a commit"
            );
        }
        assert!(stages(&world).contains(&core::RemovalStage::Rows));
        assert_eq!(world.domain.core_mut().removal(PROJECT).expect("kept row").stage, core::RemovalStage::Rows);
    }
}
#[test]
fn a_restart_keeps_the_project_closed() {
    let mut world = world(1907);
    let (task, attempt) = world.assignments[0];
    remove(&mut world);
    world.restart_with(configuration(1907).0);
    assert!(world.domain.core_mut().removing(PROJECT));
    let limits = configuration(1907).1.core;
    let documents::Document::Frame(frame) = documents::make(
        world.domain.core_mut(),
        &limits,
        1,
        &documents::Subject::Frame { party: 1 },
        &documents::Window { rows: 10, after: None },
        Wall::EPOCH,
    )
    .expect("restored frame") else {
        panic!("frame")
    };
    assert!(frame.projects.is_empty());
    act(&mut world, KEY + 1, people::Act::RemoveProject { project: PROJECT });
    assert_eq!(answered(&world), &people::Outcome::Refused(people::Refusal::Unknown));
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: fleet::Hello {
            bounds: jig_core_world::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([fleet::Hosted {
                run: Token::new(task),
                attempt: Token::new(attempt),
                phase: fleet::Phase::Active,
            }]),
        },
    })));
    world.end(tasks::End::Parked, 0);
    assert_eq!(world.domain.core_mut().removal(PROJECT).expect("resumed deletion").stage, core::RemovalStage::Rows);
}

#[test]
fn a_non_owner_cannot_remove_a_project() {
    let mut world = world(1908);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([people::Holding { person: 1, role: people::Role::Member }]),
    })));
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(1908)),
        sign_in: world.sign_in.expect("signed in"),
        key: [KEY; 16],
        ask: people::Ask::RemoveProject { project: PROJECT },
    })));
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role {
            held: Some(people::Role::Member)
        })))
    );
    assert!(!world.domain.core_mut().removing(PROJECT));
}

#[test]
fn another_project_still_takes_requests_and_stays_listed() {
    let (mut config, limits) = jig_core_world::fixtures::two_projects(1909);
    config.core.grants =
        Box::new([people::ActGrant { connector: CONNECTOR, may: Box::new([people::ActKind::RemoveProject]) }]);
    let mut world = World::configured(1909, false, config, limits);
    remove(&mut world);
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(1909)),
        sign_in: world.sign_in.expect("signed in"),
        key: [KEY + 1; 16],
        ask: people::Ask::Rename { project: 2, name: "Remaining".into() },
    })));
    assert_eq!(world.people_answers.last(), Some(&people::Reply::Outcome(people::Outcome::Renamed { project: 2 })));
    let limits = jig_core_world::fixtures::two_projects(1909).1.core;
    let documents::Document::Frame(frame) = documents::make(
        world.domain.core_mut(),
        &limits,
        1,
        &documents::Subject::Frame { party: 1 },
        &documents::Window { rows: 10, after: None },
        Wall::EPOCH,
    )
    .expect("frame") else {
        panic!("frame")
    };
    assert_eq!(frame.projects.len(), 1);
    assert_eq!(frame.projects.as_slice()[0].project, 2);
    assert!(!frame.cut);
}

#[test]
fn no_live_task_enters_erasing_in_the_close_decision() {
    let (config, limits) = configuration(1910);
    let rows = jig_core_world::fixtures::seeded_project_rows(&config.core);
    let mut core = core::Core::new(config.core, &limits.core).expect("core");
    for row in rows {
        let restored = match row {
            core::Record::Core(row) => core.restore_core(row, &limits.core) != core::Restored::Rejected,
            core::Record::People(people::Stored::Policy { project, name, value }) => {
                core.restore_policy(&limits.core, project, name, value)
            }
            core::Record::Tasks(_) | core::Record::People(_) | core::Record::Notes(_) | core::Record::Accounts(_) => {
                false
            }
        };
        assert!(restored);
    }
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.core };
    let _restored = core::step(&mut core, &env, core::Event::People(people::Event::Restored));
    let core::Requests::Out(mut out) = core::step(
        &mut core,
        &env,
        core::Event::Act {
            reply_to: ReplyTo::new(Token::new(1910)),
            connector: CONNECTOR,
            key: [KEY; 16],
            act: Box::new(people::Act::RemoveProject { project: PROJECT }),
            carried: None,
        },
    );
    let mut stages = Vec::new();
    while let Some(request) = out.pop() {
        if let core::Request::Write(core::Write::Save(core::Record::Core(core::CoreRecord::Removal(row)))) = request {
            stages.push(row.stage);
        }
    }
    assert_eq!(
        stages,
        [
            core::RemovalStage::Ending,
            core::RemovalStage::Erasing { after: None },
            core::RemovalStage::Notes,
            core::RemovalStage::Rows
        ]
    );
}

fn recurring(world: &mut World) -> (u64, u64) {
    let member = jig_core_world::fixtures::member(world, tasks::Executor::Agent { charter: 1 }, b"period work");
    let mut authority = member.authority.clone();
    authority.budget.spend = 30;
    let mut child = member.authority;
    child.budget.spend = 10;
    child.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::StartRecurring {
        project: PROJECT,
        authority,
        template: tasks::RecurringTemplate {
            key: 1,
            overlap: tasks::RecurringOverlap::Skip,
            batch: Box::new([tasks::New {
                deadline: tasks::Deadline::Named,
                label: 0,
                flags: tasks::Flags::PLAIN,
                number: 1,
                project: PROJECT,
                executor: tasks::Executor::Agent { charter: 1 },
                llm: None,
                spec: member.spec,
                contract: member.contract,
                authority: child,
                numbers: tasks::Numbers { budget: 10, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: PROJECT, period: 0 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            }]),
        },
    }));
    let task = world
        .store
        .rows
        .values()
        .find_map(|record| {
            let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record else { return None };
            if row.requester == (tasks::Party::Deployment { project: PROJECT }) && row.recurring.is_some() {
                Some(row.number)
            } else {
                None
            }
        })
        .expect("deployment's recurring root");
    let delegate = world
        .store
        .rows
        .values()
        .find_map(|record| {
            let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record else { return None };
            if row.requester == tasks::Party::Task(task) { Some(row.number) } else { None }
        })
        .expect("recurring delegate, even before a worker is available");
    (task, delegate)
}

#[test]
fn no_compound_room_refuses_before_closing_the_project() {
    let (config, mut limits) = configuration(1911);
    limits.core.people.operations = 0;
    let mut world = World::configured(1911, false, config, limits);
    remove(&mut world);
    assert_eq!(
        answered(&world),
        &people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Operations })
    );
    assert!(!world.domain.core_mut().removing(PROJECT));
    assert!(stages(&world).is_empty());
}
