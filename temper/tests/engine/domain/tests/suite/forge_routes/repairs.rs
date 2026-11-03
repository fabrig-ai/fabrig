//! A goal's repair names a red landing and keeps it across restart
//! (temper/docs/design/domain/agent.md, 4.3.1; after-landing.md, 7).
use super::tiers::{baseline, cleared, produce, result};
use super::*;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_forge_tiers::Failures;
use temper_engine_domain_world::tier_runs::{
    self, LANDING_TICKS, RUN_TICKS, SEED, Tier, change, merge, produce_delegated, restart_cut, task, tier, until,
};

const GOAL_REQUEST: u8 = 92;
const REPAIR_CALL: u64 = 96;
const GOAL_BUDGET: u64 = 400; // The tiers fixture funds four producers while their landings remain live.

// Four changes reserve their results while the goal retains branch and CI news.
const GOAL_INBOX_MESSAGES: u32 = 32;

fn red_range() -> (World, engine::Assignment, [u64; 3]) {
    let mut world = tier_runs::world(false, Tier::default());
    world.env.limits.tasks.inbox_messages = GOAL_INBOX_MESSAGES;
    world.env.limits.tasks.inbox_bytes = GOAL_INBOX_MESSAGES * world.env.limits.tasks.message_bytes;
    world.env.limits.people.inbox_recipients = GOAL_INBOX_MESSAGES * 2 + world.env.limits.authority.roles + 4;
    let client = world.env.limits.forge.client;
    world = world.with_forge_client(client);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = goal_branch::goal_request(&world, true) else {
        unreachable!("goal template creates a batch")
    };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(GOAL_BUDGET) };
    let goal = assigned_chat(&mut world, GOAL_REQUEST, people::Ask::Create { project, batch });
    let mut landings = Vec::new();
    // The red range's documentation, failing, and unrelated landings use the tiers world's content cues.
    for (call, path, content) in [
        (93, b"docs".as_slice(), b"only documentation".as_slice()),
        (94, b"failures".as_slice(), tier_runs::FAILURES),
        (95, b"file".as_slice(), b"unrelated edit".as_slice()),
    ] {
        let answer = change_tool::change(&mut world, &goal, call, None, b"Small fix");
        landings.push(tier_runs::produce_delegated(&mut world, answer, b"main", path, content));
    }
    let landings = landings.try_into().expect("three range landings");
    let [_, red, _] = landings;
    until(&mut world, RUN_TICKS * 3, "blamed landing red", |world| {
        matches!(change(world, red).change.state, State::Red { .. })
    });
    (world, goal, landings)
}

fn repair(world: &mut World, goal: &engine::Assignment, commit: skein_forge::api::Commit, base: Option<&[u8]>) {
    let event = world.protocol.call(
        Token::new(7),
        goal.task,
        goal.attempt,
        REPAIR_CALL,
        engine::Call {
            completion: u32::try_from(REPAIR_CALL).expect("bounded call"),
            position: 1,
            tool: engine::Tool::Change {
                changes: Box::new([engine::ChangeCall {
                    repairs: Some(commit),
                    landing: base.map(|branch| engine::ChangeLanding::Branch(branch.into())),
                    ..engine::ChangeCall::words(b"Repair the red landing".as_slice().into(), None)
                }]),
            },
        },
    );
    world.send(event);
}

#[test]
fn a_goals_change_keeps_the_red_landing_it_repairs_across_restart() {
    let (mut world, goal, [_, red, _]) = red_range();
    let commit = merge(&world, red);
    let before = world.answers.len();
    repair(&mut world, &goal, commit, None);
    until(&mut world, LANDING_TICKS, "repair delegated", |world| world.answers.len() > before);
    let answer = world.answers[before].clone();
    let engine::CallAnswer::Delegated(made) = &answer else { panic!("repair refused: {:?}", world.call_errors) };
    let number = made[0].task;
    until(&mut world, LANDING_TICKS, "repair row saved", |world| {
        world.store.rows.contains_key(&Key::Forge(forge_top::Key::Change(number)))
    });
    assert_eq!(change(&world, number).repairs, Some(commit));
    assert!(
        live_task(&world, number)
            .spec
            .parameters
            .contains(&tasks::Parameter::Bytes { name: 12, value: engine::commit_text(&commit) })
    );
    until(&mut world, LANDING_TICKS, "restart cut", restart_cut);
    // A worker reconnects with the attempts it still hosts.
    world.assigned = world.assigned.iter().filter(|run| {
        matches!(tier_runs::task(&world, run.task).phase, tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == run.attempt)
    }).cloned().collect();
    world.restart_same();
    until(&mut world, LANDING_TICKS, "root restored", |world| world.root.ready());
    tier_runs::produce_delegated(&mut world, answer, b"main", b"failures", b"");
    assert_eq!(change(&world, number).repairs, Some(commit));
    until(&mut world, RUN_TICKS, "repair landing cleared", |world| {
        matches!(change(world, number).change.state, State::Cleared { .. })
    });
    world.close();
}

#[test]
fn a_cleared_landing_is_refused_as_a_red_to_repair() {
    let (mut world, goal, [cleared, _, _]) = red_range();
    until(&mut world, RUN_TICKS, "unblamed landing cleared", |world| {
        matches!(change(world, cleared).change.state, State::Cleared { .. })
    });
    let commit = merge(&world, cleared);
    let before = world.call_errors.len();
    repair(&mut world, &goal, commit, None);
    until(&mut world, LANDING_TICKS, "cleared repair refused", |world| world.call_errors.len() > before);
    let digits = String::from_utf8(engine::commit_text(&commit).into_vec()).expect("commit digits");
    assert_eq!(world.call_errors[before].text.as_ref(), format!("repairs: {digits} is red in no tier.").as_bytes());
    world.close();
}

#[test]
fn a_red_landing_on_another_branch_is_refused() {
    let (mut world, goal, [_, red, _]) = red_range();
    let commit = merge(&world, red);
    let before = world.call_errors.len();
    repair(&mut world, &goal, commit, Some(b"release"));
    until(&mut world, LANDING_TICKS, "other branch repair refused", |world| world.call_errors.len() > before);
    let digits = String::from_utf8(engine::commit_text(&commit).into_vec()).expect("commit digits");
    assert_eq!(
        world.call_errors[before].text.as_ref(),
        format!("repairs: {digits} landed into main, not release.").as_bytes()
    );
    world.close();
}

fn goal_world() -> (World, engine::Assignment) {
    let mut world = tier_runs::news_world();
    world.adopt();
    let mut event = protocol::hello_slots(Token::new(7), 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("worker hello")
    };
    hello.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    world.send(event);
    let ask = tier_runs::goal(&world);
    let goal = assigned_chat(&mut world, 92, ask);
    assert_eq!(task(&world, goal.task).tracked, Some(0));
    let branch = format!("temper/{}/g{}", goal.task, goal.task);
    assert!(super::goal_branch::forge_hold(&world, branch.as_bytes()).is_none());
    baseline(&mut world);
    (world, goal)
}

fn red_goal() -> (World, engine::Assignment, u64) {
    let (mut world, goal) = goal_world();
    let red = produce(&mut world, &goal, 93, b"failures", tier_runs::FAILURES);
    assert_eq!(task(&world, red).requester, tasks::Party::Task(goal.task));
    assert_eq!(world.fake.branch(b"org/repo", b"main").map(translate::commit), Some(merge(&world, red)));
    let taken = task(&world, goal.task).inbox.iter().map(|word| word.number).max();
    world.send(protocol::turn(Token::new(7), goal.task, goal.attempt, 1, 0, taken, b"Await main's verdict"));
    world.until(Until::Ready);
    park_after_turn(&mut world, &goal, taken, 1);
    until(&mut world, LANDING_TICKS, "goal parked awaiting verification", |world| {
        task(world, goal.task).phase == tasks::Phase::Active(tasks::Active::Idle)
    });
    until(&mut world, RUN_TICKS * 2, "red verdict and goal news kept", |world| {
        matches!(change(world, red).change.state, State::Red { .. })
            && !tier_runs::red_news(world, goal.task).is_empty()
    });
    until(&mut world, LANDING_TICKS, "red wakes goal", |world| {
        world.assigned.iter().any(|run| run.task == goal.task && run.attempt > goal.attempt)
    });
    let resumed = world.assigned.iter().rev().find(|run| run.task == goal.task).expect("goal woken").clone();
    assert_red_news(&world, &resumed, red);
    eprintln!("seed={SEED}, simulated={:?}", world.env.now);
    (world, resumed, red)
}

fn assert_red_news(world: &World, goal: &engine::Assignment, red: u64) {
    let news = tier_runs::red_news(world, goal.task);
    assert_eq!(news.len(), 1, "one red news message per goal");
    assert!(matches!(news[0].kind, tasks::MessageKind::News { class: tasks::NewsClass::Wakes, .. }));
    assert!(goal.messages.iter().any(|word| word.number == news[0].number), "the resumed goal hears the news");
    let commit = String::from_utf8(forge_top::commit_text(&merge(world, red)).into_vec()).expect("commit digits");
    let words = std::str::from_utf8(&news[0].words).expect("news words");
    let sequence = tier(world).record.sequence;
    assert!(
        words.starts_with(&format!(
            "Red after landing: tier slow run {sequence} blames landing {commit} on main: 2 failing.\n"
        )),
        "{words}"
    );
    assert_eq!(
        tier(world).record.failing,
        Failures::Tests(Box::new([b"suite.alpha".as_slice().into(), b"suite.beta".as_slice().into(),]))
    );
    assert!(words.lines().any(|line| line == "suite.alpha"), "{words}");
    assert!(words.lines().any(|line| line == "suite.beta"), "{words}");
    let runs = tier_runs::runs(world);
    assert_eq!(runs.len(), 3, "baseline, whole run, confirming rerun");
    for run in &runs[1..] {
        assert_eq!(translate::commit(run.commit), merge(world, red));
        assert_eq!(run.conclusion, Some(raw::Conclusion::Failure));
    }
    let rerun = &runs[2];
    assert_eq!(
        rerun
            .inputs
            .iter()
            .find(|(name, _)| name.as_ref() == b"temper_tests")
            .expect("rerun selects failed ids")
            .1
            .as_ref(),
        tier_runs::FAILURES
    );
    assert!(words.lines().any(|line| line == format!("Run: {}", tier_runs::run_url(rerun))), "{words}");
    let logs = tier_runs::job_logs(world, rerun);
    assert_eq!(logs.len(), 1, "one failed test job");
    for log in logs {
        let name = std::str::from_utf8(&log.name).expect("job name");
        let log = std::str::from_utf8(log.end.as_deref().expect("failed job has a log")).expect("job log");
        assert!(!log.is_empty());
        assert!(words.contains(&format!("End of job {name}'s log:\n{log}")), "{words}");
    }
}

#[test]
fn a_red_after_landing_reaches_its_goal_as_news() {
    let (mut world, goal, _) = red_goal();
    for _ in 0..LANDING_TICKS {
        tier_runs::tick(&mut world);
    }
    assert_eq!(tier_runs::red_news(&world, goal.task).len(), 1, "later polls do not duplicate the red");
}

#[test]
#[ignore = "temper-engine-domain/src/route/forge_route.rs tier_landing_run loses the repair's completed tier task; repair result omits run 3"]
fn a_repair_naming_its_red_clears_it() {
    let (mut world, goal, red) = red_goal();
    let commit = merge(&world, red);
    let answer = request_repair(&mut world, &goal, commit, b"main").expect("repair accepted");
    let repair = produce_delegated(&mut world, answer, b"main", b"failures", b"");
    assert_eq!(change(&world, repair).repairs, Some(commit));
    assert!(matches!(change(&world, red).change.state, State::Red { .. }), "red awaits the next verdict");
    until(&mut world, RUN_TICKS, "repair run judged", |world| tier(world).record.sequence == 3);
    let runs = tier_runs::runs(&world);
    let run = runs.last().expect("repair run");
    assert_eq!(run.conclusion, Some(raw::Conclusion::Success));
    assert_eq!(translate::commit(run.commit), merge(&world, repair));
    cleared(&mut world, red);
    cleared(&mut world, repair);
    let hex = forge_top::commit_text(&merge(&world, repair));
    let short = std::str::from_utf8(&hex[..12]).expect("short repair commit");
    assert!(
        result(&world, red)
            .contains(&format!("Cleared by slow run 3, which passed the failures blamed on it; repaired by {short}.")),
        "{}",
        result(&world, red)
    );
    assert!(result(&world, repair).contains("Cleared by slow run 3."), "{}", result(&world, repair));
    eprintln!("seed={SEED}, simulated={:?}", world.env.now);
}

fn request_repair(
    world: &mut World,
    goal: &engine::Assignment,
    commit: skein_forge::api::Commit,
    branch: &[u8],
) -> Option<engine::CallAnswer> {
    let before = world.answers.len();
    let errors = world.call_errors.len();
    let event = world.protocol.call(
        Token::new(7),
        goal.task,
        goal.attempt,
        94,
        engine::Call {
            completion: 94,
            position: 1,
            tool: engine::Tool::Change {
                changes: Box::new([engine::ChangeCall {
                    landing: Some(engine::ChangeLanding::Branch(branch.into())),
                    repairs: Some(commit),
                    ..engine::ChangeCall::words(b"Repair the red landing".as_slice().into(), None)
                }]),
            },
        },
    );
    world.send(event);
    until(world, LANDING_TICKS, "repair call answered", |world| {
        world.answers.len() > before || world.call_errors.len() > errors
    });
    world.answers.get(before).cloned()
}
