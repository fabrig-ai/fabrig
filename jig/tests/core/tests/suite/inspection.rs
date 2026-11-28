//! Inspection at the core world's quiet points (jig/docs/design/domain/testing-fakes.md, 1.1).
use jig_core_world::{
    fixtures::{configure_resources, delegate, member, resource_member},
    world::{Setup, World},
};
use jig_domain_contract as jig;
use jig_domain_core as core;
use jig_domain_core::engine;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Queue, ReplyTo, Time, Token, Wall};
use skein_world::inspect::{Inspection, Moment, Release, Wait, Wake, check_tables, check_waits};
use std::fmt::Write;

mod records;

fn inspect(world: &World, seed: u64) -> Inspection {
    let mut inspection = Inspection::new("jig", jig_inspect::KINDS);
    jig_inspect::core(world.domain.core(), &mut inspection);
    let moment = Moment { seed, at: world.clocks().0, iteration: 0 };
    assert_eq!(check_waits(&[&inspection], moment), Ok(()));
    assert_eq!(check_tables(&[&inspection], moment), Ok(()));
    assert_eq!(jig_inspect::records(world.domain.core(), moment, world.clocks().1), Ok(()));
    inspection
}

fn waiting<'a>(inspection: &'a Inspection, task: u64, kind: &str) -> &'a Wait {
    inspection
        .waits()
        .iter()
        .find(|wait| wait.waiter == format!("task {task}") && wait.kind == kind)
        .unwrap_or_else(|| panic!("task {task} has {kind}: {:?}", inspection.waits()))
}

fn record(world: &World, task: u64, kind: core::WaitKind) -> core::Wait {
    let waits = world.domain.core().waits(task);
    waits
        .own
        .iter()
        .find(|wait| wait.kind == kind)
        .cloned()
        .unwrap_or_else(|| panic!("task {task}: no {kind:?} in {waits:?}"))
}

fn park(world: &mut World, run: (u64, u64)) {
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: Token::new(7),
        task: run.0,
        attempt: run.1,
        cumulative: 0,
        end: tasks::End::Parked,
    });
}

fn key(run: (u64, u64), completion: u32) -> core::CallKey {
    core::CallKey { task: run.0, attempt: run.1, completion, position: 0 }
}

fn subscribe(world: &mut World, task: u64, kind: tasks::SubscriptionKind) {
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Subscribe {
        reads_project: false,
        reply_to: ReplyTo::new(Token::new(9901)),
        task,
        subscription: tasks::Subscription { number: 9901, kind },
    })));
}

#[test]
fn dependencies_name_the_pending_task() {
    let mut world = super::stories::plan_world(11727);
    let parent = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"dependency");
    let dependency = delegate(&mut world, parent, 1, child);
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"wait for dependency");
    child.dependencies = Box::new([core::Dependency::Existing(dependency)]);
    let task = delegate(&mut world, parent, 2, child);
    let inspection = inspect(&world, 11727);
    let wait = waiting(&inspection, task, "on its dependencies");
    assert!(wait.wakes.contains(&Wake::Waiter(format!("task {dependency}"))));
    assert_eq!(record(&world, task, core::WaitKind::Dependency).object, core::WaitObject::Task(dependency));
}

fn pool_queue(seed: u64) -> (World, u64, u64) {
    let mut world = super::stories::plan_world(seed);
    let (holder, task) = join_pool_queue(&mut world);
    (world, holder, task)
}

fn join_pool_queue(world: &mut World) -> (u64, u64) {
    configure_resources(world);
    world.send(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::System(jig_test_connector::SystemEvent::Pool {
            path: jig_test_connector_kit::path(1, 2),
            slots: 1,
            lost: Box::new([]),
        }),
    });
    let parent = world.assignments[0];
    let child = resource_member(world, 2, b"hold the pool");
    let holder = delegate(world, parent, 1, child);
    let child = resource_member(world, 2, b"queue for the pool");
    let task = delegate(world, parent, 2, child);
    (holder, task)
}

#[test]
fn inspection_walk_borrows_each_row_without_allocating() {
    let (world, _, _) = pool_queue(12796);
    let tasks = &world.domain.core().tasks;
    let expected = tasks.view_tasks();
    let heap = skein_world::domain::heap::Meter::new();
    heap.start();
    let mut after = None;
    for view in &expected {
        let row = tasks.inspection_task_after(after).expect("next named row");
        assert_eq!(row.number, view.number);
        assert!(std::ptr::eq(row, tasks.task(view.number).expect("named row")));
        after = Some(row.number);
    }
    assert!(tasks.inspection_task_after(after).is_none());
    assert_eq!(heap.end().peak(), 0, "inspection borrows rows, not a projection");
}

#[test]
fn a_pool_queue_has_the_holders_and_its_armed_bound() {
    let (world, holder, task) = pool_queue(11728);
    let inspection = inspect(&world, 11728);
    let wait = waiting(&inspection, task, "in a resource's queue");
    let bound = world
        .domain
        .core()
        .tasks
        .inspection_hold_alarms()
        .iter()
        .find_map(|(number, due)| (number == task).then_some(due))
        .expect("armed hold alarm");
    assert_eq!(wait.bound, Some(bound));
    assert!(wait.wakes.contains(&Wake::Waiter(format!("task {holder}"))));
    assert!(wait.wakes.contains(&Wake::Timer(bound)));
    let pool = inspection
        .tables()
        .iter()
        .find(|table| table.name == "pools" && !table.refused.is_empty())
        .expect("pool queue");
    assert_eq!(pool.capacity, 1);
    assert_eq!(pool.entries.len(), 1);
    assert_eq!(pool.entries[0].key, format!("task {holder}"));
    assert_eq!(pool.entries[0].release, Release::Pending);
    assert_eq!(pool.refused[0].key, format!("task {task}"));
    assert_eq!(pool.refused[0].bound, Some(bound));
    assert_eq!(pool.refused[0].since, skein_lib::Time::ZERO);
    assert!(
        matches!(record(&world, task, core::WaitKind::Resource).object, core::WaitObject::Resource { holder: Some(owner), .. } if owner == holder)
    );
}

#[test]
fn each_drain_inspects_once() {
    let mut world = super::stories::plan_world(11951);
    let before = world.inspections();
    world.drain();
    assert_eq!(world.inspections(), before + 1);
    world.drain();
    assert_eq!(world.inspections(), before + 2);
}

#[test]
fn a_settling_backoff_waits_for_the_next_start() {
    let seed = 11955;
    let mut world = World::new(jig_core_world::world::Setup { seed, ..Default::default() });
    world.end(tasks::End::Failed(tasks::Class::Transient), 0);
    let task = world.task();
    world.send(root::Event::Core(core::Event::Settling));
    world.advance(Duration::from_secs(60));
    let inspection = inspect(&world, seed);
    let wait = waiting(&inspection, task, "backing off");
    assert_eq!(record(&world, task, core::WaitKind::Account).task, task);
    assert_eq!(wait.bound, None);
    assert!(wait.wakes.contains(&Wake::Party("the next start".into())));
    let mut child = Inspection::new("jig", jig_inspect::KINDS);
    jig_inspect::tasks(&world.domain.core().tasks, &mut child);
    let wait = waiting(&child, task, "backing off");
    assert!(wait.bound.is_some_and(|bound| bound < world.clocks().0));
    world.restart(seed, false);
    world.retry_due();
    jig_core_world::fixtures::finish_chat(&mut world, 0, b"retry finished after settling");
}

#[test]
fn an_overdue_pool_wait_is_found_then_its_timer_holds_it() {
    let seed = 11952;
    let (mut world, _, task) = pool_queue(seed);
    let inspection = inspect(&world, seed);
    let bound = waiting(&inspection, task, "in a resource's queue").bound.expect("hold deadline");
    let at = bound.saturating_add(Duration::from_millis(1));
    let moment = Moment { seed, at, iteration: 1 };
    let finding = check_waits(&[&inspection], moment).expect_err("overdue hold wait");
    for text in [format!("task {task}"), "in a resource's queue".into(), bound.as_nanos().to_string()] {
        assert!(finding.contains(&text), "{finding}");
    }
    let finding = check_tables(&[&inspection], moment).expect_err("overdue admission");
    for text in ["pools".to_string(), format!("task {task}"), bound.as_nanos().to_string()] {
        assert!(finding.contains(&text), "{finding}");
    }
    let now = world.clocks().0;
    world.advance(Duration::from_nanos(at.as_nanos() - now.as_nanos()));
    let inspection = inspect(&world, seed);
    assert_eq!(waiting(&inspection, task, "held: holds waited").waiter, format!("task {task}"));
}

#[test]
fn a_pool_admits_its_waiter_when_the_holder_ends() {
    let seed = 11953;
    let (mut world, holder, task) = pool_queue(seed);
    let run = super::stories::running(&world, holder);
    super::stories::finish(&mut world, run, b"release the pool");
    assert!(world.assignments.iter().any(|(number, _)| *number == task));
    let inspection = inspect(&world, seed);
    let pool = inspection
        .tables()
        .iter()
        .find(|table| table.name == "pools" && !table.entries.is_empty())
        .expect("occupied pool");
    assert_eq!(pool.refused.len(), 0);
    assert_eq!(pool.entries[0].key, format!("task {task}"));
}

#[test]
fn drain_and_finish_return_quiet_point_findings() {
    let seed = 11954;
    let mut world = super::stories::plan_world(seed);
    world.enable_tracing();
    let task = world.task();
    let (at, wall) = world.clocks();
    world.set_clock(at, wall);
    let env = skein_lib::Env { now: at, wall, limits: jig_core_world::faults::plan_fixture(seed).1.core.tasks };
    // Drop the child's outputs deliberately: the root never routes its escalation.
    // This malformed fixture has a wait no timer can repair.
    tasks::step(
        &mut world.domain.core_mut().tasks,
        &env,
        tasks::Event::Hold { task, why: tasks::Hold::Budget },
        &mut skein_lib::Queue::uncapped(),
    );
    let finding = world.try_drain().expect_err("drain returns wait finding");
    assert!(finding.starts_with(&format!("seed {seed}: at {} ns, iteration ", at.as_nanos())), "{finding}");
    assert!(finding.contains("held: budget, routing escalation"), "{finding}");
    let diagnostic = world.diagnostic(&finding);
    assert!(diagnostic.contains(&finding));
    assert!(diagnostic.ends_with(&world.trace[world.trace.len().saturating_sub(30)..].join("\n")));
    let finding = world.finish().expect_err("oracle returns wait finding");
    assert!(finding.contains("held: budget, routing escalation"), "{finding}");
}

#[test]
fn a_coordinator_waits_on_its_delegate_and_watch() {
    let mut world = super::stories::plan_world(11729);
    let parent = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"work");
    let child = delegate(&mut world, parent, 1, child);
    subscribe(&mut world, parent.0, tasks::SubscriptionKind::Task { target: child, held: true, result: true });
    park(&mut world, parent);
    let inspection = inspect(&world, 11729);
    for kind in ["on its delegates", "on a watch"] {
        assert!(waiting(&inspection, parent.0, kind).wakes.contains(&Wake::Waiter(format!("task {child}"))));
    }
    assert_eq!(record(&world, parent.0, core::WaitKind::Delegate).object, core::WaitObject::Task(child));
    assert!(world.domain.core().waits(parent.0).own.iter().all(|wait| wait.kind != core::WaitKind::News));
}

#[test]
fn an_open_question_waits_on_its_requester() {
    let mut world = super::stories::plan_world(11730);
    let parent = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"ask");
    let task = delegate(&mut world, parent, 1, child);
    let run = super::stories::running(&world, task);
    ask_requester(&mut world, run, parent.0);
    park(&mut world, run);
    let inspection = inspect(&world, 11730);
    assert!(waiting(&inspection, task, "on its requester").wakes.contains(&Wake::Waiter(format!("task {}", parent.0))));
    assert!(
        matches!(record(&world, task, core::WaitKind::Question).object, core::WaitObject::Question { answerer, .. } if answerer == parent.0)
    );
}

#[test]
fn a_lone_parked_watcher_records_news() {
    const SEED: u64 = 11741;
    let mut world = super::stories::plan_world(SEED);
    let parent = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"watch the coordinator");
    let task = delegate(&mut world, parent, 1, child);
    let run = super::stories::running(&world, task);
    subscribe(&mut world, task, tasks::SubscriptionKind::Task { target: parent.0, held: true, result: true });
    park(&mut world, run);
    let inspection = inspect(&world, SEED);
    assert!(waiting(&inspection, task, "on a watch").wakes.contains(&Wake::Waiter(format!("task {}", parent.0))));
    assert_eq!(record(&world, task, core::WaitKind::News).task, task);
}

fn conversation(seed: u64) -> World {
    use jig_domain_people as people;
    let (configuration, limits) = jig_core_world::world::fixture(seed, false);
    let creating = jig_test_profile::fixtures::Creating::new(&configuration.core, limits.core.tasks.tree_tasks);
    let mut initial = jig_test_profile::fixtures::create_chat(&creating, 1, 0, b"a chat".as_slice().into());
    let people::Ask::Create { batch, .. } = &mut initial else { panic!("conversation create") };
    let member = batch.first_mut().expect("one conversation");
    member.flags.ended_by = people::EndedBy::Requester;
    World::new(Setup { seed, configuration: Some(configuration), limits, initial: Some(initial), ..Setup::default() })
}

#[test]
fn a_parked_conversation_records_standing_news() {
    const SEED: u64 = 11742;
    let mut world = conversation(SEED);
    let run = world.assignments[0];
    park(&mut world, run);
    let inspection = inspect(&world, SEED);
    assert!(waiting(&inspection, run.0, "on its party").wakes.contains(&Wake::Party("person 1".into())));
    assert_eq!(record(&world, run.0, core::WaitKind::News).bound, core::Bound::None(core::Unbounded::Standing));
}

#[test]
fn paced_words_leave_a_parked_task_waiting_for_news() {
    const SEED: u64 = 11743;
    const PACE: Duration = Duration::from_secs(10);
    let mut world = super::stories::plan_world(SEED);
    let parent = world.assignments[0];
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"wait for the next instruction");
    child.wake = Some(tasks::WakePolicy { words: tasks::WakeRule::Paced { span: PACE }, ..tasks::WakePolicy::DEFAULT });
    let task = delegate(&mut world, parent, 1, child);
    let run = super::stories::running(&world, task);
    world.await_timer(task);
    park(&mut world, run);
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9910)),
        key: key(parent, 2),
        action: core::NamedAction::Message {
            target: task,
            kind: tasks::MessageKind::Words,
            words: b"next".as_slice().into(),
        },
    }));
    let inspection = inspect(&world, SEED);
    assert!(waiting(&inspection, task, "paced").bound.is_some());
    assert_eq!(record(&world, task, core::WaitKind::News).task, task);
}

fn ask_requester(world: &mut World, run: (u64, u64), requester: u64) {
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9910)),
        key: key(run, 2),
        action: core::NamedAction::Message {
            target: requester,
            kind: tasks::MessageKind::Question,
            words: b"proceed?".as_slice().into(),
        },
    }));
}

#[test]
fn the_first_start_floors_old_waits_and_later_starts_keep_the_floor() {
    const SEED: u64 = 11740;
    const T0: Wall = Wall::from_nanos(10_000_000_000);
    const T1: Wall = Wall::from_nanos(12_000_000_000);
    const T2: Wall = Wall::from_nanos(13_000_000_000);
    let (configuration, mut limits) = jig_core_world::faults::plan_fixture(SEED);
    limits.core.tasks.delegates = 4;
    // Four result reservations and the question share the requester's inbox.
    limits.core.tasks.inbox_messages = 16;
    limits.core.tasks.inbox_bytes = 2048;
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(configuration),
        limits,
        clock: Some((Time::ZERO, T0)),
        ..Setup::default()
    });
    assert_floors(&world, Some(T0));
    let (_, queued) = join_pool_queue(&mut world);
    let parent = world.assignments[0];
    let leaf = |world: &World, words: &[u8]| {
        let mut child = member(world, tasks::Executor::Agent { charter: 1 }, words);
        child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
        child
    };
    let child = leaf(&world, b"ask the requester");
    let asking = delegate(&mut world, parent, 3, child);
    let run = super::stories::running(&world, asking);
    ask_requester(&mut world, run, parent.0);
    assert!(matches!(world.answers.last(), Some((_, _, core::CallPart::Sent { .. }))), "{:?}", world.answers.last());
    park(&mut world, run);
    let child = leaf(&world, b"escalate to the requester");
    let held = delegate(&mut world, parent, 4, child);
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task: held, why: tasks::Hold::Budget })));

    let waits = [
        (asking, core::WaitKind::Question, limits.core.waits.question),
        (held, core::WaitKind::Party, limits.core.tasks.escalation_stall),
        (queued, core::WaitKind::Resource, limits.core.tasks.hold_wait),
    ];
    world.clear_wait_floors_for_upgrade();
    assert_floors(&world, None);
    for (task, kind, figure) in waits {
        assert_wait_clock(&world, task, kind, T0, figure);
    }
    for at in [T1, T2] {
        world.set_clock(world.clocks().0, at);
        let (configuration, _) = jig_core_world::faults::plan_fixture(SEED);
        world.restart_with(configuration);
        assert!(world.domain.ready());
        assert_floors(&world, Some(T1));
        for (task, kind, figure) in waits {
            assert_wait_clock(&world, task, kind, T1, figure);
        }
    }
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task: asking, why: tasks::Hold::Budget })));
    assert_wait_clock(&world, asking, core::WaitKind::Party, T2, limits.core.tasks.escalation_stall);
}

fn assert_floors(world: &World, at: Option<Wall>) {
    use core::WaitKind::*;
    let core = world.domain.core();
    for kind in [Party, Question, Dependency, Resource, Account, Host, Delegate, News] {
        assert_eq!(core.wait_floor(kind), at, "{kind:?}");
    }
    for kind in [Step, Room, Turn, SignOff] {
        assert_eq!(core.wait_floor(kind), None, "{kind:?}");
    }
}

fn assert_wait_clock(world: &World, task: u64, kind: core::WaitKind, since: Wall, span: Duration) {
    let waits = world.domain.core().waits(task);
    let wait =
        waits.own.iter().find(|wait| wait.kind == kind).unwrap_or_else(|| {
            panic!("task {task} has no {kind:?} since {since:?} at {:?}: {waits:?}", world.clocks().1)
        });
    assert_eq!(wait.since, since, "{kind:?}");
    let core::Bound::At { at, .. } = wait.bound else { panic!("bound for {kind:?}: {wait:?}") };
    assert_eq!(at, Wall::from_nanos(since.as_nanos() + span.as_nanos()), "{kind:?}");
    assert!(at > world.clocks().1, "{kind:?} is not past its bound");
}

#[test]
fn a_timer_subscription_is_pending_until_its_armed_time() {
    let mut world = super::stories::plan_world(11731);
    let task = world.assignments[0];
    let (now, wall) = world.clocks();
    let span = Duration::from_secs(20);
    subscribe(
        &mut world,
        task.0,
        tasks::SubscriptionKind::Timer { at: Wall::from_nanos(wall.as_nanos() + span.as_nanos()), period: None },
    );
    park(&mut world, task);
    let inspection = inspect(&world, 11731);
    let wait = waiting(&inspection, task.0, "on a timer");
    let due = now.saturating_add(span);
    assert_eq!(wait.bound, Some(due));
    assert!(wait.wakes.contains(&Wake::Timer(due)));
    for table in inspection.tables().iter().filter(|table| table.name == "subscriptions") {
        assert_eq!(
            table.capacity,
            u64::from(jig_core_world::faults::plan_fixture(11731).1.core.tasks.subscriptions) + 1
        );
    }
    let entry = inspection
        .tables()
        .iter()
        .filter(|table| table.name == "subscriptions")
        .flat_map(|table| &table.entries)
        .find(|entry| entry.key == format!("task {} subscription 9901", task.0))
        .expect("timer entry");
    assert_eq!(entry.release, Release::Pending);
    assert_eq!(entry.rule, "the subscriber's end, or unsubscribing");
    let wall_due = Wall::from_nanos(wall.as_nanos() + span.as_nanos());
    assert!(
        matches!(record(&world, task.0, core::WaitKind::News).object, core::WaitObject::News { timer: Some(at), .. } if at == wall_due)
    );
}

#[test]
fn a_held_task_wakes_its_escalation_holder() {
    let mut world = super::stories::plan_world(11732);
    let parent = world.assignments[0];
    let child = member(&world, tasks::Executor::Agent { charter: 1 }, b"held work");
    let task = delegate(&mut world, parent, 1, child);
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Budget })));
    let inspection = inspect(&world, 11732);
    assert!(waiting(&inspection, task, "held: budget").wakes.contains(&Wake::Waiter(format!("task {}", parent.0))));
    assert!(matches!(
        record(&world, task, core::WaitKind::Party).object,
        core::WaitObject::Entry { kind: jig_domain_people::EntryKind::Escalation { .. }, .. }
    ));
}

#[test]
fn a_placed_hold_waits_on_its_owner_without_an_escalation_alarm() {
    let mut world = super::stories::plan_world(11736);
    let task = world.assignments[0].0;
    jig_core_world::fixtures::ask(
        &mut world,
        91,
        jig_domain_people::Ask::Stop { project: 1, task, reason: Box::from(b"Parking the task".as_slice()) },
    );
    let inspection = inspect(&world, 11736);
    let wait = waiting(&inspection, task, "held: placed");
    assert_eq!(wait.wakes, [Wake::Party("person 1".into())]);
    assert_eq!(wait.bound, None);
}

#[test]
fn closing_names_the_cores_host_and_the_tasks_own_words() {
    let mut world = super::stories::plan_world(11737);
    let task = world.assignments[0].0;
    jig_core_world::fixtures::ask(
        &mut world,
        92,
        jig_domain_people::Ask::Cancel { project: 1, task, reason: Box::new([]), grace: None },
    );
    let inspection = inspect(&world, 11737);
    assert_eq!(waiting(&inspection, task, "closing on its run").wakes, [Wake::Party("host 7".into())]);
    assert!(world.domain.core().waits(task).own.is_empty());
    let mut standalone = Inspection::new("jig", jig_inspect::KINDS);
    jig_inspect::tasks(&world.domain.core().tasks, &mut standalone);
    assert_eq!(waiting(&standalone, task, "closing on its run").wakes, [Wake::Party(format!("host of task {task}"))]);
    let moment = Moment { seed: 11737, at: world.clocks().0, iteration: 0 };
    assert_eq!(check_waits(&[&standalone], moment), Ok(()));
    assert_eq!(check_tables(&[&standalone], moment), Ok(()));
}

#[test]
fn a_person_task_wakes_its_person() {
    let mut world = jig_core_world::stories::offers::world_with_policy(11733, None, |limits, rules, policy| {
        limits.core.tasks.executor_kinds = 3;
        limits.core.authority.executors = 3;
        limits.core.documents.executors = 3;
        let kinds = Box::new([
            jig_domain_authority::Executor::Charter(1),
            jig_domain_authority::Executor::Procedure(1),
            jig_domain_authority::Executor::Role(0),
        ]);
        rules.ceiling.delegation.kinds = kinds.clone();
        policy.ceiling.delegation.kinds = kinds.clone();
        for role in &mut policy.roles {
            role.authority.delegation.kinds = kinds.clone();
        }
    });
    let parent = world.assignments[0];
    let mut child = member(&world, tasks::Executor::Person(tasks::PersonAddress::Person(1)), b"answer");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let task = delegate(&mut world, parent, 1, child);
    let inspection = inspect(&world, 11733);
    let wait = waiting(&inspection, task, "on a person's answer");
    assert!(wait.wakes.contains(&Wake::Party("person 1".into())));
    assert!(matches!(
        record(&world, task, core::WaitKind::Party).object,
        core::WaitObject::Entry { kind: jig_domain_people::EntryKind::PersonTask, .. }
    ));
    assert_eq!(wait.bound, None);
}

#[test]
fn answers_are_kept_until_the_core_clock_reaches_their_expiry() {
    let mut world = super::stories::plan_world(11735);
    let (now, wall) = world.clocks();
    world.set_clock(now, wall);
    let inspection = inspect(&world, 11735);
    let answers = inspection.tables().iter().find(|table| table.name == "kept answers").expect("answers table");
    assert_ne!(answers.entries.len(), 0);
    assert!(answers.entries.iter().all(|entry| entry.release == Release::Kept));
    let (key, expiry) = world.domain.core().people.inspection_answer(0).expect("creation answer");
    let mut name = format!("person {} request ", key.person);
    for byte in key.key {
        write!(name, "{byte:02x}").expect("writing to a string");
    }
    let key = name;
    let env = skein_lib::Env {
        now: expiry,
        wall: Wall::from_nanos(wall.as_nanos() + expiry.as_nanos() - now.as_nanos()),
        limits: jig_core_world::faults::plan_fixture(11735).1.core,
    };
    // Step an unrelated owner: people's expiration input has not run yet.
    let mut out = Queue::uncapped();
    engine::fire(world.domain.engine_mut(), &env, core::Timer::Resources, &mut out);
    assert!(out.is_empty());
    let mut inspection = Inspection::new("jig", jig_inspect::KINDS);
    jig_inspect::core(world.domain.core(), &mut inspection);
    let entry = inspection
        .tables()
        .iter()
        .find(|table| table.name == "kept answers")
        .expect("answers table")
        .entries
        .iter()
        .find(|entry| entry.key == key)
        .expect("answer retained before its timer");
    assert_eq!(entry.release, Release::Due);
    assert_eq!(entry.rule, "their retention's expiry");
    let moment = Moment { seed: 11735, at: expiry, iteration: 1 };
    let finding = check_tables(&[&inspection], moment).expect_err("expired answer has not been released");
    for text in ["kept answers", key.as_str(), "their retention's expiry"] {
        assert!(finding.contains(text), "{finding}");
    }
    world.set_clock(env.now, env.wall);
    // The ordinary drain fires all due owners before it inspects.
    let before = world.inspections();
    world.drain();
    assert_eq!(world.inspections(), before + 1);
    let inspection = inspect(&world, 11735);
    assert!(
        inspection
            .tables()
            .iter()
            .filter(|table| table.name == "kept answers")
            .flat_map(|table| &table.entries)
            .all(|entry| entry.key != key)
    );
}

#[test]
fn a_parked_batch_waits_for_the_missing_connectors_report() {
    let (configuration, mut limits) = jig_core_world::faults::plan_fixture(11734);
    limits.core.tasks.resource_reports = 2;
    let mut world = World::new(jig_core_world::world::Setup {
        seed: 11734,
        configuration: Some(configuration),
        limits,
        ..Default::default()
    });
    configure_resources(&mut world);
    for number in [10_u8, 11] {
        world.send(root::Event::connector_up(
            1,
            jig::Up::Resources(jig::resources::Up::Resource {
                project: 1,
                name: tasks::Name { connector: 1, path: Box::new([Box::new([number])]) },
                role: core::effects::connector::ResourceRole::Owned,
                hold: core::effects::connector::HoldKind::Exclusive { wait: true },
            }),
        ));
    }
    world.domain.delay_holding_reports(1, Box::new([true]));
    let parent = world.assignments[0];
    let child = resource_member(&world, 1, b"missing report");
    let (parked, wall) = world.clocks();
    world.set_clock(parked, wall);
    world.send(root::Event::Core(core::Event::Delegate {
        to: ReplyTo::new(Token::new(9940)),
        key: key(parent, 1),
        batch: Box::new([child]),
    }));
    assert!(world.domain.take_holding_report(1).is_some(), "connector withheld one report");
    assert!(world.domain.take_holding_report(1).is_none(), "exactly one report was withheld");
    assert_eq!(world.domain.core().awaiting_resources_flights(), 1);
    let inspection = inspect(&world, 11734);
    let wait = inspection
        .waits()
        .iter()
        .find(|wait| wait.kind == "a batch awaiting its creation reports")
        .expect("parked batch");
    assert!(wait.waiter.starts_with("batch "));
    assert!(wait.wakes.contains(&Wake::Party("connector 1".into())));
    assert_eq!(wait.bound, Some(parked.saturating_add(limits.core.tasks.hold_wait)));
    assert!(wait.wakes.contains(&Wake::Timer(wait.bound.expect("batch deadline"))));
    let reports = inspection.tables().iter().find(|table| table.name == "creation reports").expect("reports");
    let entry =
        reports.entries.iter().find(|entry| entry.key == "connector 1 resource \u{1}/\u{1}").expect("pinned name");
    assert_eq!(entry.release, Release::Pending);
}
