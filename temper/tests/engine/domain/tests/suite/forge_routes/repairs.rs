//! A goal hears a red after landing and names it in its repair
//! (temper/docs/design/domain/after-landing.md, 7 and 12;
//! temper/docs/design/draft/reference-landing.md, 9).
use super::tiers::{baseline, change, cleared, merge, produce, produce_delegated, result, task, tier, until};
use super::*;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_forge_tiers::Failures;
use temper_engine_domain_world::tier_runs::{self, LANDING_TICKS, RUN_TICKS, SEED};

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
#[ignore = "repairs-call: temper-engine-domain/src/route/forge_route.rs chat_change/start_change drops repairs"]
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

#[test]
#[ignore = "repairs-call: temper-engine-domain/src/route/forge_route.rs change_batch does not reject repairs into another branch"]
fn a_repair_into_another_branch_is_refused() {
    let (mut world, goal, red) = red_goal();
    let commit = merge(&world, red);
    let tip = world.fake.branch(b"org/repo", b"main").expect("main tip");
    world.external(2, raw::Op::Git(raw::Git::Create { branch: b"release".as_slice().into(), commit: tip }));
    let answer = request_repair(&mut world, &goal, commit, b"release");
    assert!(answer.is_none(), "repair into another branch must be refused: {answer:?}");
    let digits = String::from_utf8(forge_top::commit_text(&commit).into_vec()).expect("commit digits");
    assert_eq!(
        world.call_errors.last().expect("repair refusal").text.as_ref(),
        format!("repairs: {digits} landed into main, not release.").as_bytes()
    );
    assert_eq!(world.merges, 1);
}
