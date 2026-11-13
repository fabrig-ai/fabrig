//! A run reads as its person under the asking family (`jig/docs/design/domain/engine/executors.md`, 3).
use crate::settings_support::{self as support, ask, run_configuration, settings_run_with_limits};
use jig_core_world::fixtures;
use jig_domain_authority as authority;
use jig_domain_core::{self as core, reads};
use jig_domain_people as people;

#[test]
fn a_run_reads_as_its_person_only_with_the_asking_family() {
    let asking = core::ToolFamilies::JIG.ask.0;
    for asking_family in [true, false] {
        let (mut world, signed, task) = settings_run_with_limits(true, None, run_configuration().1);
        if asking_family {
            let mut granted = jig_test_profile::fixtures::people_member(b"Read as the person", 40).authority;
            granted.tools = 2047 | asking;
            let amendment = people::Amendment {
                authority: Some(granted),
                ..jig_test_profile::fixtures::person_unchanged(b"Read as the person")
            };
            let revision = fixtures::live_row(&world, task).revisions.spec;
            let reply = ask(
                &mut world,
                signed,
                102,
                people::Ask::Amend { project: 1, task, revision, amendment, release: None },
            );
            assert!(matches!(reply, people::Reply::Outcome(people::Outcome::Amended { .. })), "{reply:?}");
        }
        let row = fixtures::live_row(&world, task);
        assert_eq!(row.requester, jig_domain_tasks::Party::Person(crate::settings_support::OWNER));
        assert_eq!(row.authority.tools.0 & asking != 0, asking_family);
        let project = row.project;
        let rows = world.store.rows.clone();
        let applied = world.store.applied;
        let mut first = None;
        for _ in 0..2 {
            let token = world.person_read(reads::Read::Project { project });
            let key = if asking_family {
                let (to, key, answer) = world.person_answers.pop().expect("person read answer");
                assert_eq!(to, token);
                assert_eq!(answer, support::read(&mut world, signed, reads::Read::Project { project }));
                assert_eq!(world.answers.len(), 0);
                key
            } else {
                let (to, key, part) = world.answers.pop().expect("asking family refusal");
                assert_eq!(to, token);
                assert_eq!(
                    part,
                    core::CallPart::ToolDenied {
                        answer: authority::Answer::Propose,
                        findings: Box::new([authority::Finding::Tool]),
                    }
                );
                assert_eq!(world.person_answers.len(), 0);
                key
            };
            assert_eq!(key.task, task);
            if let Some(first) = first {
                assert_eq!(key, first, "the same call name is answered again");
            } else {
                first = Some(key);
            }
            assert_eq!(world.store.rows, rows);
            assert_eq!(world.store.applied, applied, "reads commit nothing");
        }
        world.broken();
    }
}

use jig_core_world::world::{Boot, Setup, World};
use jig_domain_core::documents;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, List, ReplyTo, Token};

const PROJECT: u32 = fixtures::PROJECT;
const OTHER_PROJECT: u32 = 2;
const PERSON: u64 = support::OWNER;
const CONNECTOR: u16 = fixtures::CONNECTOR;
const TURN: u32 = 1;

struct ProjectTasks {
    project: u32,
    live: u64,
    ended: u64,
    attempt: u64,
    waiting: u64,
}

fn create(world: &mut World, signed: u64, project: u32, key: u8, person: bool) -> u64 {
    let mut goal = fixtures::chat(world, 1, b"Work to explain to the person".as_slice().into());
    let people::Ask::Create { project: owner, batch } = &mut goal else { panic!("goal") };
    *owner = project;
    let member = batch.first_mut().expect("one goal");
    member.flags.conversation = false;
    member.flags.listed = true;
    member.tracked = Some(1);
    member.budget = people::Budget::Named;
    member.authority.spend = 20;
    member.authority.tools = 2047 | core::ToolFamilies::JIG.ask.0;
    if person {
        member.executor = people::TaskExecutor::Person(people::PersonAddress::Person(PERSON));
        member.authority.spend = 0;
    }
    let people::Reply::Outcome(people::Outcome::Created { first, .. }) = ask(world, signed, key, goal) else {
        panic!("goal admitted")
    };
    first
}

fn write_notes(world: &mut World, project: u32, task: u64) {
    for (completion, scope) in [(1, notes::Scope::Project { project }), (2, notes::Scope::Goal { project, goal: task })]
    {
        let key = world.call_key(completion);
        world.send(root::Event::Core(core::Event::NamedAction {
            to: ReplyTo::new(Token::new(u64::from(completion))),
            key,
            action: core::NamedAction::Note {
                entry: notes::New {
                    name: 0,
                    scope,
                    description: b"Plan".as_slice().into(),
                    body: b"Explain this project's work".as_slice().into(),
                    references: List::with_capacity(0),
                    author: notes::Author::Task { task, attempt: key.attempt },
                },
                recalled: None,
            },
        }));
    }
}

fn two_projects() -> (World, u64, ProjectTasks, ProjectTasks) {
    let (mut config, mut limits) = run_configuration();
    // A role executor supplies work waiting on the same person in each project.
    let kinds: Box<[authority::Executor]> = Box::new([authority::Executor::Charter(1), authority::Executor::Role(0)]);
    config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    let policy = &mut config.core.seed_projects.first_mut().expect("project").policy;
    policy.policy.ceiling.delegation.kinds.clone_from(&kinds);
    for role in &mut policy.policy.roles {
        role.authority.delegation.kinds.clone_from(&kinds);
    }
    let policy = policy.clone();
    config.core.seed_projects = Box::new([
        core::SeedProject { number: PROJECT, name: "Run's project".into(), policy: policy.clone() },
        core::SeedProject { number: OTHER_PROJECT, name: "Person's other project".into(), policy },
    ]);
    limits.core.authority.projects = 2;
    limits.core.people.projects = 3; // The two projects and deployment roles.
    limits.core.load_slots = 1; // The owner check and its page share one admission.
    jig_core_world::peers::journal(&mut limits);
    let mut world = World::new(Setup { configuration: Some(config), limits, boot: Boot::Empty, ..Setup::default() });
    world.client_peer = Some(jig_core_world::client_peer::ClientPeer::default());
    let signed = support::sign_in(&mut world);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: OTHER_PROJECT,
        holdings: Box::new([people::Holding { person: PERSON, role: people::Role::Owner }]),
    })));
    for project in [PROJECT, OTHER_PROJECT] {
        world.send(root::Event::Core(core::Event::Period { project, period: 1, budget: 100, next: None }));
    }
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: jig_domain_link::Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    let mut ended = Vec::new();
    for (project, key) in [(PROJECT, 110), (OTHER_PROJECT, 111)] {
        let task = create(&mut world, signed, project, key, false);
        let attempt = world.attempt();
        world.turn(TURN, 1, None, b"A retained turn explaining this project");
        write_notes(&mut world, project, task);
        fixtures::finish_chat(&mut world, 1, b"Work completed");
        ended.push((task, attempt));
    }
    let live = create(&mut world, signed, PROJECT, 112, false);
    world.turn(TURN, 1, None, b"The asking run's committed turn");
    let other = create(&mut world, signed, OTHER_PROJECT, 113, false);
    let waiting = create(&mut world, signed, PROJECT, 114, true);
    let other_waiting = create(&mut world, signed, OTHER_PROJECT, 115, true);
    let (ended_task, attempt) = ended[0];
    let (other_ended, other_attempt) = ended[1];
    (
        world,
        signed,
        ProjectTasks { project: PROJECT, live, ended: ended_task, attempt, waiting },
        ProjectTasks {
            project: OTHER_PROJECT,
            live: other,
            ended: other_ended,
            attempt: other_attempt,
            waiting: other_waiting,
        },
    )
}

fn project_reads(project: &ProjectTasks) -> Vec<reads::Read> {
    let ProjectTasks { project, live, ended, attempt, waiting } = *project;
    vec![
        reads::Read::Project { project },
        reads::Read::Spend { project, from: None },
        reads::Read::Tasks {
            query: documents::TaskQuery {
                project: Some(project),
                requester: None,
                root: None,
                parent: None,
                label: None,
                phase: None,
                executor: None,
                tracked: None,
                order: documents::QueryOrder::Number,
            },
            live: None,
            ended: None,
        },
        reads::Read::Notes { project, from: None },
        reads::Read::ScopedNotes { scope: notes::Scope::Project { project }, description: None, from: None },
        reads::Read::ScopedNotes { scope: notes::Scope::Goal { project, goal: ended }, description: None, from: None },
        reads::Read::Inbox {
            group: reads::Group::Needs,
            filter: reads::Filter { project: Some(project), goal: None },
            from: None,
        },
        reads::Read::Task { task: live },
        reads::Read::Task { task: ended },
        reads::Read::TaskText { task: live, field: reads::TaskTextField::Specification, from: 0 },
        reads::Read::TaskText { task: ended, field: reads::TaskTextField::Result, from: 0 },
        reads::Read::Activity { task: ended, before: None },
        reads::Read::ActivityBeside { task: ended, position: 1, side: reads::Side::Later },
        reads::Read::Turn { task: ended, attempt, turn: TURN },
        reads::Read::EndedDelegates { task: ended, from: None },
        reads::Read::History { task: ended, subtree: true, from: None },
        reads::Read::Result { task: ended },
        reads::Read::PersonTask { task: waiting },
    ]
}

// A run's answer can join its transcript, so even its person's own turns are marked withheld.
fn withheld_turns(mut answer: reads::Answer) -> reads::Answer {
    if let reads::Answer::Ended(ended) = &mut answer
        && let reads::EndedValue::Turn(record) = &ended.value
    {
        ended.value = reads::EndedValue::WithheldTurn(record.clone());
    }
    if let reads::Answer::Page(page) = &mut answer {
        for row in page.rows.iter_mut() {
            if let reads::PageRow::Activity { turn: Some(turn), .. } = row
                && let reads::PageTurn::Carried(record) = turn
            {
                *turn = reads::PageTurn::Withheld(record.clone());
            }
        }
    }
    answer
}

#[test]
fn a_run_reads_its_project_as_its_person_and_no_other() {
    let (mut world, signed, own, other) = two_projects();
    let mut rows: Vec<_> = project_reads(&own).into_iter().map(|read| (read, None)).collect();
    rows.extend(project_reads(&other).into_iter().map(|read| (read, Some(reads::Answer::None(reads::Why::Unknown)))));
    let query = documents::TaskQuery {
        project: None,
        requester: Some(tasks::Party::Person(PERSON)),
        root: None,
        parent: None,
        label: None,
        phase: None,
        executor: None,
        tracked: None,
        order: documents::QueryOrder::Number,
    };
    rows.extend(
        [
            reads::Read::Tasks { query, live: None, ended: None },
            reads::Read::Inbox {
                group: reads::Group::Needs,
                filter: reads::Filter { project: None, goal: None },
                from: None,
            },
            reads::Read::Frame,
            reads::Read::Deployment,
            reads::Read::Settings,
            reads::Read::Spend { project: people::DEPLOYMENT, from: None },
            reads::Read::Proposal { proposer: people::Proposer::Party(PERSON), proposal: 1 },
        ]
        .into_iter()
        .map(|read| (read, Some(reads::Answer::None(reads::Why::Unknown)))),
    );
    for (read, refused) in rows {
        let stored = world.store.rows.clone();
        let applied = world.store.applied;
        let expected = withheld_turns(refused.unwrap_or_else(|| support::read(&mut world, signed, read.clone())));
        let token = world.person_read(read.clone());
        let (to, key, answer) = world.person_answers.pop().expect("person read answer");
        assert_eq!(to, token, "{read:?}");
        assert_eq!(key.task, own.live, "{read:?}");
        assert_eq!(answer, expected, "{read:?}");
        assert!(world.store.rows == stored, "rows unchanged: {read:?}");
        assert_eq!(world.store.applied, applied, "no call record or read position: {read:?}");
    }
    world.broken();
}

#[test]
fn person_application_reads_reach_the_connector_as_the_person() {
    let (mut world, _, own, _) = two_projects();
    for (project, allowed) in [(PROJECT, true), (OTHER_PROJECT, false)] {
        let key = world.call_key(500);
        let token = Token::new(500);
        let name = Token::new(600);
        let window = documents::Window { rows: 1, after: None };
        let stored = world.store.rows.clone();
        let env = world.env();
        let mut out = core::step(
            world.domain.core_mut(),
            &env.with(env.limits.core),
            core::Event::NamedAction {
                to: ReplyTo::new(token),
                key,
                action: core::NamedAction::PersonRead(reads::Read::Application {
                    scope: people::ApplicationScope::Project(project),
                    connector: CONNECTOR,
                    read: name,
                    window,
                }),
            },
        );
        let request = out.pop().expect("connector read or unknown");
        if allowed {
            let core::Request::Ask { connector, ask: core::Ask::Read { reply, scope, read, party, window: received } } =
                request
            else {
                panic!("connector read: {request:?}")
            };
            assert_eq!(connector, CONNECTOR);
            assert_eq!(reply.into_token(), token);
            assert_eq!(scope, people::ApplicationScope::Project(own.project));
            assert_eq!(read, name);
            assert_eq!(party, PERSON);
            assert_eq!(received, window);
        } else {
            let core::Request::Now(now) = request else { panic!("unknown") };
            let core::Now::PersonAnswer { answer, .. } = *now else { panic!("person answer") };
            assert_eq!(*answer, reads::Answer::None(reads::Why::Unknown));
        }
        assert!(out.is_empty(), "connector answers admitted reads without a PersonAnswer");
        assert_eq!(world.store.rows, stored);
    }
    world.broken();
}

#[test]
fn stored_person_reads_recheck_the_role_and_share_load_limits() {
    let (mut world, _, own, _) = two_projects();
    let mut parties = jig_core_world::reads::Parties::new(world.store.applied);
    parties.hold_loads = true;
    world.reads = Some(parties);
    let read = reads::Read::Task { task: own.ended };
    let stored = world.store.rows.clone();
    world.person_read(read.clone());
    assert!(world.person_answers.is_empty(), "stored task waits for its load");
    world.person_read_at(501, read);
    assert_eq!(world.person_answers.pop().expect("busy").2, reads::Answer::None(reads::Why::Busy));
    assert!(world.store.rows == stored, "loading and busy change no durable row");
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([]),
    })));
    let after_role = world.store.rows.clone();
    jig_core_world::reads::complete(&mut world);
    assert_eq!(world.person_answers.pop().expect("role revoked").2, reads::Answer::None(reads::Why::Unknown));
    assert!(world.store.rows == after_role, "read completion changes no durable row");
    world.broken();
}
