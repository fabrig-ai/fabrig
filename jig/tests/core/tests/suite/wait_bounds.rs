//! Bound alarms over the core world, with the stock figures and supplied clocks.
use jig_core_world::{
    fixtures::{ask, delegate, live_row, member},
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks::{self as tasks, Active, Hold, HoldSource, Phase};
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Time, Token, Wall};

const T: Wall = Wall::from_nanos(1_795_500_420_000_000_000); // 2026-11-24T06:07Z
const HOUR: u64 = 3600;
const NEWS: u64 = 24 * HOUR;
const RUN: u64 = 2 * HOUR;
const CHANNEL: Token = Token::new(7);

fn plus(at: Wall, seconds: u64) -> Wall {
    Wall::from_nanos(at.as_nanos() + Duration::from_secs(seconds).as_nanos())
}

fn clocks(world: &World, at: Wall) -> (Time, Wall) {
    let (now, wall) = world.clocks();
    (now.saturating_add(Duration::from_nanos(at.as_nanos() - wall.as_nanos())), at)
}

fn fire_at(world: &mut World, at: Wall) {
    let (now, at) = clocks(world, at);
    world.fire_due(now, at);
}

fn before(at: Wall) -> Wall {
    Wall::from_nanos(at.as_nanos() - 1)
}

fn plan(seed: u64) -> World {
    let (configuration, limits) = jig_core_world::faults::plan_fixture(seed);
    World::new(Setup { seed, configuration: Some(configuration), limits, ..Setup::default() })
}

fn plain(world: &mut World) -> u64 {
    let child = member(world, tasks::Executor::Agent { charter: 1 }, b"wait for news");
    delegate(world, world.assignments[0], 1, child)
}

fn task_event(world: &mut World, event: tasks::Event) {
    world.send(root::Event::Core(core::Event::Tasks(event)));
}

fn subscribe(world: &mut World, task: u64, number: u64, at: Wall) {
    task_event(
        world,
        tasks::Event::Subscribe {
            reply_to: ReplyTo::new(Token::new(number)),
            task,
            subscription: tasks::Subscription { number, kind: tasks::SubscriptionKind::Timer { at, period: None } },
        },
    );
}

fn mark_park(world: &mut World, task: u64, at: Wall) {
    let (now, at) = clocks(world, at);
    world.set_clock(now, at);
    assert_eq!(world.task(), task);
    world.end(tasks::End::Parked, 0);
    assert_eq!(live_row(world, task).phase, Phase::Active(Active::Idle));
    assert_eq!(live_row(world, task).phase_since, at);
}

fn held(world: &World, task: u64, cause: Hold) -> &str {
    let phase = &live_row(world, task).phase;
    let Phase::Held { why, source: Some(source), .. } = phase else { panic!("expected a sourced hold: {phase:?}") };
    assert_eq!(*why, cause);
    let HoldSource::Wait { words } = source.as_ref() else { panic!("expected a wait source: {source:?}") };
    std::str::from_utf8(words).expect("wait words are text")
}

fn release(world: &mut World, task: u64, completion: u32) {
    let (requester, attempt) = world.assignments[0];
    let revision = live_row(world, task).revisions.hold;
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9900 + u64::from(completion))),
        key: core::CallKey { task: requester, attempt, completion, position: 0 },
        action: core::NamedAction::Release { target: task, revision, again: None },
    }));
}

#[test]
fn news_since_park_holds_at_twenty_four_hours_and_keeps_its_subscriptions_and_effect() {
    let mut world = plan(17230);
    let task = plain(&mut world);
    subscribe(&mut world, task, 3, plus(T, 26 * HOUR));
    subscribe(&mut world, task, 5, plus(T, 27 * HOUR));
    world.hold_writes = true;
    world.call(1, 9710);
    let key = world.call_key(1);
    assert_eq!(key.task, task);
    assert_eq!(world.pending_writes.len(), 1, "the task has a committed effect in flight");
    let pending = world.pending_writes[0].clone();
    mark_park(&mut world, task, T);
    let subscriptions = live_row(&world, task).subscriptions.clone();
    let bound = plus(T, NEWS);
    fire_at(&mut world, before(bound));
    assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Idle));
    fire_at(&mut world, bound);
    assert_eq!(
        held(&world, task, Hold::Waited),
        "Waited 24 h for news since 2026-11-24T06:07Z: subscriptions 3 and 5, a timer at 2026-11-25T08:07Z."
    );
    assert_eq!(live_row(&world, task).subscriptions, subscriptions);
    assert!(world.pending_writes.contains(&pending), "holding the task leaves the original write pending");
    world.release_writes();
    assert!(world.store.rows.values().any(|record| matches!(record,
        root::Record::Core(core::Record::Core(core::CoreRecord::Call(row))) if row.key == key && matches!(row.part,
            core::CallPart::Effect { outcome: Some(core::effects::connector::OutboxOutcome::Made), .. }))));
    assert_eq!(live_row(&world, task).subscriptions, subscriptions);
    assert!(matches!(
        live_row(&world, task).escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(1), .. }
    ));
}

#[test]
fn news_starts_afresh_when_a_timer_wakes_the_task_and_it_parks_again_or_is_released() {
    let mut world = plan(17231);
    let task = plain(&mut world);
    let wake = plus(T, 23 * HOUR);
    let again = plus(T, 23 * HOUR + 1800);
    subscribe(&mut world, task, 3, wake);
    subscribe(&mut world, task, 5, plus(T, 99 * HOUR));
    mark_park(&mut world, task, T);
    fire_at(&mut world, wake);
    assert!(matches!(live_row(&world, task).phase, Phase::Active(Active::Running { .. })));
    let read = world.assignment().messages.last().expect("the timer wake in the brief").number;
    world.turn(1, 0, Some(read), b"read the timer wake");
    mark_park(&mut world, task, again);
    fire_at(&mut world, plus(T, NEWS));
    assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Idle));
    let bound = plus(again, NEWS);
    fire_at(&mut world, bound);
    let first = held(&world, task, Hold::Waited).to_owned();
    release(&mut world, task, 2);
    assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Idle));
    assert_eq!(world.domain.core().waits(task).own.get(0).unwrap().since, bound);
    fire_at(&mut world, before(plus(bound, NEWS)));
    assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Idle));
    fire_at(&mut world, plus(bound, NEWS));
    assert_ne!(held(&world, task, Hold::Waited), first);
    assert_eq!(live_row(&world, task).revisions.hold, 2);
}

#[test]
fn the_news_of_standing_work_and_a_conversation_without_a_deadline_does_not_hold() {
    let mut world = plan(17232);
    let chat = world.task();
    world.park_for_message(0);
    let mut request = jig_core_world::fixtures::chat(&world, 1, b"keep watching".as_slice().into());
    let people::Ask::Create { batch, .. } = &mut request else { panic!("create standing work") };
    let member = &mut batch[0];
    member.flags.ended_by = people::EndedBy::Standing;
    member.flags.conversation = false;
    member.tracked = None;
    member.budget = people::Budget::Named;
    member.authority.spend = 20;
    ask(&mut world, 79, request);
    let standing = world.task();
    assert_ne!(standing, chat);
    world.park_for_message(0);
    for task in [chat, standing] {
        assert_eq!(world.domain.core().waits(task).first.unwrap().bound, core::Bound::None(core::Unbounded::Standing));
    }
    fire_at(&mut world, plus(T, 2 * NEWS));
    for task in [chat, standing] {
        assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Idle));
    }
}

#[test]
fn a_waiting_run_is_cancelled_then_held_as_parked_with_its_turn() {
    let mut world = plan(17233);
    let task = plain(&mut world);
    let attempt = world.attempt();
    world.turn(1, 1, None, b"committed waiting turn");
    let (now, at) = clocks(&world, T);
    world.set_clock(now, at);
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Observe {
        channel: CHANNEL,
        run: Token::new(task),
        attempt: Token::new(attempt),
        report: fleet::Report::Activity(fleet::Activity::Waiting),
    })));
    fire_at(&mut world, before(plus(T, NEWS)));
    assert_eq!(world.cancellations, []);
    fire_at(&mut world, plus(T, NEWS));
    assert_eq!(world.cancellations, [(task, attempt)]);
    assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Running { attempt }));
    world.end(tasks::End::Failed(tasks::Class::Agent), 2); // the host's engine-cancelled acknowledgement
    assert_eq!(held(&world, task, Hold::Waited), "Waited 24 h for news since 2026-11-24T06:07Z.");
    let row = live_row(&world, task);
    assert_eq!(row.tries, tasks::Tries::NONE);
    assert_eq!(row.run_spent, 2);
    assert!(matches!(row.phase, Phase::Held { was: tasks::Was::Active(Active::Due), .. }));
    assert!(world.store.rows.values().any(|record| matches!(record,
        root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == task && matches!(line.line,
            tasks::Line::RunEnded { end: tasks::EndKind::Parked, .. }))));
    release(&mut world, task, 2);
    let transcript = world.assignment_transcripts.last().expect("the released run resumes");
    assert_eq!(transcript.task, task);
    assert_eq!(transcript.turns[0].as_ref(), b"committed waiting turn");
}

#[test]
fn dependency_deadline_holds_at_equality_and_again_on_release_but_not_the_working_run() {
    let deadline = plus(T, 3600);
    let mut world = jig_core_world::fixtures::run_with_deadline(17234, Some(deadline));
    let parent = world.assignments[0];
    let dependency = plain(&mut world);
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"waiting for a dependency");
    child.dependencies = Box::new([core::Dependency::Existing(dependency)]);
    let task = delegate(&mut world, parent, 2, child);
    fire_at(&mut world, before(deadline));
    assert_eq!(live_row(&world, task).phase, Phase::Waiting);
    fire_at(&mut world, deadline);
    let first = held(&world, task, Hold::Deadline).to_owned();
    assert!(first.contains("dependency since "));
    assert!(held(&world, task, Hold::Deadline).contains(&format!("task {dependency}")));
    assert!(matches!(live_row(&world, dependency).phase, Phase::Active(Active::Running { .. })));
    assert_eq!(live_row(&world, parent.0).phase, Phase::Active(Active::Running { attempt: parent.1 }));
    release(&mut world, task, 3);
    fire_at(&mut world, plus(deadline, 1));
    let again = held(&world, task, Hold::Deadline);
    assert!(again.contains(&format!("task {dependency}")));
    assert!(again.contains("dependency since 2026-11-24T07:07Z"));
    assert_ne!(again, first);
    assert_eq!(live_row(&world, task).revisions.hold, 2);
}

#[test]
fn a_deadline_for_an_idle_task_with_a_delegate_names_the_delegate() {
    let deadline = plus(T, 3600);
    let mut world = jig_core_world::fixtures::run_with_deadline(17235, Some(deadline));
    let (task, attempt) = world.assignments[0];
    let child = plain(&mut world);
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: CHANNEL,
        task,
        attempt,
        cumulative: 0,
        end: tasks::End::Parked,
    });
    assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Idle));
    fire_at(&mut world, deadline);
    let words = held(&world, task, Hold::Deadline);
    assert!(words.contains("delegate since ") && words.contains(&format!("task {child}")), "{words}");
    assert!(matches!(live_row(&world, child).phase, Phase::Active(Active::Running { .. })));
}

#[test]
fn a_due_wait_for_a_host_or_an_account_without_a_reset_holds_at_two_hours() {
    use jig_domain_accounts as accounts;
    for kind in [core::WaitKind::Host, core::WaitKind::Account] {
        let mut world = plan(17236);
        let event = if kind == core::WaitKind::Host {
            core::Event::Fleet(fleet::Event::Draining { channel: CHANNEL })
        } else {
            core::Event::Account(accounts::Event::Limited { account: 1, reset: None })
        };
        world.send(root::Event::Core(event));
        let task = 2;
        let member = jig_test_profile::fixtures::people_member(b"due without a run", 10);
        ask(&mut world, 77, people::Ask::Create { project: 1, batch: Box::new([member]) });
        let wait = world.domain.core().waits(task).first.expect("due wait");
        assert_eq!(wait.kind, kind);
        let at = plus(wait.since, RUN);
        fire_at(&mut world, before(at));
        assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Due));
        fire_at(&mut world, at);
        let words = held(&world, task, Hold::Waited);
        let (kind, object) =
            if kind == core::WaitKind::Host { ("host", "no host has room") } else { ("account", "limited") };
        assert!(words.starts_with(&format!("Waited 2 h for {kind} since ")) && words.contains(object), "{words}");
    }
}

#[test]
fn a_bound_passed_at_restart_retries_an_uncommitted_hold_but_not_a_committed_hold() {
    let seed = 17237;
    for crash_before_commit in [false, true] {
        let mut world = plan(seed);
        let task = plain(&mut world);
        subscribe(&mut world, task, 3, plus(T, 99 * HOUR));
        mark_park(&mut world, task, T);
        let (now, at) = clocks(&world, plus(T, NEWS + 1));
        world.set_clock(now, at);
        if crash_before_commit {
            world.offer(root::Event::Timer(core::Timer::WaitBounds));
            world.collect();
            assert!(!world.store.pending.is_empty());
            assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Idle));
        }
        let config = || jig_core_world::faults::plan_fixture(seed).0;
        world.restart_with(config());
        assert!(held(&world, task, Hold::Waited).contains("subscriptions 3"));
        assert_eq!(live_row(&world, task).revisions.hold, 1);
        world.restart_with(config());
        assert_eq!(live_row(&world, task).revisions.hold, 1);
    }
}

#[test]
fn a_closed_suspension_moves_the_deadline_cap_and_the_hold_together() {
    let deadline = plus(T, HOUR);
    let mut world = jig_core_world::fixtures::run_with_deadline(17238, Some(deadline));
    let task = plain(&mut world);
    let (now, wall) = clocks(&world, T);
    world.set_clock(now, wall);
    world.end(
        tasks::End::Suspended(tasks::Suspended {
            worker: 2,
            why: tasks::SuspendWhy::Stopped,
            window: Duration::from_secs(600),
        }),
        0,
    );
    assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Due));
    let resumed = plus(T, 600);
    fire_at(&mut world, resumed);
    assert!(matches!(live_row(&world, task).phase, Phase::Active(Active::Running { .. })));
    world.park_for_message(0);
    let moved = plus(deadline, 600);
    assert_eq!(live_row(&world, task).suspended, Duration::from_secs(600).as_nanos());
    assert_eq!(
        world.domain.core().waits(task).first.unwrap().bound,
        core::Bound::At { at: moved, then: core::AtBound::Hold }
    );
    fire_at(&mut world, before(moved));
    assert_eq!(live_row(&world, task).phase, Phase::Active(Active::Idle));
    fire_at(&mut world, moved);
    assert!(held(&world, task, Hold::Deadline).starts_with("Deadline 2026-11-24T07:17Z"));
}

#[test]
fn a_hold_at_the_tasks_deadline_keeps_its_place_in_the_resource_queue() {
    use jig_core_world::fixtures::{configure_resources, resource_member};
    let mut world = plan(17239);
    configure_resources(&mut world);
    world.send(root::Event::Connector {
        number: 1,
        event: jig_test_connector::Event::System(jig_test_connector::SystemEvent::Pool {
            path: jig_test_connector_kit::path(1, 2),
            slots: 1,
            lost: Box::new([]),
        }),
    });
    let parent = world.assignments[0];
    let holder = resource_member(&world, 2, b"hold the slot");
    let holder = delegate(&mut world, parent, 1, holder);
    let mut child = resource_member(&world, 2, b"queue for a slot");
    let deadline = plus(world.clocks().1, 1);
    child.authority.budget.deadline = Some(deadline);
    let task = delegate(&mut world, parent, 2, child);
    let before = live_row(&world, task).clone();
    assert!(before.hold_wait_since.is_some());
    assert_eq!(world.domain.core().tasks.resource_wait(&before).unwrap().1, Some(holder));
    fire_at(&mut world, deadline);
    assert!(held(&world, task, Hold::Deadline).contains("resource since "));
    let after = live_row(&world, task);
    assert_eq!(before.hold_wait_since, after.hold_wait_since);
    assert_eq!(world.domain.core().tasks.resource_wait(&before), world.domain.core().tasks.resource_wait(after));
    release(&mut world, task, 3);
    assert_eq!(live_row(&world, task).hold_wait_since, before.hold_wait_since);
    assert!(held(&world, task, Hold::Deadline).contains("resource since "));
}
