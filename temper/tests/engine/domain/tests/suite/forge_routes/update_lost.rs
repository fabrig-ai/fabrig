//! Lost updates hold only at their bound, with the last write's account
//! (temper/docs/design/domain/forge/README.md, 6).
use super::early_start::{make, observe_push, push, stop};
use super::light_rules::change_row;
use super::reviews::next_assignment;
use super::update_first::{assert_landed, finish, goal_world_stall, move_base, status};
use super::*;
use skein_forge_client::Condition;
use temper_engine_domain_forge_change as change;

// At 100 ms a tick, this covers three updates of three ten-second write lifetimes,
// their fresh reads and durable routing, without changing the shared fixture's limits.
const STEPS: u32 = 3000;

fn update_writes(world: &World) -> usize {
    world.writes.iter().filter(|write| matches!(write, skein_forge::api::Write::Update { .. })).count()
}

fn pending_update(stall: Option<Duration>) -> (World, engine::Assignment, u64, u64, u64, Box<[u8]>) {
    let (mut world, goal, branch, main) = goal_world_stall(true, stall);
    let task = make(&mut world, &goal, b"Add change", &[]);
    let producer = next_assignment(&mut world, 1);
    let head = push(&mut world, &producer, main, b"change");
    status(&mut world, head, raw::Check::Pending);
    let base = move_base(&mut world, &branch);
    observe_push(&mut world, &producer);
    finish(&mut world, &producer);
    for _ in 0..STEPS {
        world.tick();
        assert_not_held(&world, task);
        if world.store.pending.front().is_some_and(|(_, writes)| {
            writes.iter().any(|write| {
                matches!(write,
            engine::Write::Save(Record::Forge { row }) if matches!(row.as_ref(),
                forge_top::Stored::Entry { entry, .. } if entry.task == task
                    && matches!(entry.effect.write, skein_forge::api::Write::Update { .. })
                    && entry.attempt.is_some()))
            })
        }) {
            break;
        }
    }
    assert!(matches!(change_row(&world, task).change.state, change::State::Updating { .. }));
    assert_eq!(update_writes(&world), 0, "loss injection precedes the update's send");
    (world, goal, task, head, base, branch)
}

fn assert_not_held(world: &World, task: u64) {
    assert!(!matches!(live_task(world, task).phase, tasks::Phase::Held { .. }), "{:?}", live_task(world, task).phase);
    assert!(!matches!(change_row(world, task).change.state, change::State::Held { .. }));
}

fn checking(world: &mut World, task: u64, before: u64, count: u32) -> u64 {
    for _ in 0..STEPS {
        world.tick();
        assert_not_held(world, task);
        if matches!(change_row(world, task).change.state, change::State::Checking { .. }) {
            break;
        }
    }
    let row = change_row(world, task);
    let change::State::Checking { head } = row.change.state else { panic!("updated head checking: {row:?}") };
    let updated = world.fake.branch(b"org/repo", &row.branch).expect("updated branch");
    assert_ne!(updated, before);
    assert_eq!(head, translate::commit(updated));
    assert_eq!(row.change.updates, count);
    updated
}

fn assert_contains(world: &mut World, head: u64, base: u64) {
    let raw::Answer::Comparison { contains_base, .. } =
        world.external(1, raw::Op::Read(raw::Read::Compare { base, head, page: 1, limit: 10 }))
    else {
        panic!("updated ancestry")
    };
    assert!(contains_base);
}

fn held(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, task).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    let row = change_row(world, task);
    assert!(
        matches!(row.change.state, change::State::Held { why: change::Hold::Updates, .. }),
        "state: {:?}; updates: {}; update writes: {}; now: {:?}; stall: {:?}; lifetime: {:?}; clock margin: {:?}; write attempts: {}",
        row.change.state,
        row.change.updates,
        update_writes(world),
        world.env.wall,
        world.env.limits.forge.change_policy.stall,
        world.env.limits.forge.client.lifetime,
        world.env.limits.forge.client.clock_margin,
        world.env.limits.forge.client.write_attempts
    );
    assert_eq!(row.change.updates, world.env.limits.forge.change_policy.updates);
    assert!(matches!(live_task(world, task).phase, tasks::Phase::Held { why: tasks::Hold::Procedure, .. }));
}

fn lost_for_good() -> (World, engine::Assignment, u64, u64) {
    let stall =
        temper_engine_service_vocabulary::limits::profile(temper_engine_service_vocabulary::limits::Profile::Small)
            .forge
            .change_policy
            .stall;
    let (mut world, goal, task, head, _, _) = pending_update(Some(stall));
    world.lose_writes = u32::MAX;
    held(&mut world, task);
    assert_eq!(
        update_writes(&world),
        usize::try_from(world.env.limits.forge.change_policy.updates * world.env.limits.forge.client.write_attempts)
            .expect("small write bound")
    );
    // The lost writes outlast the owner's sixty-second sign-in.
    world.signed_in = None;
    world.sign_in();
    (world, goal, task, head)
}

#[test]
fn a_lost_update_is_made_again_and_holds_no_one() {
    assert_eq!(
        forge_top::effect_shape(11),
        Some((forge_top::EffectForm::Transition, forge_top::Recovery::Idempotent)),
        "jig's class must permit the update's recovery"
    );
    let (mut world, goal, task, head, base, _) = pending_update(None);
    world.lose_writes = 1;
    let updated = checking(&mut world, task, head, 1);
    assert_eq!(update_writes(&world), 2);
    assert_contains(&mut world, updated, base);
    stop(&mut world, &goal);
}

#[test]
fn an_update_lost_for_good_holds_at_its_count_told_why() {
    let (mut world, goal, task, _) = lost_for_good();
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, task).phase else {
        panic!("updates hold needs the last write's account: {:?}", live_task(&world, task).phase)
    };
    let tasks::HoldSource::Account { connector, entry, words } = source.as_ref() else {
        panic!("updates hold needs an account")
    };
    let account = change_row(&world, task).effect_account.as_ref().expect("last lost write's account");
    assert_eq!(*connector, 0);
    assert_eq!(*entry, Some(account.entry));
    assert_eq!(*words, account.words);
    assert!(String::from_utf8_lossy(words).contains("update pull request: forge timed out"));
    world.lose_writes = 0;
    stop(&mut world, &goal);
}

#[test]
fn a_released_update_holds_no_longer() {
    let (mut world, goal, task, head) = lost_for_good();
    world.lose_writes = 0;
    let revision = live_task(&world, task).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [95; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    }));
    // The release's task row becomes visible at its durable commit.
    for _ in 0..STEPS {
        world.tick();
        if !matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    let updated = checking(&mut world, task, head, 1);
    status(&mut world, updated, raw::Check::Passed);
    assert_landed(&mut world, task, updated, 1);
    stop(&mut world, &goal);
}

#[test]
fn an_update_whose_base_went_on_while_lost_takes_the_new_tip() {
    let (mut world, goal, task, head, base, branch) = pending_update(None);
    world.lose_writes = 1;
    for _ in 0..STEPS {
        world.tick();
        assert_not_held(&world, task);
        if update_writes(&world) == 1 {
            break;
        }
    }
    assert_eq!(update_writes(&world), 1);
    let next_base = move_base(&mut world, &branch);
    for _ in 0..STEPS {
        world.tick();
        assert_not_held(&world, task);
        if change_row(&world, task).change.updates == 2
            && world.store.rows.values().any(|record| {
                matches!(record,
                Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Entry { entry, .. }
                    if entry.task == task && matches!(entry.effect.write, skein_forge::api::Write::Update { .. })))
            })
        {
            break;
        }
    }
    assert_eq!(change_row(&world, task).change.updates, 2);
    assert!(world.store.rows.values().any(|record| matches!(record,
    Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Entry { entry, .. }
        if entry.task == task && entry.effect.condition == Condition::Update {
            head: translate::commit(head), base: translate::commit(next_base)
        }))));
    let updated = checking(&mut world, task, head, 2);
    assert_contains(&mut world, updated, base);
    assert_contains(&mut world, updated, next_base);
    status(&mut world, updated, raw::Check::Passed);
    assert_landed(&mut world, task, updated, 2);
    stop(&mut world, &goal);
}

#[test]
fn an_updates_hold_with_no_lost_write_says_as_it_did() {
    let (mut world, goal, task, head, _, branch) = pending_update(None);
    let mut checked = translate::commit(head);
    for _ in 0..STEPS {
        world.tick();
        if let change::State::Checking { head } = change_row(&world, task).change.state
            && head != checked
        {
            checked = head;
            let updated = world.fake.branch(b"org/repo", &change_row(&world, task).branch).expect("updated branch");
            status(&mut world, updated, raw::Check::Passed);
            move_base(&mut world, &branch);
        }
        if matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    held(&mut world, task);
    assert_eq!(update_writes(&world), usize::try_from(world.env.limits.forge.change_policy.updates).expect("bound"));
    assert!(change_row(&world, task).effect_account.is_none());
    let tasks::Phase::Held { source, .. } = &live_task(&world, task).phase else { panic!("updates hold") };
    assert!(source.is_none(), "made updates leave the usual hold without an account");
    stop(&mut world, &goal);
}
