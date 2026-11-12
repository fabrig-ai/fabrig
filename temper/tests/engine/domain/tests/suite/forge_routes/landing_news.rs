//! The member's goal hears every committed transition (domain/landing.md, 4.2).
use super::*;

fn produced_goal() -> (World, u64, u64) {
    let world = World::configured_limits(true, true, false, 1000, Gate::default(), None,
        StoryLimits { turn: true, ..StoryLimits::default() });
    let (mut world, goal, producer, _) = change_world_in(world, Change { tracked: true, ..Change::default() });
    world.send(hello_prepare(2));
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("change requester") };
    push_change(&mut world, &producer, b"org/repo");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"other", b"base moved", 2).expect("base moves");
    finish_producer(&mut world, &producer, forge_world::REPO);
    (world, goal.task, change)
}
fn hex(commit: &[u8; 32]) -> String {
    let width = if commit[20..] == [0; 12] { 20 } else { 32 };
    commit[..width].iter().map(|byte| format!("{byte:02x}")).collect()
}
fn news(world: &World, goal: u64, landing: u64) -> Vec<String> {
    let prefix = format!("landing {landing}: ");
    let mut lines: Vec<_> = world.store.rows.values().filter_map(|record| {
        let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(row))) = record else { return None };
        if row.task != goal { return None; }
        let tasks::Line::Heard(word) = &row.line else { return None };
        if !matches!(word.kind, tasks::MessageKind::News { .. }) || !word.words.starts_with(prefix.as_bytes()) { return None; }
        Some((row.position, String::from_utf8(word.words.to_vec()).expect("landing news")))
    }).collect();
    lines.sort_by_key(|(position, _)| *position);
    lines.into_iter().map(|(_, words)| words).collect()
}
fn joined(landing: u64, change: u64) -> String { format!("landing {landing}: change {change} joined, into org/repo main, round 1") }
fn proving(row: &forge_top::LandingRow, change: u64) -> String {
    let member = &row.landing.members[0];
    format!("landing {}: change {change} proving at {} on {}", row.landing.task, hex(&member.candidate.expect("candidate")), hex(&member.base.expect("base")))
}
#[test]
fn a_members_goal_hears_joined_proving_and_landed_in_order() {
    let (mut world, goal, change) = produced_goal();
    for _ in 0..800 { world.prepare_tick(); if task_row(&world, change).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_))) { break; } }
    let row = landing_rows(&world).into_iter().find(|row| row.landing.members[0].change == change).expect("landing");
    let landing = row.landing.task;
    let prefix = row.landing.members[0].landed.expect("landed prefix");
    assert_eq!(news(&world, goal, landing), [joined(landing, change), proving(row, change), format!("landing {landing}: change {change} landed at {}", hex(&prefix))]);
}
#[test]
fn a_members_goal_hears_joined_proving_and_left_and_the_result_names_why() {
    let (mut world, goal, change) = produced_goal();
    for _ in 0..600 {
        world.prepare_tick();
        if landing_rows(&world).iter().any(|row| row.landing.phase == temper_engine_domain_forge_landing::Phase::Proving) { break; }
    }
    let landing = landing_rows(&world).first().expect("proving landing").landing.task;
    cancel_task(&mut world, change);
    for _ in 0..600 { world.tick(); if ended(&world, landing) { break; } }
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert!(row.landing.members[0].landed.is_none());
    assert_eq!(news(&world, goal, landing), [joined(landing, change), proving(row, change), format!("landing {landing}: change {change} left: cancelled")]);
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) = &task_row(&world, landing).expect("ended landing").phase else { panic!("landing report") };
    assert!(String::from_utf8_lossy(words).contains(&format!("change {change} left: cancelled")));
}
#[test]
fn a_members_goal_hears_the_runner_bound_and_the_landing_holds_with_that_account() {
    let (mut world, goal, change) = produced_goal();
    world.fake_env.limits.setup_faults = 1000;
    for _ in 0..1600 {
        world.prepare_tick();
        if landing_rows(&world).iter().any(|row| matches!(row.landing.phase, temper_engine_domain_forge_landing::Phase::Held { .. })) { break; }
    }
    let row = landing_rows(&world).into_iter().next().expect("held landing");
    let landing = row.landing.task;
    let why = "gate gate of org/repo failed on its runner past its retries";
    assert!(matches!(row.landing.phase, temper_engine_domain_forge_landing::Phase::Held { why: temper_engine_domain_forge_landing::Hold::Faults { .. }, .. }));
    assert_eq!(news(&world, goal, landing), [joined(landing, change), proving(row, change), format!("landing {landing}: held: {why}")]);
    let tasks::Phase::Held { source: Some(source), .. } = &task_row(&world, landing).expect("held task").phase else { panic!("held task") };
    assert!(matches!(source, tasks::HoldSource::Account { words, .. } if words.as_ref() == why.as_bytes()));
}
