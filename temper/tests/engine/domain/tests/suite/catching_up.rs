//! The engine opens independently of forge reconciliation (draft/reconciling-start.md, 7).
use super::forge_routes::{change_tool, follow_changes, goal_branch};
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people::{self as people, ApplicationScope};
use jig_domain_tasks as tasks;
use skein_fake_forge_domain as fake;
use skein_fake_forge_domain::api as raw;
use skein_forge::api::{self as api, Repository};
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Event, PartyInput};
use temper_engine_domain::{Key, Record};
use temper_engine_domain_forge::{self as forge, CatchingUp, Document};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_world::forge_routes::{Gate, StoryLimits, Until, World};
use temper_engine_forge_world::REPO;

const STEPS: u32 = 200;
const FORGES: Token = Token::new(810);
const CHANGES: Token = Token::new(811);

fn fixture() -> (World, u64, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { second_project: true, kinds: true, slots: Some(3), ..StoryLimits::default() },
    );
    world.adopt();
    world.until(Until::Ready);
    extra_repository(&mut world);
    world.send(temper_engine_domain_world::protocol::hello_slots(Token::new(7), 3, Box::new([])));
    let (mut world, goal, branch) = goal_branch::running_goal_in(world);
    assert!(goal_branch::forge_hold(&world, &branch).is_some(), "goal owns its branch");
    let engine::CallAnswer::Delegated(made) = change_tool::change_landing(
        &mut world,
        &goal,
        94,
        Some(b"org/repo"),
        b"Small fix",
        Some(engine::ChangeLanding::Goal),
    ) else {
        panic!("goal's change admitted")
    };
    let task = made[0].task;
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    let branch = world.store.change(task).expect("live change").branch;
    change_tool::deliver_change(&mut world, &producer, &branch, b"Small fix");
    for _ in 0..STEPS {
        world.tick();
        if world.store.change(task).is_some_and(|row| row.pull.is_some()) {
            break;
        }
    }
    let row = world.store.change(task).expect("saved change");
    let pull = row.pull.expect("live pull");
    assert!(!matches!(row.change.state, change::State::Landed { .. }));
    let raw::Answer::Pull(observed) = world.external(1, raw::Op::Read(raw::Read::Pull { number: pull })) else {
        panic!("fake pull")
    };
    let commit = observed.commit;
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit,
                context: b"build".as_slice().into(),
                state: raw::Check::Pending,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done
    );
    world.until(Until::Ready);
    assert!(!matches!(world.store.change(task).expect("live change").change.state, change::State::Landed { .. }));
    world.drain_cut();
    (world, task, pull)
}

fn open(world: &mut World) {
    for _ in 0..STEPS {
        world.tick();
        if world.root.ready() {
            break;
        }
    }
    assert!(world.root.ready(), "local start opens while the forge is away");
    assert_eq!(world.root.start_phase(), None);
    assert!(!world.root.caught_up(0));
}

fn watch(world: &mut World, watcher: Token, scope: ApplicationScope, subject: u64) {
    world.send(Event::Party(PartyInput::Watch {
        watcher,
        sign_in: world.signed_in.expect("signed in"),
        subject: Watched::Application { scope, connector: 0, subject: Token::new(subject) },
        window: Window { rows: 1, after: None },
    }));
}

fn states(world: &World) -> (Option<CatchingUp>, Option<CatchingUp>) {
    let forges = world
        .forges_documents
        .iter()
        .rev()
        .find_map(|(watcher, rows)| {
            (*watcher == FORGES).then(|| rows.rows.first().expect("configured forge").catching_up)
        })
        .expect("forge document delivered");
    let changes = world
        .oracle_documents
        .iter()
        .rev()
        .find_map(|(watcher, document)| {
            if *watcher == CHANGES
                && let Document::Changes(panel) = document
            {
                Some(panel.catching_up)
            } else {
                None
            }
        })
        .expect("changes document delivered");
    (forges, changes)
}

fn people_answer(world: &mut World) {
    world.signed_in = None;
    world.request_sign_in();
    world.until(Until::SignedIn);
    let before = world.web_replies.len();
    let ask = world.chat(1, b"Talk while the forge catches up");
    world.send(Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(812)),
        sign_in: world.signed_in.expect("new sign-in answered"),
        key: [112; 16],
        ask,
    }));
    for _ in 0..STEPS {
        world.tick();
        if world
            .web_replies
            .iter()
            .skip(before)
            .any(|reply| matches!(reply, people::Reply::Outcome(people::Outcome::Created { .. })))
        {
            return;
        }
    }
    panic!("chat must answer Created, not Again or Busy: {:?}", world.web_replies);
}

fn extra_repository(world: &mut World) {
    fake::repository(
        &mut world.fake,
        &world.fake_env.limits,
        raw::Setup {
            workflows: Box::new([]),
            id: 3,
            name: b"org/extra".as_slice().into(),
            default: b"main".as_slice().into(),
            tree: Box::new([]),
            labels: Box::new([]),
            checks: raw::Checks {
                cues: Box::new([]),
                contexts: Box::new([]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
            protection: None,
            hooked: false,
        },
    );
    for user in [1, 2] {
        fake::grant(&mut world.fake, b"org/extra", user, raw::Permission::Admin);
    }
    let before = world.adopted.len();
    world.send(Event::Party(PartyInput::Application {
        scope: ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(115)),
        sign_in: world.signed_in.expect("owner"),
        key: [115; 16],
        digest: [115; 32],
        carried: None,
        request: engine::Application::Forge(forge::Ask::Adopt(forge::Adoption {
            project: 1,
            home: false,
            forge: 1,
            host: b"forge.example".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: b"extra".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        })),
    }));
    for _ in 0..STEPS {
        world.tick();
        if world.adopted.len() > before {
            break;
        }
    }
    assert!(
        matches!(world.adopted.get(before), Some(forge::ApplicationOutcome::Adopted(_))),
        "adoptions {:?}; replies {:?}",
        world.adopted,
        world.web_replies
    );
    world.until(Until::Ready);
}

fn second_change(world: &mut World) -> (u64, engine::Assignment, Box<[u8]>) {
    let goal = world.assigned[0].clone();
    let engine::CallAnswer::Delegated(made) = change_tool::change(world, &goal, 116, Some(b"org/extra"), b"Other fix")
    else {
        panic!("second change admitted")
    };
    let task = made[0].task;
    for _ in 0..STEPS {
        world.tick();
        if world.assigned.iter().any(|run| matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(run.task)))),
            Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if row.requester == tasks::Party::Task(task))) { break; }
    }
    let producer = world.assigned.iter().find(|run| matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(run.task)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if row.requester == tasks::Party::Task(task))).expect("second producer").clone();
    let branch = world.store.change(task).expect("second change").branch;
    world.drain_cut();
    (task, producer, branch)
}

fn deliver_second(world: &mut World, producer: &engine::Assignment, branch: &[u8]) {
    let raw::Answer::Commit(base) =
        world.external_at(b"org/extra", 2, raw::Op::Read(raw::Read::Branch { branch: b"main".as_slice().into() }))
    else {
        panic!("extra main")
    };
    assert_eq!(
        world.external_at(b"org/extra", 2, raw::Op::Git(raw::Git::Create { branch: branch.into(), commit: base })),
        raw::Answer::Branch(raw::Created::Created)
    );
    fake::advance(&mut world.fake, &world.fake_env, b"org/extra", branch, b"file", b"other fix", 2)
        .expect("second push");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Other fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge::tag(1, 3),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

fn opened_second(world: &World) -> bool {
    world.repository_writes.iter().any(|(repository, write)| {
        *repository == Repository { forge: 1, repository: 3 } && matches!(write, api::Write::OpenPull { .. })
    })
}

fn issue(world: &mut World, goal: &engine::Assignment, name: u32, title: &[u8]) {
    let event = world.protocol.call(
        Token::new(7),
        goal.task,
        goal.attempt,
        u64::from(name),
        engine::Call {
            completion: name,
            position: 1,
            tool: engine::Tool::EffectForge {
                repository: REPO,
                resource: forge::What::Repository,
                write: Box::new(api::Write::CreateIssue { key: Box::new([]), title: title.into(), body: title.into() }),
            },
        },
    );
    world.send(event);
}

#[test]
fn failing_find_keeps_its_lane_but_not_other_repositories_from_writing() {
    let (mut world, _, _) = fixture();
    let (other, producer, branch) = second_change(&mut world);
    world.lose_writes = 1;
    world.fail_issue_finds = true;
    let goal = world.assigned[0].clone();
    issue(&mut world, &goal, 117, b"uncertain");
    for _ in 0..STEPS {
        world.tick();
        if world.store.rows.values().any(|row| matches!(row,
            Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Entry { entry, held: false }
                if matches!(&entry.effect.write, api::Write::CreateIssue { title, .. } if title.as_ref() == b"uncertain")
                    && entry.attempt.is_some() && entry.failures > 0))) { break; }
    }
    assert_eq!(world.lose_writes, 0, "uncertain write was sent and lost");
    world.drain_cut();
    world.restart_same();
    open(&mut world);
    // The surviving worker's queued Hello follows the local opening.
    world.advance_ticks(20);
    deliver_second(&mut world, &producer, &branch);
    world.signed_in = None;
    world.request_sign_in();
    world.until(Until::SignedIn);
    let ask = world.chat(1, b"Queue a write behind the uncertain one");
    let other_writer = super::forge_routes::assigned_chat(&mut world, 118, ask);
    issue(&mut world, &other_writer, 119, b"behind uncertain");
    world.advance_ticks(STEPS);
    let progress = world.root.forges().rows[0].catching_up.expect("find keeps forge catching up");
    assert_eq!(
        progress,
        CatchingUp { fresh: 0, gathers: 0, ci: 0, goal_files: 0, lost_writers: 0, goal_branches: 0, uncertain: 1 },
        "all fresh-read terms completed independently of settle"
    );
    assert!(!world.root.caught_up(0), "settle has not reported");
    assert!(world.failed_issue_finds > 1, "find failed repeatedly");
    assert!(
        !world.writes.iter().any(
            |write| matches!(write, api::Write::CreateIssue { title, .. } if title.as_ref() == b"behind uncertain")
        ),
        "same lane's write was never sent"
    );
    assert!(world.store.rows.values().any(|row| matches!(row, Record::Forge { row }
        if matches!(row.as_ref(), forge::Stored::Entry { entry, .. }
            if matches!(&entry.effect.write, api::Write::CreateIssue { title, .. } if title.as_ref() == b"behind uncertain")))), "same lane's write is queued; answers {:?}; errors {:?}", world.answers, world.call_errors);
    assert!(opened_second(&world), "other change writes: {:?}", world.store.change(other));
    assert!(world.store.change(other).expect("other change").pull.is_some(), "other write was made");
    world.fail_issue_finds = false;
    // Finish the abandoned creation's find/deadline before the cold-replay oracle snapshots it.
    world.advance_ticks(STEPS);
    assert!(world.root.caught_up(0));
    world.close();
}

#[test]
fn rate_limited_read_keeps_the_forge_catching_up_but_not_people() {
    let (mut world, task, _) = fixture();
    let (other, producer, branch) = second_change(&mut world);
    let read = api::Read::Branch { branch: world.store.change(task).expect("first change").branch };
    world.scripted_forge_reads.push((read.clone(), Err(api::Error::RateLimited { after: Duration::from_secs(1) })));
    world.scripted_forge_attempts = Some(Vec::new());
    world.restart_same();
    open(&mut world);
    people_answer(&mut world);
    // Give the surviving worker's queued Hello its local turns too.
    world.advance_ticks(20);
    deliver_second(&mut world, &producer, &branch);
    world.advance_ticks(STEPS);
    let progress = world.root.forges().rows[0].catching_up.expect("rate-limited resource still owed");
    assert!(progress.fresh >= 1);
    assert!(!world.root.caught_up(0));
    let attempts = world.scripted_forge_attempts.as_ref().expect("scripted read attempt times");
    assert!(attempts.len() > 1, "every attempt is rate-limited, not a held call");
    for pair in attempts.windows(2) {
        let [(before, asked), (after, again)] = pair else { unreachable!("two attempts") };
        assert_eq!(*asked, read);
        assert_eq!(*again, read);
        assert!(*after >= before.saturating_add(Duration::from_secs(1)), "no retry before the rate reset");
    }
    assert!(!opened_second(&world), "the shared rate limit also holds the other repository's calls");
    world.scripted_forge_reads.clear();
    world.advance_ticks(STEPS);
    assert!(opened_second(&world), "other change's queued calls go on once the rate limit lifts");
    assert!(world.store.change(other).expect("other change").pull.is_some(), "other write was made");
    assert!(world.root.caught_up(0));
    world.close();
}

#[test]
fn unavailable_forge_does_not_gate_opening_people_or_settings_and_recovers() {
    let (mut world, task, pull) = fixture();
    let before = world.store.change(task).expect("saved change").change;
    world.unavailable = true;
    world.restart_same();
    open(&mut world);
    people_answer(&mut world);
    let revision = world.root.settings_revision();
    follow_changes::start_change_setting(&mut world, None, 113);
    for _ in 0..STEPS {
        world.tick();
        if world.root.settings_revision() > revision {
            break;
        }
    }
    assert_eq!(
        world.root.settings_revision(),
        revision + 1,
        "settings answered and committed: {:?}",
        world.web_replies
    );
    watch(&mut world, FORGES, ApplicationScope::Deployment, 0);
    watch(&mut world, CHANGES, ApplicationScope::Project(1), forge::changes_subject(1));
    world.advance_ticks(STEPS);
    let progress = states(&world);
    assert_eq!(progress.0, progress.1);
    assert!(progress.0.expect("catching up").fresh > 0);
    assert!(!world.unavailable_calls.is_empty(), "fault answered calls rather than holding them");
    assert_eq!(
        world.store.change(task).expect("waiting change").change,
        before,
        "no decision from restored forge facts"
    );
    assert!(!world.root.caught_up(0));

    assert_eq!(world.external(1, raw::Op::Write(raw::Write::Close { number: pull })), raw::Answer::Done);
    let deliveries = (world.forges_documents.len(), world.oracle_documents.len());
    world.unavailable = false;
    for _ in 0..STEPS {
        for watcher in [FORGES, CHANGES] {
            world.send(Event::Party(PartyInput::Delivered { watcher }));
        }
        world.tick();
        if world.root.caught_up(0)
            && states(&world) == (None, None)
            && world.store.change(task).expect("change observed").change != before
        {
            break;
        }
    }
    assert!(world.root.caught_up(0), "both halves reported to the core");
    assert_eq!(states(&world), (None, None));
    assert!(world.forges_documents.len() > deliveries.0 && world.oracle_documents.len() > deliveries.1);
    assert_ne!(
        world.store.change(task).expect("change observed").change,
        before,
        "change decides on the fresh closed pull"
    );
    world.close();
}
