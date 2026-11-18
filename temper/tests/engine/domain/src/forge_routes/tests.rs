//! Idle horizons retain the story's tick grid and every timer boundary.
use super::*;

fn tick_reference(world: &mut World) {
    world.changed_tasks.clear();
    world.env.now = world.env.now.saturating_add(world.tick_interval);
    world.sync_clock();
    world.tick_at_now(None, true, false);
    if world.auto_prepare {
        world.complete_prepare();
    }
}

fn same_observation(fixed: &World, jumped: &World) {
    assert_eq!(fixed.env.now, jumped.env.now);
    assert_eq!(fixed.env.wall, jumped.env.wall);
    assert_eq!(fixed.fake_env.now, jumped.fake_env.now);
    assert_eq!(fixed.fake.calls(), jumped.fake.calls());
    assert_eq!(fixed.fake.deliveries(), jumped.fake.deliveries());
    assert_eq!(fixed.store.applied, jumped.store.applied);
    assert_eq!(fixed.store.rows, jumped.store.rows);
    assert_eq!(fixed.store.pending, jumped.store.pending);
    assert_eq!(fixed.store.held, jumped.store.held);
    assert_eq!(format!("{:?}", fixed.events), format!("{:?}", jumped.events));
    assert_eq!(fixed.writes, jumped.writes);
    assert_eq!(fixed.forge_calls, jumped.forge_calls);
    assert_eq!(fixed.assigned, jumped.assigned);
    assert_eq!(fixed.jobs, jumped.jobs);
    assert_eq!(fixed.answers, jumped.answers);
    assert_eq!(fixed.delivered_messages, jumped.delivered_messages);
    assert_eq!(fixed.root.next_deadline(&fixed.env), jumped.root.next_deadline(&jumped.env));
}

#[test]
fn an_idle_horizon_keeps_its_end_and_both_sides_of_a_timer() {
    let mut fixed = World::new();
    let mut jumped = World::new();
    fixed.sign_in();
    jumped.sign_in();
    fixed.until(Until::Ready);
    jumped.until(Until::Ready);
    same_observation(&fixed, &jumped);
    assert!(fixed.root.quiescent());
    // The first timer falls off this tick grid: firing must still wait
    // for the first ordinary observation at or after it.
    fixed.tick_interval = Duration::from_secs(13);
    jumped.tick_interval = fixed.tick_interval;
    let due = fixed.root.next_deadline(&fixed.env).expect("session expiry armed");
    // Stories may move monotonic time directly; an ordinary tick synchronizes
    // the wall clock before consulting the root's wall timers.
    fixed.env.now = fixed.env.now.saturating_add(Duration::from_secs(1));
    jumped.env.now = fixed.env.now;
    fixed.root.drain_facts();
    jumped.root.drain_facts();
    let before = u32::try_from(
        due.as_nanos().saturating_sub(fixed.env.now.as_nanos()).saturating_sub(1) / fixed.tick_interval.as_nanos(),
    )
    .expect("bounded fixture horizon");
    let end = fixed.env.now.saturating_add(Duration::from_nanos(u64::from(before) * fixed.tick_interval.as_nanos()));
    for _ in 0..before {
        tick_reference(&mut fixed);
    }
    jumped.advance_ticks(before);
    assert_eq!(jumped.env.now, end, "silence is observed through the entire horizon");
    assert!(jumped.env.now < due);
    same_observation(&fixed, &jumped);
    let cadence = fixed.tick_interval;
    fixed.tick_interval = Duration::from_nanos(due.as_nanos().saturating_sub(fixed.env.now.as_nanos()));
    jumped.tick_interval = fixed.tick_interval;
    tick_reference(&mut fixed);
    jumped.advance_ticks(1);
    assert!(jumped.env.now >= due);
    assert_eq!(jumped.env.now, due, "a horizon ending on a timer still fires it");
    same_observation(&fixed, &jumped);
    fixed.tick_interval = cadence;
    jumped.tick_interval = cadence;
    // Drain the expiry's new work without ending the subsequent horizon early.
    for _ in 0..200 {
        tick_reference(&mut fixed);
    }
    jumped.advance_ticks(200);
    same_observation(&fixed, &jumped);
    fixed.restart_same();
    jumped.restart_same();
    for _ in 0..200 {
        tick_reference(&mut fixed);
    }
    jumped.advance_ticks(200);
    same_observation(&fixed, &jumped);
    // Calls answered between ticks still reclaim their slots at the next
    // observation, even when there is no root work or timer to fire.
    for _ in 0..=jumped.fake_env.limits.limits.calls {
        let branch = || raw::Op::Read(raw::Read::Branch { branch: b"main".as_slice().into() });
        assert_eq!(fixed.external(1, branch()), jumped.external(1, branch()));
        tick_reference(&mut fixed);
        jumped.advance_ticks(1);
        same_observation(&fixed, &jumped);
    }
    fixed.close();
    jumped.close();
}

fn ask(world: &mut World, key: u8) {
    let ask = world.chat(1, b"Wait for my next input");
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner"),
        key: [key; 16],
        ask,
    }));
}

fn running_chat() -> World {
    let mut world = World::new();
    world.sign_in();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    ask(&mut world, 92);
    world.until(Until::Assigned);
    world
}

#[test]
fn a_hosted_attempt_can_wait_but_new_owner_work_cannot_be_skipped() {
    let mut fixed = running_chat();
    let mut jumped = running_chat();
    for _ in 0..200 {
        tick_reference(&mut fixed);
    }
    jumped.advance_ticks(200);
    same_observation(&fixed, &jumped);
    let message = fixed.root.deployment().messages + 1;
    for world in [&mut fixed, &mut jumped] {
        let first = &world.assigned[0];
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(93)),
            sign_in: world.signed_in.expect("owner"),
            key: [93; 16],
            ask: people::Ask::Say { project: 1, task: first.task, words: b"Still working".as_slice().into() },
        }));
    }
    for _ in 0..200 {
        tick_reference(&mut fixed);
    }
    jumped.advance_ticks(200);
    same_observation(&fixed, &jumped);
    let run = &jumped.assigned[0];
    assert!(
        jumped.delivered_messages.get(&(run.task, run.attempt)).is_some_and(|words| words.contains(&message)),
        "the hosted attempt hears its person's new word"
    );
    fixed.close();
    jumped.close();
}
