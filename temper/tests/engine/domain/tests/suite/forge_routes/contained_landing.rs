//! A head contained from its turn has landed (domain/forge/change/README.md, 2).
use super::catch_up::{catch_up, finish_job, goal_world_with_limits, next_job, push_merge};
use super::goal_branch::branch_tip;
use super::light_rules::change_row;
use super::update_first::finish;
use super::*;
use temper_engine_domain_forge_change as change;

// At 100 ms a tick, covers the client's ten-second write lifetime and durable release.
const STEPS: u32 = 1000;
const CHECKING: Duration = Duration::from_secs(20);

fn first(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        if matches!(change_row(world, task).change.state, change::State::First { .. }) {
            return;
        }
        if let Some(number) = change_row(world, task).pull {
            fake::check(&mut world.fake, &world.fake_env, b"org/repo", number, CHECKING);
        }
        world.tick();
    }
    panic!("change did not reach its turn: {:?}", change_row(world, task));
}

fn at_first(stall: Option<Duration>) -> (World, u64, u64) {
    let mut world = World::configured_limits(true, true, false, STEPS, Gate::default(), None,
        StoryLimits { stall, result_bytes: Some(1024), inbox_bytes: Some(4096), ..StoryLimits::default() });
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    let engine::CallAnswer::Delegated(made) = change_tool::change(&mut world, &chat, 93, None, b"Small fix") else { panic!("change") };
    let task = made[0].task;
    let producer = reviews::next_assignment(&mut world, 1);
    let branch = change_row(&world, task).branch.clone();
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"change", 2)
        .expect("producer pushed");
    world.fake_env.limits.settle_min = CHECKING;
    world.fake_env.limits.settle_max = CHECKING;
    finish(&mut world, &producer);
    first(&mut world, task);
    assert_eq!(world.merges, 0);
    (world, task, head)
}

fn push_base(world: &mut World, task: u64, head: u64) -> u64 {
    let base = change_row(world, task).base.clone();
    let tip = branch_tip(world, &base);
    let merged = fake::merge_commit(
        &mut world.fake,
        &world.fake_env.limits,
        tip,
        head,
        Box::new([raw::File { path: b"file".as_slice().into(), content: b"change".as_slice().into() }]),
        b"Merge outside the forge",
    ).expect("merge holding the change head");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Push { branch: base, commit: merged, expected: Some(tip) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    merged
}

fn landed(world: &mut World, task: u64, merged: u64) -> u32 {
    for _ in 0..STEPS {
        if matches!(change_row(world, task).change.state, change::State::Landed { .. }) {
            break;
        }
        world.tick();
    }
    let row = change_row(world, task);
    assert!(matches!(row.change.state, change::State::Landed { merge, .. } if merge == translate::commit(merged)), "{row:?}");
    assert!(!row.change.owns_turn);
    assert_eq!(row.pending, None, "the merge no longer belongs to the change's step");
    let tasks::Party::Task(requester) = live_task(world, task).requester else { panic!("requester's task") };
    let calls = world.merges;
    world.lose_writes = 0; // The merge was the fault; resource release can complete.
    for _ in 0..STEPS {
        if live_task(world, requester).inbox.iter().any(|message|
            message.from == tasks::Party::Task(task) && matches!(message.kind, tasks::MessageKind::Result(_))) { break; }
        world.tick();
    }
    let row = change_row(world, task);
    let number = row.pull.expect("retained pull");
    let digits = forge_top::commit_text(&translate::commit(merged));
    let expected = format!("Landed: pull request #{number} merged into {} as {}.",
        core::str::from_utf8(&row.base).expect("base"), core::str::from_utf8(&digits[..12]).expect("short commit"));
    let results: Vec<_> = live_task(world, requester).inbox.iter().filter(|message|
        message.from == tasks::Party::Task(task) && matches!(message.kind, tasks::MessageKind::Result(_))).collect();
    assert_eq!(results.len(), 1, "one terminal result");
    assert_eq!(core::str::from_utf8(&results[0].words).expect("result words").lines().nth(2), Some(expected.as_str()), "{:?}", results[0]);
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else { panic!("pull") };
    assert_eq!(pull.merged, None, "manual merge recognition is off");
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Close { number: closed } if *closed == number)), "no close is sent for the forge's open pull");
    assert_eq!(world.merges, calls, "no merge follows the containment step");
    calls
}

#[test]
fn a_base_pushed_to_hold_the_head_before_its_merge_lands_without_a_merge_call() {
    let (mut world, task, head) = at_first(None);
    let merged = push_base(&mut world, task, head);
    landed(&mut world, task, merged);
    assert_eq!(world.merges, 0);
    world.close();
}

#[test]
fn a_catch_up_whose_merge_is_retrying_lands_at_the_goal_branchs_pushed_tip() {
    let (mut world, goal, _, head) = goal_world_with_limits(true, true, true, &StoryLimits { result_bytes: Some(1024), inbox_bytes: Some(4096), ..StoryLimits::default() });
    let main = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"main", b"main work", 1)
        .expect("main moved");
    let task = catch_up(&mut world, &goal, main);
    let job = next_job(&mut world, task);
    let head = push_merge(&mut world, task, head, main);
    world.fake_env.limits.settle_min = CHECKING;
    world.fake_env.limits.settle_max = CHECKING;
    finish_job(&mut world, &job, b"{\"left\":[]}");
    first(&mut world, task);
    let number = change_row(&world, task).pull.expect("catch-up pull");
    fake::check(&mut world.fake, &world.fake_env, b"org/repo", number, Duration::ZERO);
    world.lose_writes = u32::MAX;
    for _ in 0..STEPS {
        world.tick();
        if world.merges >= 2 { break; }
    }
    assert!(world.merges >= 2, "the forge client retried the merge write");
    assert!(matches!(change_row(&world, task).change.state, change::State::Landing { .. }));
    let merged = push_base(&mut world, task, head);
    let calls = landed(&mut world, task, merged);
    // The outstanding attempt may settle, but must not restart or hold the landed change.
    for _ in 0..200 { world.tick(); }
    assert_eq!(world.merges, calls, "no merge follows the containment step");
    assert!(matches!(goal_landing::ended(&world, task).expect("landed task ended").phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    world.lose_writes = 0;
    world.close();
}

#[test]
fn a_failed_merge_released_after_its_base_holds_its_head_lands_without_resending() {
    let stall = temper_engine_service_vocabulary::limits::profile(temper_engine_service_vocabulary::limits::Profile::Small).forge.change_policy.stall;
    let (mut world, task, head) = at_first(Some(stall));
    let number = change_row(&world, task).pull.expect("pull");
    fake::check(&mut world.fake, &world.fake_env, b"org/repo", number, Duration::ZERO);
    world.lose_writes = u32::MAX;
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }) { break; }
    }
    assert!(matches!(&change_row(&world, task).change.state,
        change::State::Held { was, why: change::Hold::Failed } if matches!(was.as_ref(), change::State::Landing { .. })), "{:?}", change_row(&world, task));
    let merged = push_base(&mut world, task, head);
    let calls = world.merges;
    world.lose_writes = 0;
    world.signed_in = None;
    world.sign_in();
    let revision = live_task(&world, task).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]), reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner"), key: [95; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    }));
    landed(&mut world, task, merged);
    assert_eq!(world.merges, calls);
    world.close();
}
