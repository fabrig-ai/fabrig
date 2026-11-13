//! A goal's tier checks its branch while changes keep landing
//! (temper/docs/design/domain/after-landing.md, 3, 5.4, 7 and 12).
use super::*;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_world::tier_runs::{self, LANDING_TICKS, RUN_TICKS, merge, named_tier, task, until};

const GOAL_REQUEST: u8 = 92;
const PRIMER_CHANGE: u64 = 91;
const TIMER_CALL: u64 = 96;
const THIRD_CHANGE: u64 = 95;
const FAILURE: &[u8] = b"suite.alpha\n";
// One green primer precedes the stories' run and the run after it.
const BASELINE_SEQUENCE: u64 = 1;
const FIRST_CHECK: u64 = 2;
const NEXT_CHECK: u64 = 3;
const BASELINE_RUNS: usize = 1;
const GREEN_RUNS: usize = 2; // Primer and the run covering both changes.
const RED_PARTS: usize = 2; // Whole run and failed-test rerun, with no bisection.
const FIRST_RED_RUNS: usize = 3; // Primer and one pair of red parts.
const STILL_RED_RUNS: usize = 5; // Primer and two pairs of red parts.
const REPAIR_RUNS: usize = 4; // Primer, red pair, green repair.
const RED_MESSAGES: usize = 1; // Each run emits one red to its goal.
const FAILED_JOBS: usize = 1; // goal.yml has the fixture's one test job.
const FIRST_CHANGE: u64 = 93;
const SECOND_CHANGE: u64 = 94;
const GOAL_SPAN: Duration = Duration::from_secs(20 * 60); // after-landing.md, 2.

fn world() -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = tier_runs::goal_world();
    world.adopt();
    let mut hello = protocol::hello_slots(Token::new(7), 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    worker.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    worker.bounds.inbox = world.env.limits.tasks.inbox_messages;
    world.send(hello);
    let people::Ask::Create { project, mut batch } = tier_runs::goal(&world) else { panic!("goal template") };
    batch[0].symbols = Box::new([people::Symbol { connector: 0, symbol: 1 }]);
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    until(&mut world, RUN_TICKS, "main baseline finished", |world| {
        tier_runs::tier(world).record.sequence == BASELINE_SEQUENCE
    });
    (world, goal, branch)
}

fn goal_runs(world: &World) -> Vec<raw::Run> {
    tier_runs::workflow_runs(world, b"goal.yml")
}

// The first move runs immediately. The stories' changes then share its 20-minute span.
fn baseline(world: &mut World, goal: &engine::Assignment, branch: &[u8]) {
    let primer = change(world, goal, branch, PRIMER_CHANGE, b"baseline", b"first goal landing");
    landed(world, primer, branch);
    until(world, RUN_TICKS, "goal baseline finished", |world| {
        named_tier(world, b"goal").record.sequence == BASELINE_SEQUENCE
    });
}

fn change(world: &mut World, goal: &engine::Assignment, branch: &[u8], call: u64, path: &[u8], content: &[u8]) -> u64 {
    read_messages(world, goal);
    let input = smith_domain_run::HostInput::attested(
        br#"{"words":"Repair or extend the goal branch","landing":"goal"}"#.as_slice().into(),
    )
    .expect("change input");
    let temper_engine_smith::Decoded::Call(decoded) = temper_engine_smith::call(
        smith_domain_run::CallName {
            activation: goal.attempt,
            completion: u32::try_from(call).expect("bounded call"),
            position: 1,
        },
        b"change",
        &input,
    )
    .expect("goal change decoded") else {
        panic!("change call")
    };
    let before = world.answers.len();
    let errors = world.call_errors.len();
    let busy = world.busy_calls;
    let event = world.protocol.call(Token::new(7), goal.task, goal.attempt, call, decoded);
    world.send(event);
    until(world, LANDING_TICKS, "goal change delegated", |world| {
        world.answers.len() > before || world.call_errors.len() > errors || world.busy_calls > busy
    });
    assert_eq!(world.busy_calls, busy, "goal call busy: {call}");
    assert_eq!(world.call_errors.len(), errors, "goal change refused: {:?}; goal={goal:?}", world.call_errors);
    let answer = world.answers[before].clone();
    assert!(matches!(answer, engine::CallAnswer::Delegated(_)), "change call {call}: {answer:?}");
    tier_runs::produce_delegated(world, answer, branch, path, content)
}

fn landed(world: &mut World, change: u64, branch: &[u8]) {
    until(world, LANDING_TICKS, "goal change result", |world| {
        matches!(task(world, change).phase, tasks::Phase::Ended(_))
    });
    assert!(matches!(tier_runs::change(world, change).change.state, State::Landed { .. }));
    let words = tiers::result(world, change);
    assert!(
        words.contains(&format!("merged into {} as", std::str::from_utf8(branch).expect("goal branch"))),
        "{words}"
    );
    assert!(!words.contains("Cleared by"), "no tier verdict on a goal landing: {words}");
    assert!(!words.contains("Tier "), "no tier line on a goal landing: {words}");
}

fn two_changes(world: &mut World, goal: &engine::Assignment, branch: &[u8], failures: &[u8]) -> [u64; 2] {
    baseline(world, goal, branch);
    let first = change(world, goal, branch, FIRST_CHANGE, b"one", b"first change");
    landed(world, first, branch);
    let second = change(world, goal, branch, SECOND_CHANGE, b"failures", failures);
    landed(world, second, branch);
    assert_ne!(first, second, "two distinct changes land into the goal branch");
    [first, second]
}

fn span_end(world: &World) -> Time {
    named_tier(world, b"goal").goals[0].started.expect("goal baseline start").saturating_add(GOAL_SPAN)
}

fn advance_to_span(world: &mut World) {
    world.env.now = span_end(world);
    tier_runs::tick(world);
}

#[test]
fn changes_into_a_goal_branch_are_cleared_as_they_land() {
    let (mut world, goal, branch) = world();
    assert!(goal_runs(&world).is_empty(), "a branch without landings starts no goal run");
    let [_, second] = two_changes(&mut world, &goal, &branch, b"");
    assert!(world.env.now < span_end(&world), "both changes land within the span");
    assert_eq!(goal_runs(&world).len(), BASELINE_RUNS, "only the goal baseline ran before the span");
    world.env.now = Time::from_nanos(span_end(&world).as_nanos() - Duration::from_secs(1).as_nanos());
    tier_runs::tick(&mut world);
    assert_eq!(goal_runs(&world).len(), BASELINE_RUNS, "the span has not passed");
    let due = span_end(&world);
    advance_to_span(&mut world);
    until(&mut world, RUN_TICKS, "newest goal tip checked", |world| {
        named_tier(world, b"goal").record.sequence == FIRST_CHECK
    });
    let runs = goal_runs(&world);
    assert!(runs[BASELINE_RUNS].created >= due, "the span passed before dispatch");
    assert_eq!(runs.len(), GREEN_RUNS, "one run covers both changes");
    assert_eq!(translate::commit(runs[BASELINE_RUNS].commit), merge(&world, second));
    assert_eq!(runs[BASELINE_RUNS].conclusion, Some(raw::Conclusion::Success));
    assert_eq!(tier_runs::tier(&world).record.sequence, BASELINE_SEQUENCE, "main starts no further run");
    assert!(tier_runs::tier(&world).record.run.is_none());
    assert_eq!(world.fake.branch(b"org/repo", b"main"), Some(1));
    eprintln!("seed={}, simulated={:?}", tier_runs::SEED, world.env.now);
    world.close();
}

fn news(world: &World, goal: u64) -> Vec<&tasks::Word> {
    task(world, goal)
        .inbox
        .iter()
        .filter(|word| matches!(word.kind, tasks::MessageKind::News { .. }) && word.words.starts_with(b"Red on "))
        .collect()
}

fn park(world: &mut World, goal: &engine::Assignment) {
    // Topic subscriptions alone do not keep an otherwise finished task waiting.
    // A timer beyond the story's horizon leaves it available for new forge news.
    if !task(world, goal.task)
        .subscriptions
        .iter()
        .any(|subscription| matches!(subscription.kind, tasks::SubscriptionKind::Timer { .. }))
    {
        let before = world.answers.len();
        let event = world.protocol.call(
            Token::new(7),
            goal.task,
            goal.attempt,
            TIMER_CALL,
            engine::Call {
                completion: u32::try_from(TIMER_CALL).expect("bounded timer call"),
                position: 1,
                tool: engine::Tool::Subscribe {
                    kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
                },
            },
        );
        world.send(event);
        until(world, LANDING_TICKS, "idle goal timer subscribed", |world| world.answers.len() > before);
    }
    let (taken, turns) = read_messages(world, goal);
    park_after_turn(world, goal, taken, turns);
    until(world, LANDING_TICKS, "goal parked awaiting its tier", |world| {
        task(world, goal.task).phase == tasks::Phase::Active(tasks::Active::Idle)
    });
}

/// Consume the inbox through the windows delivered to this run.
pub(super) fn read_messages(world: &mut World, goal: &engine::Assignment) -> (Option<u64>, u32) {
    world.until(Until::Ready);

    let mut turns = task(world, goal.task).turn;
    let mut taken = delivered_through(world, goal);
    for _ in 0..world.env.limits.tasks.inbox_messages {
        if task(world, goal.task).inbox.is_empty() {
            break;
        }
        // Read only words the worker received, opening the next bounded inbox window.
        taken = delivered_through(world, goal);
        turns += 1;
        world.send(protocol::turn(Token::new(7), goal.task, goal.attempt, turns, 0, taken, b"Await the goal tier"));
        world.until(Until::Ready);
    }
    (taken, turns)
}

fn delivered_through(world: &World, goal: &engine::Assignment) -> Option<u64> {
    goal.messages
        .iter()
        .map(|word| word.number)
        .chain(world.delivered_messages.get(&(goal.task, goal.attempt)).into_iter().flatten().copied())
        .max()
}

fn red_goal() -> (World, engine::Assignment, Box<[u8]>, skein_forge::api::Commit) {
    let (mut world, goal, branch) = world();
    let [_, second] = two_changes(&mut world, &goal, &branch, FAILURE);
    let tip = merge(&world, second);
    park(&mut world, &goal);
    advance_to_span(&mut world);
    until(&mut world, RUN_TICKS * 2, "new red news wakes the goal", |world| {
        !news(world, goal.task).is_empty()
            && world.assigned.iter().any(|run| run.task == goal.task && run.attempt > goal.attempt)
    });
    let resumed = world.assigned.iter().rev().find(|run| run.task == goal.task).expect("goal woken").clone();
    (world, resumed, branch, tip)
}

fn assert_red(world: &World, goal: &engine::Assignment, branch: &[u8], tip: skein_forge::api::Commit, repeated: bool) {
    let messages = news(world, goal.task);
    let (sequence, counts, line, class) = if repeated {
        (NEXT_CHECK, "0 new, 1 still failing.", "still failing: suite.alpha", tasks::NewsClass::Kept)
    } else {
        (FIRST_CHECK, "1 new, 0 still failing.", "new: suite.alpha", tasks::NewsClass::Wakes)
    };
    assert_eq!(messages.len(), RED_MESSAGES, "one news per red run");
    let message = &messages[0];
    assert!(matches!(message.kind, tasks::MessageKind::News { class: received, .. } if received == class));
    let words = std::str::from_utf8(&message.words).expect("red news words");
    let digits = forge_top::commit_text(&tip);
    let short = std::str::from_utf8(&digits[..12]).expect("short commit");
    let branch = std::str::from_utf8(branch).expect("goal branch");
    assert!(words.starts_with(&format!("Red on {branch}: tier goal run {sequence} at {short}: {counts}\n")), "{words}");
    assert!(words.lines().any(|word| word == line), "{words}");
    let runs = goal_runs(world);
    let expected_runs = if repeated { STILL_RED_RUNS } else { FIRST_RED_RUNS }; // Green baseline, then whole and confirming rerun per red.
    assert_eq!(runs.len(), expected_runs, "no bisection on a goal branch");
    for run in &runs[runs.len() - RED_PARTS..] {
        assert_eq!(translate::commit(run.commit), tip);
        assert_eq!(run.conclusion, Some(raw::Conclusion::Failure));
    }
    let rerun = runs.last().expect("confirming rerun");
    assert_eq!(
        rerun.inputs.iter().find(|(name, _)| name.as_ref() == b"temper_tests").expect("failed ids selected").1.as_ref(),
        FAILURE
    );
    assert!(words.lines().any(|line| line == format!("Run: {}", tier_runs::run_url(rerun))), "{words}");
    let logs = tier_runs::job_logs(world, rerun);
    assert_eq!(logs.len(), FAILED_JOBS, "one failed job");
    let log = &logs[0];
    let name = std::str::from_utf8(&log.name).expect("job name");
    let end = std::str::from_utf8(log.end.as_deref().expect("failed job log")).expect("job log");
    assert_ne!(end, "");
    assert!(words.contains(&format!("End of job {name}'s log:\n{end}")), "{words}");
}

#[test]
fn a_red_of_the_goal_tier_reaches_the_goal_as_news() {
    let (mut world, goal, branch, tip) = red_goal();
    assert_red(&world, &goal, &branch, tip, false);
    assert!(
        goal.messages.iter().any(|word| word.number == news(&world, goal.task)[0].number),
        "woken goal hears the red"
    );
    let third = change(&mut world, &goal, &branch, THIRD_CHANGE, b"three", b"merges continue while red");
    landed(&mut world, third, &branch);
    assert_eq!(
        named_tier(&world, b"goal").goals[0].failing,
        temper_engine_domain_forge_tiers::Failures::Tests(Box::new([b"suite.alpha".as_slice().into()]))
    );
    park(&mut world, &goal);
    let assignments = world.assigned.len();
    advance_to_span(&mut world);
    until(&mut world, RUN_TICKS * 2, "same failure kept as news", |world| news(world, goal.task).len() == RED_MESSAGES);
    for _ in 0..LANDING_TICKS {
        tier_runs::tick(&mut world);
    }
    assert_red(&world, &goal, &branch, merge(&world, third), true);
    assert_eq!(world.assigned.len(), assignments, "still failing news does not wake the goal");
    assert_eq!(task(&world, goal.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(tier_runs::tier(&world).record.sequence, BASELINE_SEQUENCE, "main does not follow the goal branch");
    eprintln!("seed={}, simulated={:?}", tier_runs::SEED, world.env.now);
    world.close();
}

#[test]
fn a_repair_into_the_goal_branch_turns_it_green() {
    let (mut world, goal, branch, _) = red_goal();
    let repair = change(&mut world, &goal, &branch, THIRD_CHANGE, b"failures", b"");
    landed(&mut world, repair, &branch);
    assert_eq!(tier_runs::change(&world, repair).repairs, None, "a goal tier gives no red landing to name");
    park(&mut world, &goal);
    let assignments = world.assigned.len();
    advance_to_span(&mut world);
    until(&mut world, RUN_TICKS, "goal repair checked green", |world| {
        named_tier(world, b"goal").record.sequence == NEXT_CHECK
    });
    for _ in 0..LANDING_TICKS {
        tier_runs::tick(&mut world);
    }
    let runs = goal_runs(&world);
    assert_eq!(runs.len(), REPAIR_RUNS, "baseline, red whole and rerun, green repair");
    let run = runs.last().expect("repair run");
    assert_eq!(translate::commit(run.commit), merge(&world, repair));
    assert_eq!(run.conclusion, Some(raw::Conclusion::Success));
    assert_eq!(
        named_tier(&world, b"goal").goals[0].failing,
        temper_engine_domain_forge_tiers::Failures::Tests(Box::new([]))
    );
    assert!(news(&world, goal.task).is_empty(), "a green run sends no news");
    assert_eq!(world.assigned.len(), assignments, "a green run does not wake the goal");
    assert_eq!(task(&world, goal.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    eprintln!("seed={}, simulated={:?}", tier_runs::SEED, world.env.now);
    world.close();
}

#[test]
fn a_goals_close_ends_its_branchs_following() {
    let (mut world, goal, branch) = world();
    baseline(&mut world, &goal, &branch);
    let failing = change(&mut world, &goal, &branch, FIRST_CHANGE, b"failures", FAILURE);
    landed(&mut world, failing, &branch);
    advance_to_span(&mut world);
    until(&mut world, LANDING_TICKS, "goal run in flight", |world| {
        named_tier(world, b"goal").task.is_some()
            && goal_runs(world).last().is_some_and(|run| run.state == raw::RunState::Running)
    });
    let run_task = named_tier(&world, b"goal").task.expect("goal tier task");
    let (taken, turns) = read_messages(&mut world, &goal);
    let mut event = world.protocol.answer(
        Token::new(7),
        goal.task,
        goal.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: host::LinkUp::Answer { answer, .. }, .. },
        ..
    }) = &mut event
    else {
        panic!("goal finish")
    };
    answer.read = taken.map(Token::new);
    answer.turns = turns;
    world.send(event);
    until(&mut world, LANDING_TICKS, "goal ended and branch released", |world| {
        matches!(task(world, goal.task).phase, tasks::Phase::Ended(tasks::Ending::Done(_)))
            && goal_branch::forge_hold(world, &branch).is_none()
    });
    assert!(named_tier(&world, b"goal").record.run.is_some(), "the goal ends before its tier run");
    until(&mut world, RUN_TICKS * 2, "in-flight run ends after goal close", |world| {
        named_tier(world, b"goal").record.sequence == FIRST_CHECK
            && named_tier(world, b"goal").record.run.is_none()
            && matches!(task(world, run_task).phase, tasks::Phase::Ended(tasks::Ending::Done(_)))
    });
    assert!(named_tier(&world, b"goal").goals.is_empty(), "released goal is no longer followed");
    assert!(news(&world, goal.task).is_empty(), "the late red tells no closed goal");
    let runs = goal_runs(&world);
    assert_eq!(runs.len(), FIRST_RED_RUNS, "baseline, red whole and confirming rerun");
    for run in &runs[BASELINE_RUNS..] {
        assert_eq!(run.conclusion, Some(raw::Conclusion::Failure));
        assert_eq!(translate::commit(run.commit), merge(&world, failing));
    }
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"after", b"after goal close", 1)
        .expect("released branch moved");
    world.env.now = world.env.now.saturating_add(GOAL_SPAN);
    for _ in 0..RUN_TICKS {
        tier_runs::tick(&mut world);
    }
    assert_eq!(goal_runs(&world).len(), runs.len(), "a released branch starts no more runs");
    assert!(named_tier(&world, b"goal").goals.is_empty());
    assert_eq!(tier_runs::tier(&world).record.sequence, BASELINE_SEQUENCE);
    eprintln!("seed={}, simulated={:?}", tier_runs::SEED, world.env.now);
    world.close();
}
