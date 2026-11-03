//! Root stories for temper/docs/design/domain/after-landing.md, 3–6 and 12.
use super::*;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_forge_tiers as policy;
use temper_engine_domain_world::tier_runs::{
    self, LANDING_TICKS, RUN_TICKS, SEED, Tier, change, merge, produce_delegated, restart_cut, task, tier, until,
};

fn chat_world(turn: bool, options: Tier) -> (World, engine::Assignment) {
    let mut world = tier_runs::world(turn, options);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { unreachable!() };
    // One chat funds four producers while their landed changes remain live.
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    (world, chat)
}

pub(super) fn baseline(world: &mut World) {
    until(world, RUN_TICKS, "baseline judged", |world| tier(world).record.sequence == 1);
    // The 90-second baseline outlives the fixture's 60-second sign-in.
    sign_in(world);
}

#[test]
fn a_tier_task_names_the_execution_it_dispatched() {
    let (mut world, _) = chat_world(false, Tier::default());
    until(&mut world, LANDING_TICKS, "baseline dispatched", |world| {
        world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Dispatch { .. }))
    });
    let row = tier(&world);
    let run = row.record.run.as_ref().expect("baseline running");
    let words = std::str::from_utf8(&task(&world, row.task.expect("tier task")).spec.words).expect("task words");
    assert!(words.contains("org/repo"), "repository missing from task words: {words}");
    assert!(words.contains("temper/tier/slow.1"), "owned branch missing from task words: {words}");
    let commit = String::from_utf8(forge_top::commit_text(&run.tip).into_vec()).expect("full forge commit");
    assert!(words.contains(&commit), "tested commit missing from task words: {words}");
}

fn sign_in(world: &mut World) {
    let valid = world.store.rows.values().any(|row| {
        matches!(row,
        Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { number, expires, .. }))
            if Some(*number) == world.signed_in && *expires > world.env.wall)
    });
    if !valid {
        world.signed_in = None;
        world.sign_in();
    }
}

pub(super) fn produce(world: &mut World, chat: &engine::Assignment, call: u64, path: &[u8], content: &[u8]) -> u64 {
    let answer = change_tool::change(world, chat, call, None, b"Small fix");
    produce_delegated(world, answer, b"main", path, content)
}

fn panel(world: &mut World, number: u64) -> forge_top::PanelStep {
    sign_in(world);
    let watch = Token::new(world.env.now.as_nanos());
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher: watch,
        sign_in: world.signed_in.expect("owner signed in"),
        subject: jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Project(1),
            connector: 0,
            subject: Token::new(number),
        },
        window: jig_domain_core::documents::Window { rows: 0, after: None },
    }));
    until(world, LANDING_TICKS, "panel read", |world| world.panels.iter().any(|(watcher, _)| *watcher == watch));
    let step = world.panels.iter().rev().find(|(watcher, _)| *watcher == watch).expect("panel").1.step.clone();
    world.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher: watch }));
    step
}

fn verifying(world: &mut World, number: u64) {
    assert!(!matches!(task(world, number).phase, tasks::Phase::Ended(_)), "no result before verification");
    assert!(matches!(panel(world, number), forge_top::PanelStep::Verifying { .. }));
}

pub(super) fn result(world: &World, number: u64) -> &str {
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) = &task(world, number).phase
    else {
        panic!("change {number} done: {:?}", task(world, number).phase)
    };
    std::str::from_utf8(words).expect("result words")
}

pub(super) fn cleared(world: &mut World, number: u64) {
    until(world, RUN_TICKS * 3, "change cleared", |world| matches!(task(world, number).phase, tasks::Phase::Ended(_)));
    assert!(matches!(change(world, number).change.state, State::Cleared { .. }));
    assert!(matches!(panel(world, number), forge_top::PanelStep::Cleared { uncleared: false, .. }));
    assert!(result(world, number).contains("Landed:"), "result is given at cleared");
}

fn cleared_result(world: &World, number: u64, run: u64, repaired: bool) {
    let words = result(world, number);
    let cleared = if repaired {
        format!("Cleared by slow run {run}, which passed the failures blamed on it.")
    } else {
        format!("Cleared by slow run {run}.")
    };
    assert!(
        words.find("Landed:").zip(words.find(&cleared)).is_some_and(|(landed, cleared)| landed < cleared),
        "seed={SEED}, simulated={:?}: expected {cleared:?} after Landed:, got {words:?}",
        world.env.now
    );
}

fn tier_done(world: &World, number: u64, expected: &str) {
    let words: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
                if matches!(row.executor, tasks::Executor::Procedure { connector: 0, code: 4 }) =>
            {
                let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) = &row.phase else {
                    return None;
                };
                Some(std::str::from_utf8(words).expect("tier words"))
            }
            _ => None,
        })
        .collect();
    assert!(words.contains(&expected), "tier run {number}: {words:?}");
}

fn green(turn: bool) -> (World, u64) {
    let (mut world, chat) = chat_world(turn, Tier::default());
    baseline(&mut world);
    let number = produce(&mut world, &chat, 93, b"file", b"small fix");
    verifying(&mut world, number);
    until(&mut world, RUN_TICKS, "tier judged", |world| tier(world).record.sequence == 2);
    tier_done(&world, 2, "Tier slow run 2: green, cleared 1 landings");
    cleared(&mut world, number);
    eprintln!("seed={SEED}, simulated={:?}", world.env.now);
    (world, number)
}

#[test]
fn a_chats_change_waits_for_main_to_clear_it() {
    let _ = green(false);
}

#[test]
fn two_landings_during_one_run_clear_in_the_next() {
    let _ = two_landings();
}

fn two_landings() -> (World, [u64; 2]) {
    let (mut world, chat) = chat_world(false, Tier::default());
    let a = produce(&mut world, &chat, 93, b"file", b"first fix");
    verifying(&mut world, a);
    let b = produce(&mut world, &chat, 94, b"file", b"second fix");
    verifying(&mut world, b);
    assert_eq!(tier(&world).record.run.as_ref().expect("baseline running").number, 1);
    until(&mut world, RUN_TICKS, "next run covers both", |world| {
        tier(world).record.run.as_ref().is_some_and(|run| run.number == 2)
    });
    let run = tier(&world).record.run.as_ref().expect("range run");
    assert_eq!(
        run.landings.iter().map(|landing| landing.commit).collect::<Vec<_>>(),
        [merge(&world, a), merge(&world, b)]
    );
    until(&mut world, RUN_TICKS, "range judged", |world| tier(world).record.sequence == 2);
    tier_done(&world, 2, "Tier slow run 2: green, cleared 2 landings");
    cleared(&mut world, a);
    cleared(&mut world, b);
    eprintln!("seed={SEED}, simulated={:?}", world.env.now);
    (world, [a, b])
}

fn red_range() -> (World, engine::Assignment, [u64; 3]) {
    let (mut world, chat) = chat_world(false, Tier::default());
    // The first key equals the baseline; the second introduces both failures.
    let a = produce(&mut world, &chat, 93, b"docs", b"only documentation");
    verifying(&mut world, a);
    let b = produce(&mut world, &chat, 94, b"failures", tier_runs::FAILURES);
    verifying(&mut world, b);
    let c = produce(&mut world, &chat, 95, b"file", b"unrelated edit");
    verifying(&mut world, c);
    assert_eq!(tier(&world).record.run.as_ref().expect("baseline running").number, 1);
    until(&mut world, RUN_TICKS, "next run covers three landings", |world| {
        tier(world).record.run.as_ref().is_some_and(|run| run.number == 2)
    });
    assert_eq!(
        tier(&world)
            .record
            .run
            .as_ref()
            .expect("red range")
            .landings
            .iter()
            .map(|landing| landing.commit)
            .collect::<Vec<_>>(),
        [merge(&world, a), merge(&world, b), merge(&world, c)]
    );
    let run_task = tier(&world).task.expect("range task");
    let runs = observe_range(&mut world, run_task);
    until(&mut world, LANDING_TICKS, "blamed change red", |world| {
        matches!(change(world, b).change.state, State::Red { .. })
    });
    assert!(!matches!(task(&world, b).phase, tasks::Phase::Ended(_)));
    let forge_top::PanelStep::Red { tiers, .. } = panel(&mut world, b) else { panic!("red panel") };
    assert_eq!(tiers.as_ref(), [Box::<[u8]>::from("slow run 2 · 2 tests".as_bytes())]);
    assert_eq!(
        tier(&world).record.failing,
        policy::Failures::Tests(Box::new([b"suite.alpha".as_slice().into(), b"suite.beta".as_slice().into(),]))
    );
    red_dispatches(&world, merge(&world, b));
    red_task_words(&world, run_task, merge(&world, c), merge(&world, b), runs);
    cleared(&mut world, a);
    cleared(&mut world, c);
    eprintln!("seed={SEED}, simulated={:?}", world.env.now);
    (world, chat, [a, b, c])
}

#[test]
fn a_red_range_reruns_and_skips_the_unchanged_landing() {
    let _ = red_range();
}

#[test]
fn a_later_run_passing_the_blamed_tests_repairs_red() {
    let _ = repaired();
}

fn repaired() -> (World, [u64; 2]) {
    let (mut world, chat, [_, red, _]) = red_range();
    let repair = produce(&mut world, &chat, 96, b"failures", b"");
    verifying(&mut world, repair);
    assert!(
        matches!(panel(&mut world, red), forge_top::PanelStep::Red { .. }),
        "red survives until the repair run judges"
    );
    until(&mut world, RUN_TICKS, "repair judged", |world| tier(world).record.sequence == 3);
    tier_done(&world, 3, "Tier slow run 3: green, cleared 1 landings");
    cleared(&mut world, red);
    cleared(&mut world, repair);
    eprintln!("seed={SEED}, simulated={:?}", world.env.now);
    (world, [red, repair])
}

#[test]
fn a_silent_runner_past_the_bound_ends_landed_uncleared() {
    const BOUND_MS: u64 = 60_000; // This story exercises the project's verification timeout itself.
    let (mut world, chat) = chat_world(false, Tier { silent: true, bound_ms: BOUND_MS });
    let number = produce(&mut world, &chat, 93, b"file", b"small fix");
    verifying(&mut world, number);
    let State::Landed { since, .. } = change(&world, number).change.state else { panic!("verifying") };
    until(&mut world, RUN_TICKS, "verification bound", |world| {
        matches!(task(world, number).phase, tasks::Phase::Ended(_))
    });
    assert!(world.env.wall.as_nanos() >= since.as_nanos() + Duration::from_millis(BOUND_MS).as_nanos());
    assert!(matches!(change(&world, number).change.state, State::Uncleared { .. }));
    assert!(matches!(panel(&mut world, number), forge_top::PanelStep::Cleared { uncleared: true, .. }));
    assert!(
        result(&world, number).contains("Not cleared: no verdict within the verification bound."),
        "{:?}",
        result(&world, number)
    );
    assert_eq!(tier(&world).record.sequence, 0, "silent workflow gives no verdict");
    assert!(tier(&world).record.run.is_some());
    let run = task(&world, tier(&world).task.expect("live tier task"));
    assert!(matches!(run.phase, tasks::Phase::Active(_)));
    assert!(run.spec.words.starts_with(b"Tier slow run 1\n"));
    eprintln!("seed={SEED}, simulated={:?}", world.env.now);
}

#[test]
fn a_change_landed_by_turn_waits_for_main_to_clear_it() {
    let _ = green(true);
}

#[test]
#[ignore = "tiers-root: forge_route.rs tier_landing_run loses the completed tier task; cleared results omit run N"]
fn a_cleared_chat_result_names_its_tier_run() {
    let (world, number) = green(false);
    cleared_result(&world, number, 2, false);
}

#[test]
#[ignore = "tiers-root: forge_route.rs tier_landing_run loses completed ranges; both cleared results must name run N"]
fn both_landing_results_name_the_run_that_cleared_them() {
    let (world, numbers) = two_landings();
    for number in numbers {
        cleared_result(&world, number, 2, false);
    }
}

#[test]
#[ignore = "tiers-root: forge_route.rs tier_landing_run loses completed ranges; unblamed changes must name run N"]
fn unblamed_results_name_the_red_run_that_cleared_them() {
    let (world, _, [a, _, c]) = red_range();
    for number in [a, c] {
        cleared_result(&world, number, 2, false);
    }
}

#[test]
#[ignore = "temper-engine-domain/src/route/forge_route.rs tier_landing_run loses completed ranges; change_verification_words omits the repairing run and passed-failures wording"]
fn a_repaired_result_names_the_run_that_passed_its_failures() {
    let (world, [red, repair]) = repaired();
    cleared_result(&world, red, 3, true);
    cleared_result(&world, repair, 3, false);
}

#[test]
#[ignore = "tiers-root: forge_route.rs tier_landing_run loses the completed tier task for a by-turn change"]
fn a_cleared_by_turn_result_names_its_tier_run() {
    let (world, number) = green(true);
    cleared_result(&world, number, 2, false);
}

#[test]
fn a_change_into_a_goals_unfollowed_branch_ends_at_landed() {
    const BASE: &[u8] = b"temper/1/g1";
    let (mut world, chat) = chat_world(false, Tier::default());
    baseline(&mut world);
    world.external(1, raw::Op::Git(raw::Git::Create { branch: BASE.into(), commit: 1 }));
    let mut delegate = temper_engine_domain_world::fixtures::report_delegate(b"Small fix", Box::new([]));
    delegate.executor = tasks::Executor::Procedure { connector: 0, code: 2 };
    delegate.contract = tasks::Contract::Change { connector: 0, kind: 1, words: 256 };
    delegate.authority = task(&world, chat.task).authority.clone();
    delegate.authority.delegation.depth = delegate.authority.delegation.depth.saturating_sub(1);
    delegate.authority.delegation.tasks = 6;
    delegate.authority.budget.spend = 40;
    delegate.spec.parameters = Box::new([
        tasks::Parameter::Resource {
            name: 1,
            connector: 0,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        },
        tasks::Parameter::Bytes { name: 2, value: BASE.into() },
    ]);
    let before = world.answers.len();
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        93,
        engine::Call { completion: 93, position: 1, tool: engine::Tool::Delegate { batch: Box::new([delegate]) } },
    );
    world.send(event);
    until(&mut world, LANDING_TICKS, "change delegated into goal branch", |world| world.answers.len() > before);
    let answer = world.answers[before].clone();
    let number = produce_delegated(&mut world, answer, BASE, b"file", b"goal edit");
    until(&mut world, LANDING_TICKS, "unfollowed change done", |world| {
        matches!(task(world, number).phase, tasks::Phase::Ended(_))
    });
    assert_eq!(panel(&mut world, number), forge_top::PanelStep::Landed { merge: merge(&world, number) });
    let words = result(&world, number);
    assert!(words.contains("merged into temper/1/g1 as"), "{words}");
    assert!(!words.contains("Cleared by"), "unfollowed branch keeps its landed result");
    assert_eq!(world.fake.branch(b"org/repo", b"main"), Some(1));
    let goal = world.fake.branch(b"org/repo", BASE).expect("goal tip");
    assert_eq!(translate::commit(goal), merge(&world, number));
    assert_eq!(tier(&world).record.sequence, 1);
    assert!(tier(&world).record.run.is_none());
    let workflows: Vec<_> = world
        .writes
        .iter()
        .filter_map(|write| match write {
            skein_forge::api::Write::Dispatch { workflow, .. } => Some(workflow.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(workflows, [b"slow.yml".as_slice()], "the goal branch starts no tier run");
    tier_done(&world, 1, "Tier slow run 1: green, cleared 0 landings");
    eprintln!("seed={SEED}, simulated={:?}", world.env.now);
}

fn dispatched_runs(world: &World) -> Vec<skein_fake_forge_domain::api::Run> {
    let raw::Answer::Runs(runs) = world
        .fake
        .inspect(
            &world.fake_env.limits,
            b"org/repo",
            &raw::Read::Runs {
                workflow: Some(b"slow.yml".as_slice().into()),
                commit: None,
                number: None,
                page: 1,
                limit: 100,
            },
        )
        .expect("forge runs")
    else {
        panic!("forge runs")
    };
    runs.into_vec().into_iter().filter(|run| run.event == raw::RunEvent::Dispatch).collect()
}

fn execution_words(world: &World, number: u64) -> &str {
    std::str::from_utf8(&task(world, number).spec.words).expect("execution words")
}

#[test]
fn a_running_tier_task_keeps_the_forge_link_through_completion_and_restart() {
    let (mut world, _) = chat_world(false, Tier::default());
    until(&mut world, LANDING_TICKS, "forge link projected while running", |world| {
        tier(world).task.is_some_and(|number| execution_words(world, number).contains("forge run "))
            && restart_cut(world)
    });
    let number = tier(&world).task.expect("running tier task");
    let run = dispatched_runs(&world).into_iter().next().expect("baseline dispatch");
    assert_eq!(run.state, raw::RunState::Running);
    let before = execution_words(&world, number).to_owned();
    assert!(before.contains(&format!("forge run {} http://forge/org/repo/actions/runs/{}", run.number, run.number)));
    world.restart_same();
    until(&mut world, RUN_TICKS, "baseline completed after restart", |world| {
        tier(world).record.sequence == 1
            && restart_cut(world)
            && matches!(task(world, number).phase, tasks::Phase::Ended(_))
    });
    assert!(execution_words(&world, number).contains(&before));
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::DeleteBranch { branch } if branch.as_ref() == b"temper/tier/slow.1")));
    let completed = task(&world, number).clone();
    world.restart_same();
    for _ in 0..LANDING_TICKS {
        tier_runs::tick(&mut world);
    }
    assert_eq!(task(&world, number).spec, completed.spec);
}

fn assert_part_words(
    world: &World,
    number: u64,
    label: &str,
    branch: &[u8],
    commit: skein_forge::api::Commit,
    forge_run: u64,
) {
    let words = execution_words(world, number);
    let commit = String::from_utf8(forge_top::commit_text(&commit).into_vec()).expect("full forge commit");
    let branch = std::str::from_utf8(branch).expect("branch");
    let expected = format!(
        "{label}: commit {commit}, branch {branch}, forge run {forge_run} http://forge/org/repo/actions/runs/{forge_run}"
    );
    assert!(words.contains(&expected), "missing {expected}: {words}");
}

fn observe_range(world: &mut World, task: u64) -> Vec<raw::Run> {
    let mut written = 0;
    let mut runs = Vec::<raw::Run>::new();
    for _ in 0..RUN_TICKS * 5 {
        assert_dispatch_words(world, task, &world.writes[written..]);
        written = world.writes.len();
        for run in dispatched_runs(world) {
            if !runs.iter().any(|seen| seen.id == run.id) {
                runs.push(run);
            }
        }
        if tier(world).record.sequence == 2 {
            return runs;
        }
        tier_runs::tick(world);
    }
    panic!("red range did not finish");
}

fn red_dispatches(world: &World, commit: skein_forge::api::Commit) {
    let dispatches: Vec<_> = world
        .writes
        .iter()
        .filter_map(|write| match write {
            skein_forge::api::Write::Dispatch { workflow, branch, inputs } if workflow.as_ref() == b"slow.yml" => {
                Some((
                    branch.as_ref(),
                    inputs
                        .iter()
                        .find(|input| input.name.as_ref() == b"temper_tests")
                        .map(|input| input.value.as_ref()),
                ))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        dispatches,
        [
            (b"temper/tier/slow.1".as_slice(), None),
            (b"temper/tier/slow.2".as_slice(), None),
            (b"temper/tier/slow.2".as_slice(), Some(tier_runs::FAILURES)),
            (b"temper/tier/slow.2.1".as_slice(), Some(tier_runs::FAILURES)),
        ],
        "whole, confirming rerun, then oldest changed landing; documentation skipped"
    );
    let probes: Vec<_> = world
        .writes
        .iter()
        .filter_map(|write| match write {
            skein_forge::api::Write::CreateBranch { branch, commit } if branch.as_ref() == b"temper/tier/slow.2.1" => {
                Some(*commit)
            }
            _ => None,
        })
        .collect();
    assert_eq!(probes, [commit], "the second landing owns both tests");
}

fn red_task_words(
    world: &World,
    run_task: u64,
    tip: skein_forge::api::Commit,
    probe: skein_forge::api::Commit,
    mut runs: Vec<raw::Run>,
) {
    runs.sort_by_key(|run| run.number);
    assert!(runs.len() >= 3);
    let runs = &runs[runs.len() - 3..];
    for (run, label, branch, commit) in [
        (&runs[0], "Judged run", b"temper/tier/slow.2".as_slice(), tip),
        (&runs[1], "Failed-test rerun", b"temper/tier/slow.2".as_slice(), tip),
        (&runs[2], "Bisection probe", b"temper/tier/slow.2.1".as_slice(), probe),
    ] {
        assert_part_words(world, run_task, label, branch, commit, run.number);
    }
    tier_done(world, 2, "Tier slow run 2: red, 2 tests failing, blamed on 1 landings");
}

fn assert_dispatch_words(world: &World, task: u64, writes: &[skein_forge::api::Write]) {
    for write in writes {
        let skein_forge::api::Write::Dispatch { workflow, branch, inputs } = write else { continue };
        if workflow.as_ref() != b"slow.yml" || !branch.starts_with(b"temper/tier/slow.2") {
            continue;
        }
        let label = if branch.as_ref() != b"temper/tier/slow.2" {
            "Bisection probe"
        } else if inputs.iter().any(|input| input.name.as_ref() == b"temper_tests") {
            "Failed-test rerun"
        } else {
            "Judged run"
        };
        let commit =
            &inputs.iter().find(|input| input.name.as_ref() == b"temper_commit").expect("dispatch commit").value;
        let commit = std::str::from_utf8(commit).expect("commit text");
        let branch = std::str::from_utf8(branch).expect("branch text");
        let expected = format!("{label}: commit {commit}, branch {branch}");
        assert!(execution_words(world, task).contains(&expected), "dispatch left before its task named {expected}");
    }
}
