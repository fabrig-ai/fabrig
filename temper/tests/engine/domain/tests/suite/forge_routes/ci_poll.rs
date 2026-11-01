//! Pull discovery and CI wakes through the root, without hints or forced gathers.
use super::*;
use temper_engine_domain_forge_change as change_step;

fn status(world: &mut World, head: u64, state: raw::Check) {
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done,
    );
}

fn finish(world: &mut World, producer: &engine::Assignment) {
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: b"Change".as_slice().into(),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

fn change(world: &World) -> forge_top::ChangeRow {
    world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Change(row) => Some(row.clone()),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("registered change")
}

fn pending_change() -> (World, Box<[u8]>, u64) {
    let configured = World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { stall: Some(Duration::from_secs(30 * 60)), ..StoryLimits::default() },
    );
    let (mut world, _, producer, branch) = change_world_in(configured, Change::default());
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first", 1)
        .expect("producer pushed");
    status(&mut world, head, raw::Check::Pending);
    finish(&mut world, &producer);
    (world, branch, head)
}

fn assert_watched(world: &World) {
    let row = change(world);
    let Some(Record::Forge { row: names }) = world.store.rows.get(&Key::Forge(forge_top::Key::Names(row.task))) else {
        panic!("live change's names")
    };
    assert!(matches!(names.as_ref(), forge_top::Stored::Names { resources, .. }
        if resources.iter().any(|name| name.what == forge_top::What::Pull(row.pull.expect("discovered pull")))));
}

fn repair_by(world: &mut World, deadline: Time) -> engine::Assignment {
    let before = world.assigned.len();
    while world.env.now <= deadline && world.assigned.len() == before {
        world.tick();
    }
    assert!(world.assigned.len() > before, "CI repair starts within bound: {:?}", change(world));
    assert!(matches!(change(world).change.state, change_step::State::Repairing { why: change_step::Repair::Ci, .. }));
    assert_watched(world);
    world.assigned.last().expect("repair assignment").clone()
}

#[test]
fn both_heads_ci_failures_wake_their_change_without_hints() {
    let (mut world, branch, head) = pending_change();
    for _ in 0..300 {
        world.tick();
        if change(&world).pull.is_some() {
            break;
        }
    }
    assert!(change(&world).pull.is_some(), "root discovers the opened pull");
    for _ in 0..100 {
        world.tick();
    }
    status(&mut world, head, raw::Check::Failed);
    let allowance = world.env.limits.forge.client.poll_max.saturating_add(Duration::from_secs(5));
    let deadline = world.env.now.saturating_add(allowance);
    let repair = repair_by(&mut world, deadline);
    let second = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"second", 1)
        .expect("repair pushed");
    assert_ne!(head, second);
    status(&mut world, second, raw::Check::Pending);
    finish(&mut world, &repair);
    for _ in 0..300 {
        world.tick();
        if matches!(change(&world).change.state, change_step::State::Checking { head } if head == translate::commit(second))
        {
            break;
        }
    }
    assert!(
        matches!(change(&world).change.state, change_step::State::Checking { head } if head == translate::commit(second))
    );
    assert_watched(&world);
    status(&mut world, second, raw::Check::Failed);
    let deadline = world.env.now.saturating_add(allowance);
    repair_by(&mut world, deadline);
}

fn hold_reviews(world: &mut World) -> engine::Event {
    for _ in 0..500 {
        world.tick();
        if change(world).pull.is_some()
            && let Some(index) = world.events.iter().position(|event| {
                matches!(event,
                engine::Event::Forge(event) if matches!(event.as_ref(), forge_top::Event::Client(
                    skein_forge_client::Event::Answered { result: Ok(skein_forge::api::Answer::Reviews { .. }), .. }
                )))
            })
        {
            return world.events.remove(index).expect("held gather terminal");
        }
    }
    panic!("the keep's first pull observation starts a fresh gather");
}

fn fail_during_gather(world: &mut World, head: u64) -> engine::Assignment {
    let terminal = hold_reviews(world);
    assert_watched(world);
    status(world, head, raw::Check::Failed);
    // The keep observes the failure while the earlier pending-CI gather waits
    // for its final terminal. The root's scheduler coalesces an already-due wake.
    for _ in 0..200 {
        world.tick();
    }
    assert!(
        world.store.rows.values().any(|record| matches!(record,
        Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::PullState(row)
            if row.head == translate::commit(head) && row.ci == skein_forge::api::Ci::Failed))),
        "the keep saw failure while the gather was in flight"
    );
    assert_watched(world);
    world.send(terminal);
    let deadline = world.env.now.saturating_add(Duration::from_secs(5));
    repair_by(world, deadline)
}

#[test]
fn ci_progress_during_a_gather_is_not_lost_when_that_gather_waits() {
    let (mut world, branch, head) = pending_change();
    let repair = fail_during_gather(&mut world, head);
    let second = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"second", 1)
        .expect("repair pushed");
    status(&mut world, second, raw::Check::Pending);
    finish(&mut world, &repair);
    fail_during_gather(&mut world, second);
}
