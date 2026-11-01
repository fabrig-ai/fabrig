//! Project deletion closes admission before ending work (playground-reset contracts J1–J4, J8).
use jig_core_world::{
    client_peer::ClientPeer,
    effects::{Cut, World},
    fixtures::{ask_as, chat, delegate, live_row, member},
    reads::{ask, latest},
};
use jig_domain_core::{self as core, documents, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, List, ReplyTo, Token, Wall};

const PROJECT: u32 = 1;
const OTHER: u32 = 2;
const KEY: u8 = 91;
const GRACE: Duration = Duration::from_secs(5);
const REASON: &[u8] = b"Project deleted";

fn configuration(seed: u64) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_core_world::faults::plan_fixture(seed);
    limits.core.authority.projects = 2;
    limits.core.people.projects = 3;
    limits.core.people.initial_owners = 2;
    limits.core.people.requests = 32;
    limits.core.views.watchers = 8;
    limits.core.views.watchers_per_party = 8;
    config.core.defaults.rules =
        jig_domain_authority::Domain::new(config.core.defaults.rules.clone(), limits.core.authority)
            .expect("two projects")
            .rules()
            .clone();
    let first = config.core.seed_projects[0].policy.policy.clone();
    let second = core::SeedProject {
        number: OTHER,
        name: "Other".into(),
        policy: core::ConfiguredPolicy {
            policy: first,
            permissions: config.core.seed_projects[0].policy.permissions.clone(),
            templates: config.core.seed_projects[0].policy.templates.clone(),
        },
    };
    let mut projects = config.core.seed_projects.into_vec();
    projects.push(second);
    config.core.seed_projects = projects.into_boxed_slice();
    config.core.owners = Box::new([
        config.core.owners[0].clone(),
        people::InitialOwner {
            project: OTHER,
            identity: people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
        },
    ]);
    config.core.grants = Box::new([people::ActGrant {
        connector: 1,
        may: Box::new([
            people::ActKind::Close,
            people::ActKind::RemoveProject,
            people::ActKind::Cancel,
            people::ActKind::Adopt,
        ]),
    }]);
    jig_core_world::peers::journal(&mut limits);
    (config, limits)
}
fn world(seed: u64, tasks: bool) -> World {
    let (config, limits) = configuration(seed);
    let mut world = if tasks {
        World::configured(seed, false, config, limits)
    } else {
        World::configured_with(
            seed,
            false,
            config,
            limits,
            people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } },
        )
    };
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    world
}
fn act(world: &mut World, key: u8, act: people::Act) {
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        connector: 1,
        key: [key; 16],
        act: Box::new(act),
        carried: None,
    }));
}
fn remove(world: &mut World, key: u8, project: u32) {
    act(world, key, people::Act::RemoveProject { project });
}
fn answer(world: &World) -> &people::Outcome {
    &world.act_answers.last().expect("act answered").1
}
fn stage(world: &World, project: u32) -> people::Stage {
    let key = root::Key::Core(core::Key::People(people::Key::Deletion(project)));
    let root::Record::Core(core::Record::People(people::Stored::Deletion { stage, .. })) = &world.store.rows[&key]
    else {
        panic!("deletion row")
    };
    *stage
}
fn request(world: &mut World, key: u8, ask: people::Ask) {
    ask_as(world, world.sign_in.expect("signed in"), key, ask);
}
fn refusal(world: &World) {
    jig_core_world::fixtures::refusal(world, people::Refusal::Unknown);
}
fn cancelled(world: &World, task: u64, reason: &[u8]) {
    let key = root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)));
    let row =
        if let Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) = world.store.rows.get(&key) {
            row
        } else {
            live_row(world, task)
        };
    let ending = match &row.phase {
        tasks::Phase::Closing(closing) => &closing.ending,
        tasks::Phase::Ended(ending) => ending,
        other @ (tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. }) => {
            panic!("cancelled task {task}: {other:?}")
        }
    };
    assert!(matches!(ending, tasks::Ending::Cancelled {
        by: tasks::Party::Deployment { project: PROJECT }, reason: given, ..
    } if given.as_ref() == reason));
}
fn end_cancelled_runs(world: &mut World) {
    for (task, attempt) in world.cancellations.clone() {
        world.send(root::Event::Answer {
            channel: Token::new(7),
            task,
            attempt,
            cumulative: 0,
            read: None,
            refused: Box::new([]),
            end: tasks::End::Parked,
        });
    }
}
fn other_chat(world: &mut World, key: u8, project: u32) -> u64 {
    let value = chat(world, project, b"second chat".as_slice().into());
    request(world, key, value);
    let Some(people::Reply::Outcome(people::Outcome::Created { first, .. })) = world.people_answers.last() else {
        panic!("chat created: {:?}", world.people_answers.last())
    };
    *first
}

#[test]
fn a_granted_connector_deletes_a_project_and_its_tasks_end() {
    let mut world = world(1900, true);
    let first = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    let child = delegate(&mut world, first, 1, child);
    let second = other_chat(&mut world, 60, PROJECT);
    world.client_peer = Some(ClientPeer::routed());
    for (watcher, subject) in
        [(701, documents::Watched::Task { task: first.0 }), (702, documents::Watched::Tree { task: child })]
    {
        world.send(root::Event::Core(core::Event::Watch {
            watcher: Token::new(watcher),
            sign_in: world.sign_in.expect("deletion fixture supplies this value"),
            subject,
            window: documents::Window { rows: 2, after: None },
        }));
        world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(watcher) }));
    }
    remove(&mut world, KEY, PROJECT);
    assert_eq!(answer(&world), &people::Outcome::ProjectRemoved { project: PROJECT });
    assert_eq!(stage(&world, PROJECT), people::Stage::Ending);
    for number in [701, 702] {
        assert!(world.client_peer.as_ref().expect("deletion fixture supplies this value").delivered.iter().any(
            |(_, delivery)| matches!(delivery,
            root::Delivery::Core(core::Held::WatchEnded { watcher, why: jig_domain_views::End::Gone })
            if watcher.raw() == number)
        ));
    }
    world.client_peer = None;
    for task in [first.0, second] {
        let finish = live_row(&world, task).grace.as_ref().expect("conversation has grace");
        assert_eq!(finish.by, tasks::Party::Deployment { project: PROJECT });
        assert_eq!(finish.reason.as_ref(), REASON);
    }
    world.advance(Duration::from_secs(4));
    assert_eq!(stage(&world, PROJECT), people::Stage::Ending);
    assert!(live_row(&world, first.0).grace.is_some());
    world.advance(Duration::from_secs(1));
    world.advance(Duration::ZERO);
    for task in [first.0, second, child] {
        cancelled(&world, task, REASON);
    }
    end_cancelled_runs(&mut world);
    assert_eq!(stage(&world, PROJECT), people::Stage::Erasing);
}

#[test]
fn an_owner_deletes_a_project_from_its_document_offer() {
    let mut world = world(1901, false);
    let owner = world.sign_in.expect("deletion fixture supplies this value");
    ask(&mut world, 1, reads::Read::Project { project: PROJECT });
    let reads::Answer::Document(value) = latest(&world) else { panic!("project document") };
    let documents::Document::Project(value) = value.as_ref() else { panic!("project") };
    assert!(
        value
            .acts
            .iter()
            .any(|offer| offer.act == core::acts::Act::RemoveProject
                && offer.admission == core::acts::Admission::Allowed)
    );
    world.send(root::Event::Core(core::Event::SignIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(900)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
            login: b"member".as_slice().into(),
            name: b"Member".as_slice().into(),
        },
    }));
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([
            people::Holding { person: 1, role: people::Role::Owner },
            people::Holding { person: 2, role: people::Role::Member },
        ]),
    })));
    let limits = configuration(1901).1.core;
    let value = documents::make(
        world.domain.core_mut(),
        &limits,
        2,
        &documents::Subject::Project { project: PROJECT },
        &documents::Window { rows: 2, after: None },
        Wall::EPOCH,
    )
    .expect("member project");
    let documents::Document::Project(value) = value else { panic!("project") };
    assert!(!value.acts.iter().any(|offer| offer.act == core::acts::Act::RemoveProject));
    request(&mut world, KEY, people::Ask::RemoveProject { project: PROJECT });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role {
            held: Some(people::Role::Member)
        })))
    );
    ask_as(&mut world, owner, KEY, people::Ask::RemoveProject { project: PROJECT });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::ProjectRemoved { project: PROJECT }))
    );
}

#[test]
fn a_deleting_project_refuses_every_request_naming_it() {
    let mut world = world(1902, true);
    let task = world.assignments[0].0;
    let new_chat = chat(&world, PROJECT, b"new chat".as_slice().into());
    remove(&mut world, KEY, PROJECT);
    for (key, value) in [
        (61, new_chat),
        (62, people::Ask::Say { project: PROJECT, task, words: b"message".as_slice().into() }),
        (
            63,
            people::Ask::ChangePolicy {
                project: PROJECT,
                change: people::PolicyChange::Templates { overrides: Box::new([]) },
            },
        ),
        (64, people::Ask::RemoveProject { project: PROJECT }),
        (65, people::Ask::Say { project: OTHER, task, words: b"wrong project".as_slice().into() }),
    ] {
        request(&mut world, key, value);
        refusal(&world);
    }
    let mut operations = List::with_capacity(1);
    operations
        .push(people::Operation::Message { task, words: b"compound".as_slice().into() })
        .expect("deletion fixture supplies this value");
    request(&mut world, 67, people::Ask::Compound { project: OTHER, operations });
    refusal(&world);
    world.send(root::Event::Application {
        reply_to: ReplyTo::new(Token::new(40000)),
        sign_in: world.sign_in.expect("deletion fixture supplies this value"),
        key: [68; 16],
        scope: people::ApplicationScope::Project(PROJECT),
        number: 1,
        digest: [68; 32],
        value: jig_test_connector::ApplicationRequest { object: 1, revision: 0, value: 11, release: None },
        carried: None,
    });
    refusal(&world);
    act(
        &mut world,
        KEY + 1,
        people::Act::Adopt {
            project: PROJECT,
            adoption: people::Adoption {
                resource: people::ResourceName { connector: 1, path: Box::new([b"own".as_slice().into()]) },
                role: people::ResourceRole::Owned,
                options: Box::new([]),
            },
        },
    );
    assert_eq!(answer(&world), &people::Outcome::Refused(people::Refusal::Unknown));
    for (key, read) in [(1, reads::Read::Project { project: PROJECT }), (2, reads::Read::Task { task })] {
        ask(&mut world, key, read);
        assert_eq!(latest(&world), &reads::Answer::None(reads::Why::Unknown));
    }
    world.client_peer = Some(ClientPeer::default());
    world.send(root::Event::Core(core::Event::Watch {
        watcher: Token::new(700),
        sign_in: world.sign_in.expect("deletion fixture supplies this value"),
        subject: documents::Watched::Task { task },
        window: documents::Window { rows: 2, after: None },
    }));
    assert!(
        world
            .client_peer
            .as_ref()
            .expect("deletion fixture supplies this value")
            .now
            .iter()
            .any(|now| matches!(now, core::Now::WatchRefused { why: core::WatchRefusal::Unknown, .. }))
    );
    world.client_peer = None;
    remove(&mut world, KEY + 2, PROJECT);
    assert_eq!(answer(&world), &people::Outcome::Refused(people::Refusal::Unknown));
    let other = other_chat(&mut world, 66, OTHER);
    ask(&mut world, 3, reads::Read::Task { task: other });
    assert!(matches!(latest(&world), reads::Answer::Document(_)));
}

#[test]
fn a_deleting_project_leaves_the_frame_and_the_deployment_list() {
    let mut world = world(1903, true);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
    })));
    remove(&mut world, KEY, PROJECT);
    ask(&mut world, 1, reads::Read::Frame);
    let reads::Answer::Document(value) = latest(&world) else { panic!("frame") };
    let documents::Document::Frame(value) = value.as_ref() else { panic!("frame") };
    assert_eq!(value.projects.iter().map(|project| project.project).collect::<Vec<_>>(), [OTHER]);
    ask(&mut world, 2, reads::Read::Deployment);
    let reads::Answer::Document(value) = latest(&world) else { panic!("deployment") };
    let documents::Document::Deployment(value) = value.as_ref() else { panic!("deployment") };
    assert_eq!(value.projects.iter().map(|project| project.project).collect::<Vec<_>>(), [OTHER]);
}

#[test]
fn a_project_without_tasks_goes_straight_to_erasing() {
    let mut world = world(1904, false);
    remove(&mut world, KEY, PROJECT);
    assert_eq!(answer(&world), &people::Outcome::ProjectRemoved { project: PROJECT });
    assert_eq!(stage(&world, PROJECT), people::Stage::Erasing);
    let writes = world.commits.last().expect("deletion fixture supplies this value");
    let stages: Vec<_> = writes
        .iter()
        .filter_map(|write| match write {
            root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Deletion {
                project: PROJECT,
                stage,
            }))) => Some(*stage),
            root::Write::Save(_)
            | root::Write::Erase(_)
            | root::Write::CopySettingsRevision(_)
            | root::Write::Wires
            | root::Write::DropTranscript { .. } => None,
        })
        .collect();
    assert_eq!(stages, [people::Stage::Ending, people::Stage::Erasing]);
}

#[test]
fn an_unknown_or_deleting_project_is_refused_unknown_and_the_answer_is_kept() {
    let mut world = world(1905, true);
    remove(&mut world, KEY, PROJECT);
    let commits = world.commits.len();
    remove(&mut world, KEY, PROJECT);
    assert_eq!(answer(&world), &people::Outcome::ProjectRemoved { project: PROJECT });
    assert_eq!(world.commits.len(), commits);
    for (key, project) in [(KEY + 1, PROJECT), (KEY + 2, u32::MAX)] {
        remove(&mut world, key, project);
        assert_eq!(answer(&world), &people::Outcome::Refused(people::Refusal::Unknown));
        let commits = world.commits.len();
        remove(&mut world, key, project);
        assert_eq!(answer(&world), &people::Outcome::Refused(people::Refusal::Unknown));
        assert_eq!(world.commits.len(), commits);
    }
}

#[test]
fn a_start_during_ending_resumes_it() {
    let mut world = world(1906, true);
    let task = world.assignments[0].0;
    let cut = Cut::Durable(world.store.applied + 1);
    world.cut = Some(cut);
    remove(&mut world, KEY, PROJECT);
    assert_eq!(world.reached_cut, Some(cut));
    assert!(world.act_answers.is_empty());
    assert_eq!(stage(&world, PROJECT), people::Stage::Ending);
    world.restart_with(configuration(1906).0);
    remove(&mut world, KEY, PROJECT);
    assert_eq!(answer(&world), &people::Outcome::ProjectRemoved { project: PROJECT });
    world.advance(GRACE);
    cancelled(&world, task, REASON);
    end_cancelled_runs(&mut world);
    assert_eq!(stage(&world, PROJECT), people::Stage::Erasing);
}

#[test]
fn the_cancel_act_cancels_any_live_task_of_its_project() {
    let mut world = world(1907, true);
    let source = live_row(&world, world.assignments[0].0).clone();
    let core = world.domain.core_mut();
    let number = core::fresh(&mut core.counters, core::Family::Task).expect("deletion fixture supplies this value");
    let period = core.period(PROJECT);
    let mut authority = source.authority;
    authority.budget.spend = 100;
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Make {
        reply_to: ReplyTo::new(Token::new(u64::MAX - 3)),
        creator: tasks::Party::Deployment { project: PROJECT },
        batch: Box::new([tasks::New {
            number,
            project: PROJECT,
            label: 0,
            flags: tasks::Flags::PLAIN,
            executor: tasks::Executor::Agent { charter: 1 },
            llm: None,
            spec: source.spec,
            contract: source.contract,
            authority,
            deadline: tasks::Deadline::Named,
            numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            funder: tasks::Funder::Period { project: PROJECT, period },
            dependencies: Box::new([]),
            holdings: Box::new([]),
            wake: tasks::WakePolicy::DEFAULT,
            recurring: None,
            tracked: None,
        }]),
    })));
    let parent = *world.assignments.last().expect("deletion fixture supplies this value");
    assert_eq!(parent.0, number);
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    let child = delegate(&mut world, parent, 1, child);
    act(&mut world, KEY, people::Act::Cancel { project: OTHER, task: parent.0, reason: b"cancel".as_slice().into() });
    assert_eq!(answer(&world), &people::Outcome::Refused(people::Refusal::Unknown));
    act(
        &mut world,
        KEY + 1,
        people::Act::Cancel { project: PROJECT, task: parent.0, reason: b"cancel".as_slice().into() },
    );
    assert!(matches!(answer(&world), people::Outcome::Compound { project: PROJECT, operations: 2, .. }));
    for task in [parent.0, child] {
        cancelled(&world, task, b"cancel");
    }
    end_cancelled_runs(&mut world);
    act(
        &mut world,
        KEY + 2,
        people::Act::Cancel { project: PROJECT, task: parent.0, reason: b"cancel".as_slice().into() },
    );
    assert!(matches!(answer(&world), people::Outcome::Refused(people::Refusal::Ended { .. })));
}

#[test]
fn a_start_resumes_an_ending_row_with_uncancelled_roots_or_no_tasks() {
    for (seed, tasks) in [(1908, true), (1909, false)] {
        let mut world = world(seed, tasks);
        world.send(root::Event::Core(core::Event::People(people::Event::Deletion {
            project: PROJECT,
            stage: Some(people::Stage::Ending),
        })));
        world.restart_with(configuration(seed).0);
        if tasks {
            let task = world.assignments.last().expect("deletion fixture supplies this value").0;
            assert!(live_row(&world, task).grace.is_some());
            assert_eq!(stage(&world, PROJECT), people::Stage::Ending);
            world.advance(GRACE);
            end_cancelled_runs(&mut world);
        }
        assert_eq!(stage(&world, PROJECT), people::Stage::Erasing);
    }
}

#[test]
fn a_deleting_project_refuses_new_delegates_during_its_conversations_grace() {
    let mut world = world(1910, true);
    let parent = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"late delegate");
    remove(&mut world, KEY, PROJECT);
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9001)),
        key: core::CallKey { task: parent.0, attempt: parent.1, completion: 1, position: 0 },
        batch: Box::new([child]),
        stubs: Box::new([]),
    }));
    assert!(matches!(world.answers.last().expect("deletion fixture supplies this value").2,
        core::CallPart::DelegationRefused(ref problem) if problem.why == tasks::Refusal::Unknown));
}

#[test]
fn a_read_waiting_for_history_drops_the_deleting_projects_live_snapshot() {
    let mut world = world(1911, true);
    world.reads.as_mut().expect("deletion fixture supplies this value").hold_loads = true;
    ask(&mut world, 1, reads::Read::Tasks { query: jig_core_world::fixtures::OWN_TASKS, live: None, ended: None });
    assert!(world.reads.as_ref().expect("deletion fixture supplies this value").paused());
    remove(&mut world, KEY, PROJECT);
    jig_core_world::reads::complete(&mut world);
    let reads::Answer::Tasks(page) = latest(&world) else { panic!("task query") };
    assert!(page.live.is_empty());
}
