//! A chat's change runs through the root's deployment-funded landing task.
use super::*;
use skein_forge::api;

fn task_row(world: &World, task: u64) -> Option<&tasks::TaskRecord> {
    world.store.rows.values().find_map(|record| match record {
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
            if row.number == task =>
        {
            Some(row.as_ref())
        }
        _ => None,
    })
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one producer lifecycle follows saved work through parking, resumption and its committed finish"
)]
fn a_parked_producers_existing_change_waits_for_its_resumed_terminal_to_commit() {
    let (mut world, chat, producer, branch) = change_world(false, 1000, None);
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else {
        panic!("producer belongs to its change")
    };
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let pushed = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"existing edit", 1)
        .expect("worker saved and delivered its edit");
    let tag = forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository);
    let saved_branch = format!("temper/{}/s{}", live_task(&world, producer.task).root, producer.task).into_bytes();
    world.external(2, raw::Op::Git(raw::Git::Create { branch: saved_branch.clone().into(), commit: pushed }));
    let timer = world.protocol.call(
        Token::new(7),
        producer.task,
        producer.attempt,
        u64::MAX,
        engine::Call {
            completion: u32::MAX,
            position: 1,
            tool: engine::Tool::Subscribe {
                kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
            },
        },
    );
    world.send(timer);
    world.until(Until::Ready);
    let parked = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        0,
        tasks::End::Parked,
        engine::Parts {
            saved: Some(Box::new([tag])),
            pushed: Box::new([forge_top::Pushed { tag, commit: translate::commit(pushed) }]),
            preparation: None,
        },
    );
    world.send(parked);
    world.until(Until::Ready);
    assert_eq!(live_task(&world, producer.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(world.merges, 0);
    let ask = people::Ask::Say {
        project: 1,
        task: producer.task,
        words: b"Finish the existing change without another edit".as_slice().into(),
    };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("person"),
        key: [95; 16],
        ask,
    }));
    for _ in 0..200 {
        world.tick();
        if world.assigned.iter().any(|run| run.task == producer.task && run.attempt > producer.attempt) {
            break;
        }
    }
    let resumed = world
        .assigned
        .iter()
        .find(|run| run.task == producer.task && run.attempt > producer.attempt)
        .expect("parked producer resumed")
        .clone();
    assert_eq!(resumed.saved.as_ref(), [tag]);
    assert!(
        matches!(&resumed.workspace.repositories[0].start, engine::ForgeStart::Saved(name) if name.as_ref() == saved_branch)
    );
    assert_eq!(resumed.workspace.repositories[0].push.as_deref(), Some(branch.as_ref()));
    for _ in 0..200 {
        world.tick();
    }
    assert_eq!(world.fake.branch(b"org/repo", &branch), Some(pushed), "no extra edit or commit");
    assert_eq!(world.merges, 0, "withheld terminal keeps the existing change unlanded");
    assert!(!matches!(task_row(&world, change).expect("change").phase, tasks::Phase::Ended(_)));
    let terminal = world.protocol.answer(
        Token::new(7),
        resumed.task,
        resumed.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: tag,
                words: b"existing edit delivered".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts {
            saved: None,
            pushed: Box::new([forge_top::Pushed { tag, commit: translate::commit(pushed) }]),
            preparation: None,
        },
    );
    world.send(terminal);
    for _ in 0..200 {
        world.tick();
        if matches!(task_row(&world, change).expect("change").phase, tasks::Phase::Ended(_)) {
            break;
        }
    }
    assert!(world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(producer.task)))));
    assert!(matches!(
        task_row(&world, change).expect("landed change").phase,
        tasks::Phase::Ended(tasks::Ending::Done(_))
    ));
    assert_eq!(world.merges, 1);
    let main = world.fake.branch(b"org/repo", b"main").expect("landed main");
    assert_eq!(
        world.fake.object(main).expect("merge object").tree.get(b"file".as_slice()).map(AsRef::as_ref),
        Some(b"existing edit".as_slice())
    );
    assert!(live_task(&world, chat.task).inbox.iter().any(|word| word.from == tasks::Party::Task(change)
        && matches!(word.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))));
}

fn hello_prepare(slots: u32) -> engine::Event {
    let mut event = protocol::hello_slots(Token::new(7), slots, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    let mut bound = protocol::merge_bound();
    bound.kind = jig_domain_link::JobKind(1);
    hello.jobs = Box::new([bound]);
    event
}
fn produced_turn_change(landing_members: Option<u32>) -> (World, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn: true, landing_members, ..StoryLimits::default() },
    );
    world.adopt();
    world.send(hello_prepare(2));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    let engine::CallAnswer::Delegated(made) =
        change_tool::change(&mut world, &chat, 93, None, b"Small fix. Keep the full detail.")
    else {
        panic!("change delegated")
    };
    let change = made[0].task;
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    let branch = producer.workspace.repositories[0].push.as_ref().expect("producer branch").clone();
    assert!(branch.starts_with(b"temper/"));
    world.external(1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushes");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"other", b"main moved", 2)
        .expect("main moves under change");
    let answer = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(answer);
    (world, change)
}

#[test]
fn a_chats_change_lands_by_turn() {
    let (mut world, change) = produced_turn_change(None);
    let mut landing = None;
    for _ in 0..600 {
        world.prepare_tick();
        for record in world.store.rows.values() {
            if let Record::Forge { row } = record
                && let forge_top::Stored::Landing(row) = row.as_ref()
            {
                landing = Some(row.landing.task);
            }
        }
        if landing.is_some_and(|landing| {
            task_row(&world, landing).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_)))
        }) && task_row(&world, change).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_)))
        {
            break;
        }
    }
    let landing = landing.unwrap_or_else(|| {
        panic!(
            "landing made: {:?}",
            world.store.rows.values().filter(|record| matches!(record, Record::Forge { .. })).collect::<Vec<_>>()
        )
    });
    let row = task_row(&world, landing).expect("landing task");
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 0, code: 3 });
    assert_eq!(row.requester, tasks::Party::Deployment { project: 1 });
    assert_eq!(row.spec.words.as_ref(), b"Land Small fix");
    assert!(row.dependencies.is_empty());
    for kind in [3, 4, 6, 9, 12, 15, 16, 17] {
        assert!(row.authority.grants.iter().any(|grant| grant.kind == kind), "landing granted kind {kind}");
    }
    assert!(!row.authority.grants.iter().any(|grant| grant.kind == 5));
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) = &row.phase else {
        panic!("landing done: {:?}; writes={:?}", row.phase, world.writes);
    };
    let report = words.clone();
    let own_branch =
        &landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("round").landing.round.branch;
    assert_eq!(world.fake.branch(b"org/repo", own_branch), None, "the landing releases its own held branch");
    let change_row = task_row(&world, change).expect("change task");
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) = &change_row.phase else {
        panic!("change done: {:?}", change_row.phase)
    };
    let change_result = words.clone();
    let raw::Answer::Commit(main) = world.external(1, raw::Op::Git(raw::Git::Fetch { want: raw::Want::Default }))
    else {
        panic!("main tip")
    };
    let main = translate::commit(main);
    let mut hex = String::new();
    for byte in &main[..20] {
        write!(&mut hex, "{byte:02x}").expect("commit hex");
    }
    assert_eq!(report.as_ref(), format!("org/repo {hex}\n").as_bytes());
    assert!(std::str::from_utf8(&change_result).expect("change result text").contains(&hex));
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::OpenPull { head, .. } if head.windows(2).any(|bytes| bytes == b"/l")))
            .count(),
        1
    );
    assert_eq!(
        world
            .writes
            .iter()
            .filter(|write| matches!(write, api::Write::Dispatch { workflow, .. } if workflow.as_ref() == b"gate.yml"))
            .count(),
        1
    );
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Status { commit, context, .. } if *commit == main && context.as_ref() == b"temper/gate/gate")).count(), 2);
    assert_eq!(world.writes.iter().filter(|write| matches!(write, api::Write::Merge { head, how: api::MergeHow::FastForwardOnly, .. } if *head == main)).count(), 1);
    assert_eq!(world.merges, 1);
}

#[test]
fn a_goal_is_kept_on_its_own_changes_landing_by_turn() {
    let world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn: true, ..StoryLimits::default() },
    );
    let (mut world, goal, producer, branch) = change_world_in(world, Change { tracked: true, ..Change::default() });
    world.send(hello_prepare(2));
    let owner = live_task(&world, goal.task);
    assert_ne!(owner.root, goal.task, "goal is delegated");
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else {
        panic!("producer belongs to change")
    };
    assert_eq!(live_task(&world, change).requester, tasks::Party::Task(goal.task));
    let subscription = world.store.rows.values().find_map(|record| match record {
        Record::Forge { row } => match row.as_ref() {
            forge_top::Stored::Subscription(row) if row.task == goal.task
                && matches!(&row.topic, forge_top::Topic::Landings { branch, .. } if branch.as_ref() == b"main") => Some(row.number),
            _ => None,
        },
        Record::Core(_) => None,
    }).expect("goal's landing subscription");
    let event =
        world.protocol.answer(Token::new(7), goal.task, goal.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    for _ in 0..30 {
        world.prepare_tick();
    }
    world.external(1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushes");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..600 {
        world.prepare_tick();
    }
    assert_eq!(world.merges, 1);
    let inbox = &live_task(&world, goal.task).inbox;
    let news: Vec<_> = inbox
        .iter()
        .filter_map(|word| match word.kind {
            tasks::MessageKind::News { subscription: number, class, .. } if number == subscription => Some(class),
            _ => None,
        })
        .collect();
    assert_eq!(news, [tasks::NewsClass::Kept], "own landing never wakes goal");
    assert!(
        inbox
            .iter()
            .any(|word| word.from == tasks::Party::Task(change) && matches!(word.kind, tasks::MessageKind::Result(_))),
        "change result reaches goal"
    );
    assert!(task_row(&world, change).is_some_and(|row| matches!(
        row.phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. }))
    )));
}

#[test]
fn a_ready_change_waits_when_landing_room_is_full() {
    let (mut world, change) = produced_turn_change(Some(0));
    for _ in 0..300 {
        world.prepare_tick();
    }
    assert!(task_row(&world, change).is_some_and(|row| matches!(row.phase, tasks::Phase::Active(_))));
    let row = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Change(row) if row.task == change => Some(row),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("change remains");
    assert_eq!(row.change.landing, None);
    assert!(matches!(row.change.state, temper_engine_domain_forge_change::State::Ready { .. }));
    assert!(world.store.rows.values().all(
        |record| !matches!(record, Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Landing(_)))
    ));
    assert_eq!(world.merges, 0);
}

const LIB: api::Repository = api::Repository { forge: 1, repository: 4 };
// Four single-member horizons cover two preparations, the turn and its partial retry.
const PAIR_TICKS: usize = 4 * 600;

fn adopt_library(world: &mut World) {
    world.adopted.clear();
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("signed in"),
        key: [94; 16],
        digest: [6; 32],
        carried: None,
        request: engine::Application::Forge(forge_top::Ask::Adopt(forge_top::Adoption {
            project: 1,
            home: false,
            forge: LIB.forge,
            host: b"forge.example".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: b"lib".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge_top::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        })),
    }));
    world.until(Until::Adopted);
    assert!(matches!(world.adopted.as_slice(), [forge_top::ApplicationOutcome::Adopted(_)]));
}

fn make_change(
    world: &mut World,
    chat: &engine::Assignment,
    name: u64,
    repository: &[u8],
) -> (u64, engine::Assignment) {
    let engine::CallAnswer::Delegated(made) = change_tool::change(world, chat, name, Some(repository), b"Small fix")
    else {
        panic!("change delegated")
    };
    let change = made[0].task;
    for _ in 0..PAIR_TICKS {
        if let Some(run) = world
            .assigned
            .iter()
            .find(|run| task_row(world, run.task).is_some_and(|row| row.requester == tasks::Party::Task(change)))
        {
            return (change, run.clone());
        }
        world.prepare_tick();
    }
    panic!("producer assigned for {change}: {:?}", task_row(world, change));
}

fn push_change(world: &mut World, producer: &engine::Assignment, repository: &[u8]) {
    let branch = producer.workspace.repositories[0].push.as_ref().expect("producer branch").clone();
    let raw::Answer::Commit(base) =
        world.external_at(repository, 1, raw::Op::Git(raw::Git::Fetch { want: raw::Want::Default }))
    else {
        panic!("main commit")
    };
    world.external_at(repository, 1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base }));
    fake::advance(&mut world.fake, &world.fake_env, repository, &branch, b"file", b"fixed", 1).expect("push change");
}

fn finish_producer(world: &mut World, producer: &engine::Assignment, repository: api::Repository) {
    let answer = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(repository.forge, repository.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(answer);
}

struct Pair {
    world: World,
    chat: engine::Assignment,
    a: u64,
    b: u64,
    producer_a: engine::Assignment,
    producer_b: engine::Assignment,
}

fn turn_chat() -> (World, engine::Assignment) {
    turn_chat_with_library_gate(None)
}

fn turn_chat_with_library_gate(library_gate_latency: Option<Duration>) -> (World, engine::Assignment) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn: true, across: true, library_gate_latency, ..StoryLimits::default() },
    );
    world.adopt();
    adopt_library(&mut world);
    world.send(hello_prepare(3));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { unreachable!() };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(250) };
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    (world, chat)
}

fn pair_producers(same_repository: bool, library_gate_latency: Option<Duration>) -> Pair {
    let (mut world, chat) = turn_chat_with_library_gate(library_gate_latency);
    let upstream_name = if same_repository { b"org/repo".as_slice() } else { b"org/lib".as_slice() };
    let (a, producer_a) = make_change(&mut world, &chat, 95, upstream_name);
    let (b, producer_b) = make_change(&mut world, &chat, 96, b"org/repo");
    Pair { world, chat, a, b, producer_a, producer_b }
}

fn add_dependency(world: &mut World, chat: &engine::Assignment, a: u64, b: u64) {
    let mut amendment = temper_engine_domain_world::fixtures::task_amendment(None, None, b"land upstream first");
    amendment.dependencies = Some(Box::new([a]));
    let revision = live_task(world, b).revisions.spec;
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        97,
        engine::Call {
            completion: 97,
            position: 1,
            tool: engine::Tool::Amend { keep_parameters: false, target: b, revision, amendment },
        },
    );
    world.send(event);
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        if live_task(world, b).waiting_on.as_ref() == [a] {
            break;
        }
    }
    assert_eq!(live_task(world, b).waiting_on.as_ref(), [a], "amend accepted: {:?}", world.answers);
}

fn pushed_pair(same_repository: bool) -> Pair {
    pushed_pair_with_library_gate(same_repository, None)
}

fn pushed_pair_with_library_gate(same_repository: bool, latency: Option<Duration>) -> Pair {
    let Pair { mut world, chat, a, b, producer_a, producer_b } = pair_producers(same_repository, latency);
    let upstream_name = if same_repository { b"org/repo".as_slice() } else { b"org/lib".as_slice() };
    push_change(&mut world, &producer_a, upstream_name);
    push_change(&mut world, &producer_b, b"org/repo");
    fake::advance(&mut world.fake, &world.fake_env, upstream_name, b"main", b"other", b"main moved", 2)
        .expect("upstream main moved");
    if !same_repository {
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"other", b"main moved", 2)
            .expect("downstream main moved");
    }
    Pair { world, chat, a, b, producer_a, producer_b }
}

fn dependent_pair(same_repository: bool) -> (World, u64, u64) {
    let Pair { mut world, chat, a, b, producer_a, producer_b } = pushed_pair(same_repository);
    add_dependency(&mut world, &chat, a, b);
    finish_producer(&mut world, &producer_a, if same_repository { forge_world::REPO } else { LIB });
    finish_producer(&mut world, &producer_b, forge_world::REPO);
    (world, a, b)
}

fn landing_rows(world: &World) -> Vec<&forge_top::LandingRow> {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Landing(row) => Some(row),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect()
}

fn ended(world: &World, task: u64) -> bool {
    task_row(world, task).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_))))
}

fn land_pair(world: &mut World, a: u64, b: u64, stale_downstream: bool) {
    let mut moved = false;
    let mut ends = Vec::new();
    let mut upstream_landed = None;
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        if stale_downstream
            && !moved
            && landing_rows(world).iter().any(|row| {
                row.landing.members.iter().any(|member| member.change == b)
                    && matches!(row.landing.phase, forge_top::LandingPhase::Turn { .. })
            })
        {
            fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"race", b"moved during proof", 2)
                .expect("downstream main moved before turn merge");
            moved = true;
        }
        for row in landing_rows(world) {
            for member in &row.landing.members {
                if member.change == a {
                    if let Some(commit) = upstream_landed {
                        assert_eq!(member.landed, Some(commit), "landed stays landed");
                    }
                    upstream_landed = member.landed;
                }
            }
        }
        for task in [a, b] {
            if ended(world, task) && !ends.contains(&task) {
                ends.push(task);
            }
        }
        if ends.len() == 2 && landing_rows(world).iter().all(|row| ended(world, row.landing.task)) {
            break;
        }
    }
    let changes: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Change(row) => {
                    Some((row.task, row.change.landing, row.change.state.clone(), row.pending, row.effect))
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect();
    assert_eq!(moved, stale_downstream, "changes={changes:?}");
    if task_row(world, b).is_some_and(|row| row.dependencies.contains(&a)) {
        assert_eq!(ends, [a, b], "jig orders dependent task results");
    }
    ends.sort_unstable();
    let mut expected = [a, b];
    expected.sort_unstable();
    assert_eq!(
        ends,
        expected,
        "changes={changes:?}; rounds={:?}; landing tasks={:?}",
        landing_rows(world)
            .iter()
            .map(|row| (
                &row.landing.phase,
                row.landing.preparation,
                &row.landing.round,
                &row.landing.members[0].gates,
                &row.entries
            ))
            .collect::<Vec<_>>(),
        landing_rows(world)
            .iter()
            .map(|row| task_row(world, row.landing.task).map(|task| &task.phase))
            .collect::<Vec<_>>()
    );
}

fn assert_pair_result(world: &mut World, a: u64, b: u64, downstream_rebases: usize) {
    let rows = landing_rows(world);
    assert_eq!(rows.len(), 2, "one sealed landing per repository");
    let candidates = [a, b].map(|change| {
        let row = rows.iter().find(|row| row.landing.members[0].change == change).expect("member's landing");
        assert_eq!(row.landing.members.len(), 1);
        row.landing.members[0].landed.expect("member landed")
    });
    for (change, repository, candidate) in
        [(a, b"org/lib".as_slice(), candidates[0]), (b, b"org/repo".as_slice(), candidates[1])]
    {
        let raw::Answer::Commit(main) =
            world.external_at(repository, 1, raw::Op::Git(raw::Git::Fetch { want: raw::Want::Default }))
        else {
            panic!("landed main")
        };
        assert_eq!(translate::commit(main), candidate);
        let mut expected = String::new();
        write!(&mut expected, "{} ", std::str::from_utf8(repository).expect("repository name")).expect("report");
        for byte in &candidate[..20] {
            write!(&mut expected, "{byte:02x}").expect("commit hex");
        }
        expected.push('\n');
        let task = landing_rows(world)
            .iter()
            .find(|row| row.landing.members[0].change == change)
            .expect("landing")
            .landing
            .task;
        let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) =
            &task_row(world, task).expect("landing task").phase
        else {
            panic!("landing done")
        };
        assert_eq!(words.as_ref(), expected.as_bytes());
    }
    for (repository, expected) in [(LIB, 1), (forge_world::REPO, downstream_rebases)] {
        assert_eq!(
            world
                .repository_writes
                .iter()
                .filter(|(repo, write)| *repo == repository
                    && matches!(write, api::Write::OpenPull { head, .. } if head.windows(2).any(|bytes| bytes == b"/l")))
                .count(),
            expected
        );
        assert_eq!(
            world
                .repository_writes
                .iter()
                .filter(|(repo, write)| *repo == repository && matches!(write, api::Write::Dispatch { .. }))
                .count(),
            expected
        );
    }
    let mut merges = world
        .repository_writes
        .iter()
        .filter_map(|(repository, write)| match write {
            api::Write::Merge { head, how: api::MergeHow::FastForwardOnly, .. } => Some((*repository, *head)),
            _ => None,
        })
        .collect::<Vec<_>>();
    merges.sort_unstable();
    let mut expected = [(LIB, candidates[0]), (forge_world::REPO, candidates[1])];
    expected.sort_unstable();
    assert_eq!(merges, expected);
}

#[test]
fn dependent_changes_in_two_repositories_are_refused_as_a_unit() {
    let (mut world, a, b) = dependent_pair(false);
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        if matches!(task_row(&world, a).expect("change").phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    assert!(landing_rows(&world).is_empty());
    let tasks::Phase::Held { source: Some(source), .. } = &task_row(&world, a).expect("held first change").phase else {
        panic!("unit refused")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("refusal account") };
    let words = std::str::from_utf8(words).expect("words");
    assert!(words.starts_with("unit of 2 changes:"));
    assert!(words.contains(&format!("C{a}")) && words.contains(&format!("C{b}")));
    assert!(words.contains("temper/docs/design/draft/merge-queue.md, 13"));
    assert!(!world.repository_writes.iter().any(|(_, write)| matches!(write, api::Write::Merge { .. })));
}

#[test]
fn a_stale_train_reprepares_without_repeating_the_other_trains_merge() {
    let Pair { mut world, a, b, producer_a, producer_b, .. } = pushed_pair(false);
    finish_producer(&mut world, &producer_a, LIB);
    finish_producer(&mut world, &producer_b, forge_world::REPO);
    land_pair(&mut world, a, b, true);
    assert_pair_result(&mut world, a, b, 2);
}

#[test]
fn dependent_changes_in_one_repository_land_in_two_landings() {
    let (mut world, a, b) = dependent_pair(true);
    land_pair(&mut world, a, b, false);
    let rows = landing_rows(&world);
    assert_eq!(rows.len(), 2);
    let mut members = rows
        .iter()
        .map(|row| row.landing.members.iter().map(|member| member.change).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    members.sort();
    assert_eq!(members, [vec![a], vec![b]]);
}

#[test]
fn a_dependency_amendment_leaves_a_proving_landing_sealed() {
    let Pair { mut world, chat, a, b, producer_a, producer_b } = pushed_pair(false);
    finish_producer(&mut world, &producer_a, LIB);
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        if world
            .repository_writes
            .iter()
            .any(|(repo, write)| *repo == LIB && matches!(write, api::Write::Dispatch { .. }))
        {
            break;
        }
    }
    let initial = landing_rows(&world)[0].landing.task;
    add_dependency(&mut world, &chat, a, b);
    assert_eq!(landing_rows(&world)[0].landing.task, initial);
    assert_eq!(landing_rows(&world)[0].landing.members.iter().map(|member| member.change).collect::<Vec<_>>(), [a]);
    assert_ne!(landing_rows(&world)[0].landing.phase, forge_top::LandingPhase::Waiting);
    finish_producer(&mut world, &producer_b, forge_world::REPO);
    land_pair(&mut world, a, b, false);
    assert_pair_result(&mut world, a, b, 1);
}

#[test]
fn a_dependency_between_two_formed_trains_does_not_keep_either_turn() {
    let Pair { mut world, chat, a, b, producer_a, producer_b } =
        pushed_pair_with_library_gate(false, Some(Duration::from_secs(120)));
    finish_producer(&mut world, &producer_a, LIB);
    finish_producer(&mut world, &producer_b, forge_world::REPO);
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        if landing_rows(&world).len() == 2 {
            break;
        }
    }
    assert_eq!(landing_rows(&world).len(), 2);
    let ids = landing_rows(&world).iter().map(|row| row.landing.task).collect::<Vec<_>>();
    assert!(landing_rows(&world).iter().all(|row| row.landing.members.len() == 1));
    add_dependency(&mut world, &chat, a, b);
    let mut turned_before_upstream = false;
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        let rows = landing_rows(&world);
        assert_eq!(rows.iter().map(|row| row.landing.task).collect::<Vec<_>>(), ids);
        assert!(rows.iter().all(|row| row.landing.members.len() == 1));
        let upstream = rows.iter().find(|row| row.landing.members[0].change == a).expect("upstream landing");
        let downstream = rows.iter().find(|row| row.landing.members[0].change == b).expect("downstream landing");
        let upstream_read_back = upstream.landing.members[0].landed.is_some();
        if !upstream_read_back
            && (matches!(downstream.landing.phase, forge_top::LandingPhase::Turn { .. })
                || downstream.landing.members[0].landed.is_some())
        {
            turned_before_upstream = true;
            assert!(
                world
                    .repository_writes
                    .iter()
                    .any(|(repo, write)| *repo == forge_world::REPO && matches!(write, api::Write::Dispatch { .. }))
            );
        }
        if ended(&world, a) && ended(&world, b) {
            break;
        }
    }
    assert!(turned_before_upstream, "each train has its own turn: {:?}", landing_rows(&world));
    assert!(ended(&world, a) && ended(&world, b));
    let merges = world
        .repository_writes
        .iter()
        .filter_map(|(repo, write)| if matches!(write, api::Write::Merge { .. }) { Some(*repo) } else { None })
        .collect::<Vec<_>>();
    assert_eq!(merges, [forge_world::REPO, LIB]);
}

#[test]
fn a_change_created_with_a_dependency_does_not_join_a_formed_landing() {
    let (mut world, chat) = turn_chat();
    let (a, producer_a) = make_change(&mut world, &chat, 95, b"org/lib");
    push_change(&mut world, &producer_a, b"org/lib");
    finish_producer(&mut world, &producer_a, LIB);
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        if !landing_rows(&world).is_empty() {
            break;
        }
    }
    let mut change = temper_engine_domain_world::fixtures::report_delegate(
        b"Small fix",
        Box::new([engine::Dependency::Existing(a)]),
    );
    change.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
    change.spec.parameters = Box::new([
        tasks::Parameter::Resource {
            name: 1,
            connector: 0,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        },
        tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
    ]);
    change.contract = tasks::Contract::Change { connector: 0, kind: 1, words: world.env.limits.tasks.result_bytes };
    change.authority = live_task(&world, chat.task).authority.clone();
    change.authority.delegation.depth = change.authority.delegation.depth.saturating_sub(1);
    change.authority.delegation.tasks = 6;
    change.authority.budget.spend = 50;
    let before = world.answers.len();
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        96,
        engine::Call { completion: 96, position: 1, tool: engine::Tool::Delegate { batch: Box::new([change]) } },
    );
    world.send(event);
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        if world.answers.len() > before {
            break;
        }
    }
    let engine::CallAnswer::Delegated(made) = &world.answers[before] else {
        panic!("change created with dependency: {:?}", world.answers[before]);
    };
    let b = made[0].task;
    for _ in 0..PAIR_TICKS {
        world.prepare_tick();
        if world.assigned.iter().any(|producer| producer.task != producer_a.task && producer.task != chat.task) {
            break;
        }
    }
    assert_eq!(landing_rows(&world).len(), 1);
    assert_eq!(landing_rows(&world)[0].landing.members.iter().map(|member| member.change).collect::<Vec<_>>(), [a]);
    assert!(!ended(&world, b), "new change is still producing, outside the sealed landing");
}

#[test]
fn the_prepare_jobs_conflict_result_ends_the_round_and_asks_for_resolution() {
    let (mut world, change) = produced_turn_change(None);
    world.manual_prepare_jobs();
    for _ in 0..600 {
        world.tick();
        if !world.jobs.is_empty() {
            break;
        }
    }
    assert_eq!(world.jobs.len(), 1);
    let job = world.jobs.remove(0);
    assert_eq!(job.kind, jig_domain_link::JobKind(1));
    let landing = job.workspace.as_ref().expect("job workspace").workstream;
    let result = format!("{{\"conflict\":{{\"member\":{change},\"with\":\"branch\",\"files\":[\"file\"]}}}}");
    let event = world.protocol.answer_job(
        Token::new(7),
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Finished { result: result.into_bytes().into(), work: host::Work { left: None, saved: None } },
    );
    world.send(event);
    for _ in 0..600 {
        world.tick();
        if task_row(&world, landing).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_)))
            && matches!(
                super::light_rules::change_row(&world, change).change.state,
                temper_engine_domain_forge_change::State::Resolving { .. }
            )
        {
            break;
        }
    }
    assert!(matches!(task_row(&world, landing).expect("landing").phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("round");
    assert!(row.landing.members[0].landed.is_none());
    assert!(
        matches!(super::light_rules::change_row(&world, change).change.state, temper_engine_domain_forge_change::State::Resolving { base, .. } if Some(base) == row.landing.members[0].base)
    );
    assert!(!world.writes.iter().any(|write| matches!(write, api::Write::Merge { .. } | api::Write::Update { .. })));
}
