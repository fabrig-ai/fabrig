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
            reads_project: false,
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

const QUESTION_SPAN: u64 = 2 * HOUR;
const PERSON_SPAN: u64 = 8 * HOUR;
const QUESTION_PROJECT: u32 = jig_core_world::fixtures::PROJECT;
const QUESTION_PERSON: u64 = jig_core_world::fixtures::PARTY;
const OWNER_SUBJECT: u64 = 7;
const LAST_ROLE: u32 = 0;
const QUESTION_TEXT: &[u8] = b"Which result should I use?";
const ANSWER_TEXT: &[u8] = b"Use the first result.";

struct QuestionTree {
    world: World,
    covering: (u64, u64),
    first: (u64, u64),
    asker: u64,
    question: u64,
}

fn question_tree(seed: u64, cap: Option<Wall>) -> QuestionTree {
    let (configuration, limits) = jig_core_world::faults::plan_fixture(seed);
    let creating = jig_test_profile::fixtures::Creating::new(&configuration.core, limits.core.tasks.tree_tasks);
    let people::Ask::Create { project, mut batch } = jig_test_profile::fixtures::create_chat(
        &creating,
        QUESTION_PROJECT,
        LAST_ROLE,
        b"Decide for the plan".as_slice().into(),
    ) else {
        unreachable!()
    };
    batch[0].flags = people::Flags::PLAIN;
    let mut world = World::new(Setup {
        seed,
        configuration: Some(configuration),
        limits,
        initial: Some(people::Ask::Create { project, batch }),
        ..Setup::default()
    });
    let covering = world.assignments[0];
    let mut goal = member(&world, tasks::Executor::Agent { charter: 1 }, b"Decide for the change");
    goal.authority.budget.spend = 40;
    let first = delegate(&mut world, covering, 1, goal);
    let first = run_for(&world, first);
    let mut change = member(&world, tasks::Executor::Agent { charter: 1 }, b"Ask for a decision");
    change.authority.delegation.tasks = 0;
    change.authority.delegation.depth = 0;
    // The task's own deadline is independent of its holders' deadlines.
    change.authority.budget.deadline = cap;
    let asker = delegate(&mut world, first, 1, change);
    let run = run_for(&world, asker);
    let (now, at) = clocks(&world, T);
    world.set_clock(now, at);
    question_call(&mut world, run, first.0, tasks::MessageKind::Question, QUESTION_TEXT, 2);
    let Some((_, _, core::CallPart::Sent { message })) = world.answers.last() else {
        panic!("question refused: {:?}", world.answers.last())
    };
    let question = *message;
    QuestionTree { world, covering, first, asker, question }
}

fn run_for(world: &World, task: u64) -> (u64, u64) {
    world.assignments.iter().rev().find(|run| run.0 == task).copied().expect("task has a run")
}

fn question_call(
    world: &mut World,
    run: (u64, u64),
    target: u64,
    kind: tasks::MessageKind,
    words: &[u8],
    completion: u32,
) {
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(19_900 + u64::from(completion))),
        key: core::CallKey { task: run.0, attempt: run.1, completion, position: 0 },
        action: core::NamedAction::Message { target, kind, words: words.into() },
    }));
}

fn question_wait(tree: &QuestionTree) -> core::Wait {
    let waits = tree.world.domain.core().waits(tree.asker);
    let mut matching = waits.own.iter().filter(|wait| match wait.object {
        core::WaitObject::Question { number, .. } => number == tree.question,
        core::WaitObject::Entry { kind: people::EntryKind::Question { message }, .. } => message == tree.question,
        _ => false,
    });
    let wait = matching.next().expect("one open question").clone();
    assert!(matching.next().is_none(), "one question must have one wait");
    wait
}

fn at_task(tree: &QuestionTree, task: u64, since: Wall) {
    let wait = question_wait(tree);
    assert_eq!(wait.object, core::WaitObject::Question { number: tree.question, answerer: task });
    assert_eq!(wait.kind, core::WaitKind::Question);
    assert_eq!(wait.since, since);
}

fn at_party(tree: &QuestionTree, at: tasks::EscalationHolder, since: Wall, then: core::AtBound) {
    let wait = question_wait(tree);
    assert_eq!(
        wait.object,
        core::WaitObject::Entry { kind: people::EntryKind::Question { message: tree.question }, at }
    );
    assert_eq!(wait.kind, core::WaitKind::Party);
    assert_eq!(wait.since, since);
    assert_eq!(wait.bound, core::Bound::At { at: plus(since, PERSON_SPAN), then });
}

fn last_role() -> tasks::EscalationHolder {
    tasks::EscalationHolder::Role { project: QUESTION_PROJECT, role: LAST_ROLE }
}

#[test]
fn one_question_wait_moves_at_two_four_and_twelve_hours() {
    let mut tree = question_tree(17240, None);
    let task_bound = plus(T, QUESTION_SPAN);
    let person_bound = plus(task_bound, QUESTION_SPAN);
    let role_bound = plus(person_bound, PERSON_SPAN);
    fire_at(&mut tree.world, before(task_bound));
    at_task(&tree, tree.first.0, T);
    fire_at(&mut tree.world, task_bound);
    at_task(&tree, tree.covering.0, task_bound);
    fire_at(&mut tree.world, person_bound);
    at_party(&tree, tasks::EscalationHolder::Person(QUESTION_PERSON), person_bound, core::AtBound::PassUp);
    fire_at(&mut tree.world, role_bound);
    at_party(&tree, last_role(), role_bound, core::AtBound::Overdue);
    fire_at(&mut tree.world, plus(role_bound, PERSON_SPAN));
    assert_eq!(tree.world.domain.core().inspection_wait_bound(tree.asker), None);
    assert!(live_row(&tree.world, tree.asker).inbox.is_empty(), "the moves sent no words to the asker");
}

#[test]
fn restart_keeps_a_question_placement_and_rearms_its_bound() {
    let seed = 17241;
    let mut tree = question_tree(seed, None);
    let placed = plus(T, QUESTION_SPAN);
    let next = plus(placed, QUESTION_SPAN);
    fire_at(&mut tree.world, placed);
    tree.world.restart_with(jig_core_world::faults::plan_fixture(seed).0);
    at_task(&tree, tree.covering.0, placed);
    assert_eq!(tree.world.domain.core().inspection_wait_bound(tree.asker), Some(clocks(&tree.world, next).0));
}

#[test]
fn a_role_holders_answer_settles_the_original_question() {
    let mut tree = question_tree(17242, None);
    let task_bound = plus(T, QUESTION_SPAN);
    let person_bound = plus(task_bound, QUESTION_SPAN);
    for at in [task_bound, person_bound, plus(person_bound, PERSON_SPAN)] {
        fire_at(&mut tree.world, at);
    }
    tree.world.send(jig_core_world::fixtures::sign_in(
        Token::new(19_901),
        0,
        &OWNER_SUBJECT.to_be_bytes(),
        b"person",
        b"Person",
    ));
    let answer = people::Ask::AnswerQuestion {
        project: QUESTION_PROJECT,
        task: tree.asker,
        question: tree.question,
        words: ANSWER_TEXT.into(),
    };
    ask(&mut tree.world, 95, answer.clone());
    assert!(live_row(&tree.world, tree.asker).questions.is_empty(), "{:?}", tree.world.people_answers.last());
    ask(&mut tree.world, 96, answer);
    assert_eq!(
        tree.world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::QuestionAnswered {
            question: tree.question,
            by: people::QuestionAnswerer::Person(QUESTION_PERSON),
        })))
    );
}

#[test]
fn a_question_passes_at_the_askers_earlier_deadline() {
    let cap = plus(T, HOUR);
    let mut tree = question_tree(17243, Some(cap));
    fire_at(&mut tree.world, cap);
    // Each onward wait is also capped by that same deadline.
    let wait = question_wait(&tree);
    assert_eq!(
        wait.object,
        core::WaitObject::Entry { kind: people::EntryKind::Question { message: tree.question }, at: last_role() }
    );
    assert_eq!(wait.bound, core::Bound::At { at: cap, then: core::AtBound::Overdue });
}

#[test]
fn an_answer_before_the_bound_disarms_the_question() {
    let mut tree = question_tree(17244, None);
    fire_at(&mut tree.world, plus(T, HOUR));
    question_call(
        &mut tree.world,
        tree.first,
        tree.asker,
        tasks::MessageKind::Answer { question: tree.question },
        ANSWER_TEXT,
        2,
    );
    fire_at(&mut tree.world, plus(T, QUESTION_SPAN));
    assert_eq!(tree.world.domain.core().inspection_wait_bound(tree.asker), None);
    assert!(live_row(&tree.world, tree.asker).questions.is_empty());
}

#[test]
fn the_first_task_hears_who_answered_after_the_pass() {
    let mut tree = question_tree(17245, None);
    let pass = plus(T, QUESTION_SPAN);
    fire_at(&mut tree.world, pass);
    let answer_at = plus(pass, HOUR);
    fire_at(&mut tree.world, answer_at);
    let kind = tasks::MessageKind::Answer { question: tree.question };
    question_call(&mut tree.world, tree.covering, tree.asker, kind.clone(), ANSWER_TEXT, 2);
    fire_at(&mut tree.world, plus(answer_at, 10 * 60));
    question_call(&mut tree.world, tree.first, tree.asker, kind, ANSWER_TEXT, 2);
    let (_, _, core::CallPart::MessageRefused(problem)) = tree.world.answers.last().expect("answer reply") else {
        panic!("expected late refusal: {:?}", tree.world.answers.last());
    };
    assert_eq!(
        problem.why,
        tasks::Refusal::QuestionAnswered {
            question: tree.question,
            by: tasks::QuestionAnswerer::Task(tree.covering.0)
        }
    );
}
