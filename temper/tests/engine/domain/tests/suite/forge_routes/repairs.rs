//! A goal's repair names a red landing and keeps it across restart
//! (temper/docs/design/domain/agent.md, 4.3.1; after-landing.md, 7).
use super::*;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_world::tier_runs::{self, LANDING_TICKS, RUN_TICKS, Tier, change, merge, restart_cut, until};

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
