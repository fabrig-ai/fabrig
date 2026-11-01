//! Project deletion closes admission before ending work (playground-reset contracts J1–J4, J8).
use jig_core_world::stories::reads::{request as erasure_request, world as erasure_world};
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
    if world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Deletion(PROJECT)))) {
        // These Ending checks retain the rows until their cancellation assertions finish.
        world.reads.as_mut().expect("deletion reads").hold_loads = true;
    }
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
    world.reads.as_mut().expect("deletion reads").hold_loads = false;
    jig_core_world::reads::complete(&mut world);
    released(&world, PROJECT);
    for task in [first.0, second, child] {
        assert!(!world.store.rows.contains_key(&ended(task)));
    }
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
    released(&world, PROJECT);
    let writes = world
        .commits
        .iter()
        .find(|writes| {
            writes.contains(&root::Write::Save(root::Record::Core(core::Record::People(people::Stored::Deletion {
                project: PROJECT,
                stage: people::Stage::Ending,
            }))))
        })
        .expect("deletion's first decision");
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
    world.reads.as_mut().expect("deletion reads").hold_loads = true;
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
        world.reads.as_mut().expect("deletion reads").hold_loads = true;
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

const SEED: u64 = 911;

fn ended(task: u64) -> root::Key {
    root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task)))
}

fn finished(turns: u32) -> (World, u64) {
    let mut world = erasure_world(SEED);
    let task = world.assigned.as_ref().expect("conversation").task;
    for number in 1..=turns {
        world.turn(number, u64::from(number), None, b"reply");
    }
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        u64::from(turns),
    );
    assert!(world.store.rows.contains_key(&ended(task)));
    (world, task)
}

fn erase(world: &mut World) {
    world.send(root::Event::Core(core::Event::People(people::Event::Deletion {
        project: 1,
        stage: Some(people::Stage::Erasing),
    })));
}

fn released(world: &World, project: u32) {
    assert!(!world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::Deletion(project)))));
}

fn erasure_configuration() -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = jig_core_world::faults::plan_fixture(SEED);
    limits.core.fleet.slots = 2;
    limits.core.tasks.executor_kinds = 3;
    limits.core.authority.executors = 3;
    limits.core.documents.executors = 3;
    limits.core.authority.projects = 2;
    limits.core.people.projects = 3;
    limits.core.people.initial_owners = 2;
    limits.core.people.inbox_recipients = 24;
    let kinds: Box<[jig_domain_authority::Executor]> = Box::new([
        jig_domain_authority::Executor::Charter(1),
        jig_domain_authority::Executor::Procedure(1),
        jig_domain_authority::Executor::Role(0),
    ]);
    configuration.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    let policy = &mut configuration.core.seed_projects[0].policy;
    policy.policy.ceiling.delegation.kinds.clone_from(&kinds);
    for role in &mut policy.policy.roles {
        role.authority.delegation.kinds.clone_from(&kinds);
    }
    let second = core::SeedProject { number: 2, name: "Other project".into(), policy: policy.clone() };
    let first =
        core::SeedProject { number: 1, name: configuration.core.seed_projects[0].name.clone(), policy: policy.clone() };
    configuration.core.seed_projects = Box::new([first, second]);
    let owner = configuration.core.owners[0].clone();
    configuration.core.owners =
        Box::new([owner.clone(), people::InitialOwner { project: 2, identity: owner.identity }]);
    jig_core_world::peers::journal(&mut limits);
    (configuration, limits)
}

#[expect(
    clippy::too_many_lines,
    reason = "one project fixture includes delegation, a question, a person, a proposal and the unaffected project"
)]
fn rich_world() -> (World, Vec<u64>, u64, u64) {
    let (configuration, limits) = erasure_configuration();
    let mut world =
        World::configured_with_startup(SEED, configuration, limits, |domain| domain.core_mut().person_answers());
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    let task = world.assigned.as_ref().expect("conversation").task;
    let parent_attempt = world.assigned.as_ref().expect("conversation").attempt;
    world.turn(1, 1, None, b"reply with transcript");
    let mut member = jig_core_world::fixtures::member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(9100)),
        key: world.call_key(1),
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
    let delegate = world.assigned.as_ref().expect("agent delegate").task;
    assert_ne!(delegate, task, "delegate: {:?}", world.answers);
    world.turn(1, 1, None, b"delegate transcript");
    world.send(root::Event::Core(core::Event::NamedAction {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(9101)),
        key: world.call_key(1),
        action: core::NamedAction::Message {
            target: task,
            kind: tasks::MessageKind::Question,
            words: b"question?".as_slice().into(),
        },
    }));
    let Some((_, _, core::CallPart::Sent { message: question })) = world.answers.last() else {
        panic!("question: {:?}", world.answers)
    };
    let question = *question;
    erasure_request(
        &mut world,
        64,
        people::Ask::AnswerQuestion { project: 1, task, question, words: b"answer".as_slice().into() },
    );
    assert!(matches!(
        world.people_answers.last(),
        Some(people::Reply::Outcome(people::Outcome::QuestionAnswered { .. }))
    ));
    world.send(root::Event::Core(core::Event::NamedAction {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(9102)),
        key: core::CallKey { task, attempt: parent_attempt, completion: 2, position: 0 },
        action: core::NamedAction::Message {
            target: delegate,
            kind: tasks::MessageKind::Answer { question },
            words: b"answer".as_slice().into(),
        },
    }));
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"delegated".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    assert!(world.store.rows.contains_key(&ended(delegate)));
    let revision = jig_core_world::fixtures::live_row(&world, task).revisions.spec;
    erasure_request(
        &mut world,
        60,
        people::Ask::Propose {
            project: 1,
            action: people::Action::Amend {
                task,
                revision,
                amendment: Box::new(people::Amendment {
                    spec: Some(people::Spec {
                        asks: false,
                        words: b"proposed change".as_slice().into(),
                        parameters: Box::new([]),
                        inputs: Box::new([]),
                    }),
                    ..jig_core_world::fixtures::person_unchanged(b"proposed")
                }),
            },
            reason: b"review this change".as_slice().into(),
        },
    );
    let Some(people::Reply::Outcome(people::Outcome::Proposed { proposal, .. })) = world.people_answers.last() else {
        panic!("person proposal: {:?}", world.people_answers)
    };
    let proposal = *proposal;
    erasure_request(
        &mut world,
        61,
        people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Party(1),
            proposal,
            decision: people::ProposalDecision::Reject { reason: b"keep current".as_slice().into() },
            running: None,
        },
    );
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::DecidedProposal(proposal)))));
    let mut member = jig_core_world::fixtures::people_member(b"person report", 10);
    member.executor = people::TaskExecutor::Person(people::PersonAddress::Role(0));
    erasure_request(&mut world, 63, people::Ask::Create { project: 1, batch: Box::new([member]) });
    let Some(people::Reply::Outcome(people::Outcome::Created { first: person_task, .. })) = world.people_answers.last()
    else {
        panic!("person task: {:?}", world.people_answers)
    };
    let person_task = *person_task;
    erasure_request(&mut world, 67, people::Ask::TakePerson { project: 1, task: person_task });
    let revision = jig_core_world::fixtures::live_row(&world, person_task).revisions.person;
    erasure_request(
        &mut world,
        65,
        people::Ask::AnswerPerson {
            project: 1,
            task: person_task,
            revision,
            result: people::PersonResult::Report { words: b"checked".as_slice().into() },
        },
    );
    assert!(world.store.rows.contains_key(&ended(person_task)), "person answer: {:?}", world.people_answers);
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: skein_lib::Token::new(7),
        task,
        attempt: parent_attempt,
        cumulative: 1,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
    });
    assert!(world.store.rows.contains_key(&ended(task)));
    let other_request = jig_core_world::fixtures::chat(&world, 2, b"other".as_slice().into());
    erasure_request(&mut world, 66, other_request);
    let Some(people::Reply::Outcome(people::Outcome::Created { first: other, .. })) = world.people_answers.last()
    else {
        panic!("other project conversation: {:?}", world.people_answers)
    };
    let other = *other;
    assert_eq!(world.assigned.as_ref().expect("other assignment").task, other);
    assert_ne!(task, other);
    world.turn(1, 1, None, b"other transcript");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"other done".as_slice().into() },
            cancel_delegates: false,
        },
        1,
    );
    (world, vec![task, delegate, person_task], proposal, other)
}

fn names(row: &root::Record, tasks: &[u64]) -> bool {
    match row {
        root::Record::Core(core::Record::Tasks(row)) => match row {
            tasks::Stored::Live(row) | tasks::Stored::Ended(row) => tasks.contains(&row.number),
            tasks::Stored::History(row) => tasks.contains(&row.task),
            tasks::Stored::Activity(row) => tasks.contains(&row.task),
            tasks::Stored::Milestone(row) => tasks.contains(&row.task),
            tasks::Stored::Stub(row) => tasks.contains(&row.task),
            tasks::Stored::Result(row) => tasks.contains(&row.task),
            tasks::Stored::Update(row) => tasks.contains(&row.task),
            tasks::Stored::EndedByParty(row) => tasks.contains(&row.task),
            tasks::Stored::EndedByProject(row) => tasks.contains(&row.task),
            tasks::Stored::EndedDelegate(row) | tasks::Stored::Delegate(row) => {
                tasks.contains(&row.task) || tasks.contains(&row.requester)
            }
            tasks::Stored::PlacedHold(row) => {
                matches!(row.owner, tasks::Owner::Task { task } if tasks.contains(&task))
                    || matches!(row.scope, tasks::Scope::Subtree { task } if tasks.contains(&task))
            }
            tasks::Stored::Ledger(_)
            | tasks::Stored::ClosedLedger(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::DecidedProposal(_) => false,
        },
        root::Record::Core(core::Record::Core(row)) => match row {
            core::CoreRecord::Terminal(row) => tasks.contains(&row.task),
            core::CoreRecord::Turn(row) => tasks.contains(&row.task),
            core::CoreRecord::Call(row) => tasks.contains(&row.key.task),
            core::CoreRecord::RunProof(row) => tasks.contains(&row.task),
            core::CoreRecord::EscalationDecision(row) => tasks.contains(&row.task),
            core::CoreRecord::ProposalDecision(row) => {
                matches!(row.proposer, people::Proposer::Task(task) if tasks.contains(&task))
            }
            core::CoreRecord::Projection(_) | core::CoreRecord::Deployment(_) | core::CoreRecord::Settings(_) => false,
        },
        root::Record::Core(core::Record::People(
            people::Stored::EndedRead { task, .. } | people::Stored::ReadConversation { task, .. },
        )) => tasks.contains(task),
        root::Record::Core(core::Record::People(_) | core::Record::Notes(_) | core::Record::Accounts(_))
        | root::Record::Connector { .. } => false,
    }
}

fn transcript(world: &mut World, task: u64) -> Vec<Box<[u8]>> {
    let attempt = world.assignments.iter().find(|(owner, _)| *owner == task).expect("task assigned").1;
    world.store.ask(jig_test_store::StoreOut::Now(core::Now::StartPreparation {
        task,
        transcript_waiter: None,
        from: Some((attempt, 1)),
    }));
    let mut turns = Vec::new();
    for answer in world.store.answers() {
        if let jig_test_store::Up::Root(root::Event::TranscriptLoaded { rows, .. }) = answer {
            turns.extend(rows.into_vec().into_iter().map(|(row, _)| row.transcript));
        }
    }
    turns
}

#[test]
fn erasing_leaves_no_row_of_the_projects_ended_tasks() {
    let (mut world, tasks, proposal, other) = rich_world();
    let kept: Vec<_> = world.store.rows.values().filter(|row| names(row, &[other])).cloned().collect();
    assert!(!kept.is_empty());
    assert!(!transcript(&mut world, tasks[0]).is_empty());
    assert!(!transcript(&mut world, other).is_empty());
    erase(&mut world);
    released(&world, PROJECT);
    for row in world.store.rows.values() {
        assert!(!names(row, &tasks), "erased task row remains: {row:?}");
        assert!(
            !matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::DecidedProposal(row))) if row.number == proposal)
        );
        assert!(
            !matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Update(tasks::Update { object: tasks::UpdateObject::Proposal { proposal: owner }, .. }))) if owner.value() == proposal)
        );
    }
    assert_eq!(world.store.rows.values().filter(|row| names(row, &[other])).cloned().collect::<Vec<_>>(), kept);
    assert!(transcript(&mut world, tasks[0]).is_empty());
    assert!(!transcript(&mut world, other).is_empty());
}

#[test]
fn an_erased_tasks_inbox_entries_and_positions_are_gone() {
    let (mut world, task) = finished(2);
    erasure_request(&mut world, 62, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
    assert!(
        world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead { task, party: 1 })))
    );
    erase(&mut world);
    for restart in [false, true] {
        if restart {
            world.restart(SEED, false);
        }
        released(&world, PROJECT);
        assert!(
            !world
                .store
                .rows
                .contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead { task, party: 1 })))
        );
        jig_core_world::reads::ask(
            &mut world,
            500,
            reads::Read::Inbox {
                group: reads::Group::Updates,
                filter: reads::Filter { project: None, goal: None },
                from: None,
            },
        );
        let reads::Answer::Page(page) = jig_core_world::reads::latest(&world) else { panic!("inbox page") };
        assert!(page.rows.is_empty());
    }
}

#[test]
fn erasing_pages_through_a_task_larger_than_one_page() {
    let (mut world, task) = finished(40);
    let before = world.commits.len();
    erase(&mut world);
    released(&world, PROJECT);
    let write_bound = 74
        + jig_core_world::world::limits().core.tasks.placed_holds
        + 2 * jig_core_world::world::limits().core.people.people;
    assert!(
        world.commits[before..].iter().all(|writes| writes.len() <= usize::try_from(write_bound).expect("write bound"))
    );
    let activity_pages = world.commits[before..].iter().filter(|writes| writes.iter().any(|write| matches!(write,
        root::Write::Erase(root::Key::Core(core::Key::Tasks(tasks::Key::Activity { task: owner, .. }))) if *owner == task
    ))).count();
    assert!(activity_pages >= 2);
    assert!(!world.store.rows.contains_key(&ended(task)));
}

#[test]
fn a_crash_during_erasing_resumes_from_the_start_and_ends_the_same() {
    let (mut uncut, task) = finished(40);
    let first = uncut.store.applied + 1;
    erase(&mut uncut);
    let last = uncut.store.applied;
    let expected: Vec<_> = uncut
        .store
        .rows
        .values()
        .filter(|row| !matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(_)))))
        .cloned()
        .collect();
    assert!(last > first);
    for number in first..=last {
        for durable in [true, false] {
            if !durable && number == first {
                continue; // An uncommitted deletion request has not started Erasing.
            }
            let cut = if durable { Cut::Durable(number) } else { Cut::Submitted(number) };
            let (mut world, _) = finished(40);
            world.cut = Some(cut);
            erase(&mut world);
            assert_eq!(world.reached_cut, Some(cut));
            world.reads.as_mut().expect("read observer").hold_loads = true;
            world.restart(SEED, false);
            let erasing = world.store.rows.get(&root::Key::Core(core::Key::People(people::Key::Deletion(1))))
                == Some(&root::Record::Core(core::Record::People(people::Stored::Deletion {
                    project: 1,
                    stage: people::Stage::Erasing,
                })));
            if erasing {
                let loads = &world.reads.as_ref().expect("read observer").loads;
                assert_eq!(loads.len(), 1);
                assert_eq!(loads[0].1.scope, reads::Scope::ProjectRows { project: 1 });
                assert_eq!(loads[0].1.after, None, "restart scans the project from its beginning");
            }
            world.reads.as_mut().expect("read observer").hold_loads = false;
            for request in world.reads.as_mut().expect("read observer").complete() {
                world.store.ask(request);
            }
            world.drain();
            released(&world, PROJECT);
            assert!(!world.store.rows.contains_key(&ended(task)));
            let rows: Vec<_> = world
                .store
                .rows
                .values()
                .filter(|row| !matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(_)))))
                .cloned()
                .collect();
            assert_eq!(rows, expected, "crash at {cut:?}");
        }
    }
}

#[test]
fn erasing_uses_its_own_load_while_the_partys_only_read_slot_is_busy() {
    let (mut world, task) = finished(2);
    world.reads.as_mut().expect("read observer").hold_loads = true;
    jig_core_world::reads::ask(&mut world, 501, reads::Read::Task { task });
    erase(&mut world);
    let loads = &world.reads.as_ref().expect("read observer").loads;
    assert_eq!(loads.len(), 2, "one party load and the independent erasure load");
    assert_ne!(loads[0].0, loads[1].0);
    assert_eq!(loads[1].1.party, 0);
    assert_eq!(loads[1].1.scope, reads::Scope::ProjectRows { project: 1 });
    let first = loads[1].0;
    world.send(root::Event::Core(core::Event::ReadLoaded { load: first, result: reads::Loaded::Busy }));
    let loads = &world.reads.as_ref().expect("read observer").loads;
    assert_eq!(loads.len(), 3);
    assert_ne!(loads[2].0, first, "retry has its own terminal identity");
    world.send(root::Event::Core(core::Event::ReadLoaded {
        load: first,
        result: reads::Loaded::Page { rows: Box::new([]), next: None },
    }));
    assert_eq!(world.reads.as_ref().expect("read observer").loads.len(), 3, "stale load cannot advance the scan");
    world.reads.as_mut().expect("read observer").hold_loads = false;
    for request in world.reads.as_mut().expect("read observer").complete() {
        world.store.ask(request);
    }
    world.drain();
    released(&world, PROJECT);
    assert!(!world.store.rows.contains_key(&ended(task)));
}

#[test]
fn erasing_is_independent_of_the_authenticated_read_byte_budget() {
    let (mut world, task) = finished(2);
    world.seed(Box::new([root::Record::Core(core::Record::People(people::Stored::Deletion {
        project: 1,
        stage: people::Stage::Erasing,
    }))]));
    let (configuration, mut limits) = jig_core_world::effects::fixture(SEED, false);
    limits.core.load_bytes = 1;
    world.restart_with_limits(configuration, limits);
    released(&world, PROJECT);
    assert!(!world.store.rows.contains_key(&ended(task)));
}

#[test]
fn erasing_serializes_projects_in_project_order() {
    let (mut world, tasks, _, other) = rich_world();
    world.seed(Box::new([2, 1].map(|project| {
        root::Record::Core(core::Record::People(people::Stored::Deletion { project, stage: people::Stage::Erasing }))
    })));
    world.reads.as_mut().expect("read observer").hold_loads = true;
    let before = world.commits.len();
    world.restart_with(erasure_configuration().0);
    let loads = &world.reads.as_ref().expect("read observer").loads;
    assert_eq!(loads.len(), 1, "one erasing project at a time");
    assert_eq!(loads[0].1.scope, reads::Scope::ProjectRows { project: 1 });
    world.reads.as_mut().expect("read observer").hold_loads = false;
    for request in world.reads.as_mut().expect("read observer").complete() {
        world.store.ask(request);
    }
    world.drain();
    released(&world, PROJECT);
    released(&world, OTHER);
    let dropped: Vec<_> = world.commits[before..]
        .iter()
        .flatten()
        .filter_map(|write| match write {
            root::Write::DropTranscript { task } => Some(*task),
            root::Write::Save(_)
            | root::Write::Erase(_)
            | root::Write::CopySettingsRevision(_)
            | root::Write::Wires => None,
        })
        .collect();
    assert_eq!(dropped.len(), tasks.len() + 1);
    assert_eq!(dropped.last(), Some(&other));
    assert!(world.store.rows.values().all(|row| !names(row, &tasks) && !names(row, &[other])));
}

// A page plus entries in every project scope exercises the notes continuation.
const RELEASE_NOTES: u64 = 40;
const SPEND: u64 = 7;
const ACCOUNT: u32 = 0;
const OWNER: u64 = 1;
const REUSED_NAME: &str = "Released";

fn release_configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_core_world::effects::fixture(SEED, false);
    limits.core.people.requests = 32;
    config.core.grants = Box::new([people::ActGrant {
        connector: 1,
        may: Box::new([people::ActKind::RemoveProject, people::ActKind::Adopt]),
    }]);
    let seed = &config.core.seed_projects[0].policy;
    config.core.defaults.policies = skein_lib::Map::with_capacity(1);
    config
        .core
        .defaults
        .policies
        .insert(
            1,
            core::ConfiguredPolicy {
                policy: seed.policy.clone(),
                permissions: Box::new([]),
                templates: seed.templates.clone(),
            },
        )
        .expect("configured replacement policy");
    jig_core_world::peers::journal(&mut limits);
    (config, limits)
}

fn note_rows(goal: u64) -> Vec<root::Record> {
    use jig_domain_notes as notes;
    let mut rows = Vec::new();
    for name in 1..=RELEASE_NOTES {
        let scope = match name % 3 {
            0 => notes::Scope::Project { project: PROJECT },
            1 => notes::Scope::Goal { project: PROJECT, goal },
            _ => notes::Scope::Resources {
                project: PROJECT,
                connector: 1,
                pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
            },
        };
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Entry(notes::Entry {
            name,
            scope: scope.clone(),
            description: b"guidance".as_slice().into(),
            body: b"retained body".as_slice().into(),
            references: List::with_capacity(0),
            author: notes::Author::Party { party: OWNER },
            revision: 1,
            changed: Wall::EPOCH,
        }))));
        rows.push(root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
            name,
            scope,
            description: b"guidance".as_slice().into(),
            revision: 1,
        }))));
    }
    rows
}

fn release_world() -> World {
    use jig_domain_accounts as accounts;
    let (config, limits) = release_configuration();
    let mut world = World::configured_with_startup(SEED, config, limits, |domain| {
        domain.core_mut().tasks.start_restore();
    });
    let task = world.assigned.as_ref().expect("conversation").task;
    let hold = core::fresh(&mut world.domain.core_mut().counters, core::Family::Message).expect("placed hold number");
    world.turn(1, SPEND, None, b"settled spend");
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"finished".as_slice().into() },
            cancel_delegates: false,
        },
        SPEND,
    );
    let mut rows = note_rows(task);
    let mut account =
        world.store.rows[&root::Key::Core(core::Key::Accounts(accounts::Key::Account { number: ACCOUNT }))].clone();
    let root::Record::Core(core::Record::Accounts(accounts::Stored::Account { uses, .. })) = &mut account else {
        panic!("fixture account");
    };
    *uses = Some(Box::new([accounts::Use::Project(PROJECT), accounts::Use::Party(OWNER)]));
    rows.push(account);
    rows.push(root::Record::Core(core::Record::Tasks(tasks::Stored::PlacedHold(Box::new(tasks::PlacedHold {
        number: hold,
        project: PROJECT,
        owner: tasks::Owner::Party { person: OWNER },
        mode: tasks::Mode::Stop,
        scope: tasks::Scope::Project,
        reason: b"project hold".as_slice().into(),
        placed_at: 0,
        revision: 1,
        exempt: Box::new([]),
        lifted: false,
    })))));
    world.seed(rows.into_boxed_slice());
    world.restart_with(release_configuration().0);
    request(&mut world, 68, people::Ask::Rename { project: PROJECT, name: REUSED_NAME.into() });
    act(
        &mut world,
        69,
        people::Act::Adopt {
            project: PROJECT,
            adoption: people::Adoption {
                resource: people::ResourceName {
                    connector: 1,
                    path: jig_test_connector_world::path(1, 1).segments().into(),
                },
                role: people::ResourceRole::Owned,
                options: Box::new([]),
            },
        },
    );
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    world
}

fn project_ledger(funder: tasks::Funder, project: u32) -> bool {
    match funder {
        tasks::Funder::Pool { project: owner, .. }
        | tasks::Funder::Period { project: owner, .. }
        | tasks::Funder::Recurring { project: owner, .. } => owner == project,
        tasks::Funder::Task(_) => false,
    }
}

fn own_rows_gone(world: &World) {
    use jig_domain_notes as notes;
    for row in world.store.rows.values() {
        match row {
            root::Record::Core(core::Record::People(
                people::Stored::Policy { project, .. }
                | people::Stored::Roles { project, .. }
                | people::Stored::Deletion { project, .. },
            )) => assert_ne!(*project, PROJECT),
            root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row))) => {
                assert!(!project_ledger(row.funder, PROJECT));
            }
            root::Record::Core(core::Record::Tasks(tasks::Stored::PlacedHold(row))) => assert_ne!(row.project, PROJECT),
            root::Record::Core(core::Record::Notes(row)) => {
                let scope = match row {
                    notes::Record::Entry(row) => &row.scope,
                    notes::Record::Line(row) => &row.scope,
                };
                assert!(!matches!(scope, notes::Scope::Project { project } | notes::Scope::Goal { project, .. }
                    | notes::Scope::Resources { project, .. } if *project == PROJECT));
            }
            root::Record::Connector { record, .. } => assert!(!matches!(
                record,
                jig_test_connector::Record::Adoption { project: PROJECT, .. }
                    | jig_test_connector::Record::Task { project: PROJECT, .. }
            )),
            root::Record::Core(_) => {}
        }
    }
}

#[test]
fn releasing_erases_the_projects_own_rows_and_frees_its_name() {
    use jig_domain_accounts as accounts;
    let mut world = release_world();
    let ledgers: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Ledger(row)))
                if project_ledger(row.funder, PROJECT) =>
            {
                Some(*row)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect();
    assert!(
        ledgers.iter().any(|row| matches!(row.funder, tasks::Funder::Pool { .. }) && row.numbers.spent_below == SPEND)
    );
    assert!(ledgers.iter().any(|row| matches!(row.funder, tasks::Funder::Period { .. })));
    assert!(
        world.store.rows.values().any(|row| matches!(
            row,
            root::Record::Connector { record: jig_test_connector::Record::Adoption { project: PROJECT, .. }, .. }
        )),
        "adoption crossed the test connector: {:?}",
        world.act_answers
    );
    let before = world.commits.len();
    remove(&mut world, KEY, PROJECT);
    own_rows_gone(&world);
    let note_pages = world.commits[before..]
        .iter()
        .filter(|writes| {
            writes.iter().any(|write| matches!(write, root::Write::Erase(root::Key::Core(core::Key::Notes(_)))))
        })
        .count();
    assert!(note_pages >= 2, "notes exceed a single release page");
    for mut ledger in ledgers {
        ledger.closed = true;
        assert_eq!(
            world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::ClosedLedger(ledger.funder)))),
            Some(&root::Record::Core(core::Record::Tasks(tasks::Stored::ClosedLedger(ledger))))
        );
    }
    assert_eq!(
        world.domain.accounts().account(ACCOUNT).expect("account retained").uses(),
        Some([accounts::Use::Party(OWNER)].as_slice())
    );
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
    })));
    request(&mut world, 70, people::Ask::MakeProject { name: REUSED_NAME.into(), policy: 1 });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::ProjectMade { project: OTHER }))
    );
}

fn core_outputs(world: &mut World, event: core::Event) -> Vec<core::Request> {
    let (now, wall) = world.clocks();
    let env = skein_lib::Env { now, wall, limits: release_configuration().1.core };
    let core::Requests::Out(mut out) = core::step(world.domain.core_mut(), &env, event);
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    requests
}

fn release_load(world: &World) -> Token {
    let loads = &world.reads.as_ref().expect("release reads").loads;
    assert_eq!(loads.len(), 1, "only the release scan is waiting");
    assert_eq!(loads[0].1.scope, reads::Scope::Notes { project: PROJECT });
    assert_eq!(loads[0].1.after, None);
    loads[0].0
}

fn complete_release_at_core(world: &mut World) -> Vec<core::Request> {
    let load = release_load(world);
    core_outputs(
        world,
        core::Event::ReadLoaded { load, result: reads::Loaded::Page { rows: Box::new([]), next: None } },
    )
}

fn release_asks(requests: &[core::Request]) -> Vec<u16> {
    requests
        .iter()
        .filter_map(|request| match request {
            core::Request::Ask { connector, ask: core::Ask::ReleaseProject { project: PROJECT } } => Some(*connector),
            core::Request::ProjectMade { .. }
            | core::Request::Write(_)
            | core::Request::Ask { .. }
            | core::Request::Held(_)
            | core::Request::Now(_)
            | core::Request::Decided => None,
        })
        .collect()
}

fn releasing_world() -> World {
    let (config, limits) = release_configuration();
    let mut world = World::configured_with(
        SEED,
        false,
        config,
        limits,
        people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } },
    );
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    world.reads.as_mut().expect("release reads").hold_loads = true;
    world.send(root::Event::Core(core::Event::People(people::Event::Deletion {
        project: PROJECT,
        stage: Some(people::Stage::Releasing),
    })));
    world
}

fn answer_release(world: &mut World, project: u32, connector: u16) -> Vec<core::Request> {
    core_outputs(world, core::Event::ProjectReleased { project, connector })
}

fn unsettled_projection_world() -> (World, u64) {
    let (config, limits) = release_configuration();
    let mut world = World::configured(SEED, false, config, limits);
    let goal = world.assigned.as_ref().expect("conversation assigned").task;
    let revision = live_row(&world, goal).revisions.spec;
    request(
        &mut world,
        60,
        people::Ask::Amend {
            project: PROJECT,
            task: goal,
            revision,
            release: None,
            amendment: people::Amendment {
                tracked: Some(people::Tracking::Tracked { priority: 1 }),
                ..jig_core_world::fixtures::person_unchanged(b"track the closing projection")
            },
        },
    );
    world.hold_projection_settlements = true;
    jig_core_world::fixtures::finish_chat(&mut world, 0, b"goal ended before deletion");
    world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    world.reads.as_mut().expect("release reads").hold_loads = true;
    remove(&mut world, KEY, PROJECT);
    while stage(&world, PROJECT) != people::Stage::Releasing {
        jig_core_world::reads::complete(&mut world);
    }
    (world, goal)
}

#[test]
fn releasing_forgets_the_projects_projections() {
    for restart in [false, true] {
        let (mut world, goal) = unsettled_projection_world();
        if restart {
            world.restart_with(release_configuration().0);
        }
        let projection_keys: Vec<_> = world
            .store
            .rows
            .keys()
            .filter(|key| matches!(key, root::Key::Core(core::Key::Core(core::CoreKey::Projection(_)))))
            .cloned()
            .collect();
        assert!(!projection_keys.is_empty(), "closing feed retained with its unsettled cohort");
        let out = complete_release_at_core(&mut world);
        let forgotten: Vec<_> = out
            .iter()
            .filter_map(|request| match request {
                core::Request::Ask { connector, ask: core::Ask::ForgetProjection { goal: number } }
                    if *number == goal =>
                {
                    Some(*connector)
                }
                core::Request::ProjectMade { .. }
                | core::Request::Write(_)
                | core::Request::Ask { .. }
                | core::Request::Held(_)
                | core::Request::Now(_)
                | core::Request::Decided => None,
            })
            .collect();
        assert_eq!(forgotten, [1, 2], "every connector forgets the deleted goal, restart={restart}");
        for key in projection_keys {
            let root::Key::Core(erased) = &key else { panic!("core projection key") };
            assert!(
                out.iter().any(|request| matches!(request,
                    core::Request::Write(core::Write::Erase(found)) if found == erased
                )),
                "release erases {key:?}"
            );
            let _row = world.erase(key);
        }
        assert!(
            !world
                .store
                .rows
                .keys()
                .any(|key| matches!(key, root::Key::Core(core::Key::Core(core::CoreKey::Projection(_)))))
        );
        for connector in [1, 2] {
            let out = core_outputs(&mut world, core::Event::ProjectionSettled { goal, connector });
            assert!(
                out.iter().all(|request| matches!(request, core::Request::Decided)),
                "late settlement changes nothing"
            );
        }
    }
}

#[test]
fn the_row_stays_until_every_connector_answers() {
    let mut world = releasing_world();
    assert_eq!(release_asks(&complete_release_at_core(&mut world)), [1, 2]);
    for (project, connector) in [(OTHER, 1), (PROJECT, 3), (PROJECT, 1), (PROJECT, 1)] {
        let out = answer_release(&mut world, project, connector);
        assert!(!out.iter().any(|request| matches!(
            request,
            core::Request::Write(core::Write::Erase(core::Key::People(people::Key::Deletion(_))))
        )));
        assert!(world.domain.core_mut().deleting(PROJECT));
    }
    let out = answer_release(&mut world, PROJECT, 2);
    assert!(out.iter().any(|request| matches!(
        request,
        core::Request::Write(core::Write::Erase(core::Key::People(people::Key::Deletion(PROJECT))))
    )));
    assert!(!world.domain.core_mut().deleting(PROJECT));
    assert_eq!(answer_release(&mut world, PROJECT, 2).len(), 1, "duplicate answer decides no writes");
}

#[test]
fn a_start_during_releasing_asks_every_connector_again() {
    let mut world = releasing_world();
    assert_eq!(release_asks(&complete_release_at_core(&mut world)), [1, 2]);
    let _out = answer_release(&mut world, PROJECT, 1);
    world.restart_with(release_configuration().0);
    assert_eq!(release_asks(&complete_release_at_core(&mut world)), [1, 2]);
    // Model the durable cleanup decision with only its deletion row retained.
    for key in [core::Key::People(people::Key::Policy(PROJECT)), core::Key::People(people::Key::Roles(PROJECT))] {
        let _row = world.erase(root::Key::Core(key));
    }
    world.restart_with(release_configuration().0);
    assert_eq!(release_asks(&complete_release_at_core(&mut world)), [1, 2]);
}

#[test]
fn a_start_after_a_deletion_restores_its_retained_answers() {
    let mut world = release_world();
    remove(&mut world, KEY, PROJECT);
    let retained: Vec<_> = world
        .store
        .rows
        .values()
        .filter(|row| {
            matches!(
                row,
                root::Record::Core(core::Record::People(
                    people::Stored::Acted { outcome: people::Outcome::ProjectRemoved { project: PROJECT }, .. }
                        | people::Stored::Answer { outcome: people::Outcome::Renamed { project: PROJECT }, .. }
                ))
            )
        })
        .cloned()
        .collect();
    assert_eq!(retained.len(), 2, "the act and owner's rename answer outlive the project");
    world.restart_with(release_configuration().0);
    assert!(world.domain.ready());
    for row in retained {
        assert_eq!(world.store.rows.get(&row.key()), Some(&row));
    }
    remove(&mut world, KEY, PROJECT);
    assert_eq!(answer(&world), &people::Outcome::ProjectRemoved { project: PROJECT });
    request(&mut world, 68, people::Ask::Rename { project: PROJECT, name: REUSED_NAME.into() });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Renamed { project: PROJECT }))
    );
}

fn deletion_rows(world: &World) -> Vec<root::Record> {
    world
        .store
        .rows
        .values()
        .filter(|row| !matches!(row, root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(_)))))
        .cloned()
        .collect()
}

#[test]
fn a_crash_after_each_commit_of_a_deletion_ends_the_same() {
    let mut uncut = release_world();
    let first = uncut.store.applied + 1;
    remove(&mut uncut, KEY, PROJECT);
    let last = uncut.store.applied;
    let expected = deletion_rows(&uncut);
    assert!(last > first);
    for number in first..=last {
        for durable in [true, false] {
            if !durable && number == first {
                continue;
            }
            let cut = if durable { Cut::Durable(number) } else { Cut::Submitted(number) };
            let mut world = release_world();
            world.cut = Some(cut);
            remove(&mut world, KEY, PROJECT);
            assert_eq!(world.reached_cut, Some(cut));
            world.restart_with(release_configuration().0);
            assert!(world.domain.ready());
            own_rows_gone(&world);
            assert_eq!(deletion_rows(&world), expected, "crash at {cut:?}");
        }
    }
}

#[test]
fn releasing_without_connectors_erases_the_row_in_its_cleanup_decision() {
    let mut world = releasing_world();
    world.domain.core_mut().connectors = Box::new([]);
    let out = complete_release_at_core(&mut world);
    assert!(release_asks(&out).is_empty());
    for key in [people::Key::Policy(PROJECT), people::Key::Deletion(PROJECT)] {
        assert!(out.iter().any(|request| matches!(request,
            core::Request::Write(core::Write::Erase(core::Key::People(erased))) if *erased == key)));
    }
    assert!(!world.domain.core_mut().deleting(PROJECT));
}

#[test]
fn releasing_retries_busy_loads_and_ignores_their_stale_answers() {
    let mut world = releasing_world();
    let first = release_load(&world);
    world.send(root::Event::Core(core::Event::ReadLoaded { load: first, result: reads::Loaded::Busy }));
    let loads = &world.reads.as_ref().expect("release reads").loads;
    assert_eq!(loads.len(), 2, "the busy load and its retry");
    let retry = loads[1].0;
    assert_ne!(retry, first);
    world.send(root::Event::Core(core::Event::ReadLoaded {
        load: first,
        result: reads::Loaded::Page { rows: Box::new([]), next: None },
    }));
    assert_eq!(world.reads.as_ref().expect("release reads").loads.len(), 2);
    assert_eq!(stage(&world, PROJECT), people::Stage::Releasing);
    world.send(root::Event::Core(core::Event::ReadLoaded {
        load: retry,
        result: reads::Loaded::Page { rows: Box::new([]), next: None },
    }));
    released(&world, PROJECT);
}
