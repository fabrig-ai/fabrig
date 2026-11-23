//! No stall, repair or run clock counts a worker's stop
//! (jig/docs/design/domain/engine/executors.md, 6.3 and 6.6).
use super::*;
use temper_engine_domain_forge_change::{Delegate, State};

const CHANNEL: Token = Token::new(7);
const RETURNED: Token = Token::new(8);
const STALL: Duration = Duration::from_secs(30);
const WINDOW: Duration = Duration::from_secs(600);
const GAP: Duration = Duration::from_secs(90);
const CHECK: Option<Duration> = Some(Duration::from_secs(900));
const STEPS: u32 = 1000;

fn unblamed(world: &World, owner: u64, change: u64, producer: u64) {
    assert_eq!(questions::row(world, change).delegate, Some((producer, Delegate::Produce)));
    assert_eq!(questions::row(world, change).change.state, State::Producing { requested: true });
    assert_eq!(live_task(world, change).delegates.as_ref(), &[producer], "no repair task");
    assert_eq!(live_task(world, producer).tries, tasks::Tries::NONE);
    for task in [owner, change, producer] {
        let row = live_task(world, task);
        assert!(matches!(row.phase, tasks::Phase::Active(_)), "task {task}: {:?}", row.phase);
        assert!(matches!(row.escalation, tasks::Escalation::Unheld { .. }));
        assert_eq!(row.revisions.hold, 0, "no hold was placed on {task}");
        assert!(
            !row.inbox
                .iter()
                .any(|word| matches!(word.kind, tasks::MessageKind::Result(_) | tasks::MessageKind::Escalation { .. })),
            "no failure sent to {task}"
        );
    }
    let no_findings: [&forge_top::Finding; 0] = [];
    assert_eq!(red_findings::findings(world), no_findings);
    assert!(!world.store.rows.values().any(|record| matches!(record,
        Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Red(_) | forge_top::Stored::RedPass(_)))));
}

#[test]
fn a_suspended_producer_waits_past_the_stall_without_blame_and_restarts_with_whole_clocks() {
    let world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { stall: Some(STALL), ..StoryLimits::default() },
    )
    .with_restart_wait(WINDOW, false)
    .with_check_timeout(CHECK);
    let (mut world, owner, producer, branch) = change_world_in(world, Change::default());
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("producer's change") };
    world.until(Until::Ready);
    assert_eq!(producer.check_timeout, CHECK);
    assert!(producer.run.policy.time > Duration::ZERO);
    // Some of the first run's time has elapsed when its worker stops.
    stall_hold::jump(&mut world, Duration::from_secs(10));
    world.send(protocol::up(CHANNEL, host::LinkUp::Draining, engine::Parts::none()));
    world.send(protocol::shutdown(CHANNEL, producer.task, producer.attempt, 1));
    world.advance_ticks(40);
    let suspended = live_task(&world, producer.task).suspension.expect("worker shutdown reaches the tasks child");
    assert_eq!(suspended.worker, 1);
    assert_eq!(suspended.until.as_nanos() - suspended.since.as_nanos(), WINDOW.as_nanos());
    assert!(world.store.rows.values().any(|record| matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Attempt(row)))
        if row.task == producer.task && row.attempt == producer.attempt
            && row.end == Some(tasks::EndKind::Suspended { worker: 1, why: tasks::SuspendWhy::Stopped }))));
    let assigned = world.assigned.len();
    world.jump(GAP);
    early_start::observe_push(&mut world, &producer);
    // Complete a fresh change step past the stall, not only the timer's wake.
    let past = suspended.since.as_nanos() + GAP.as_nanos();
    for _ in 0..STEPS {
        world.tick();
        unblamed(&world, owner.task, change, producer.task);
        if questions::row(&world, change).change.since.as_nanos() >= past {
            break;
        }
    }
    assert!(questions::row(&world, change).change.since.as_nanos() >= past, "the change refreshed its stall");
    assert!(world.env.wall < suspended.until);
    assert_eq!(world.assigned.len(), assigned, "draining worker cannot take the next attempt");
    assert!(matches!(
        live_task(&world, producer.task).phase,
        tasks::Phase::Active(tasks::Active::Due | tasks::Active::Preparing)
    ));
    unblamed(&world, owner.task, change, producer.task);

    // The surviving owner's claim is listed on the new channel; only the producer restarts.
    world.send(protocol::lost(CHANNEL));
    world.send(protocol::hello_slots(
        RETURNED,
        2,
        Box::new([protocol::hosting(owner.task, owner.attempt, jig_domain_link::Phase::Active)]),
    ));
    let retry = reviews::next_assignment(&mut world, assigned);
    assert_eq!(retry.task, producer.task);
    assert!(retry.attempt > producer.attempt);
    assert_eq!(retry.run.policy.time, producer.run.policy.time, "whole run budget");
    assert_eq!(retry.check_timeout, CHECK, "whole check bound");
    unblamed(&world, owner.task, change, producer.task);

    // A new attempt can finish production and the change goes on to its forge checks.
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("returned producer pushes");
    let event = world.protocol.answer(
        RETURNED,
        retry.task,
        retry.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if matches!(questions::row(&world, change).change.state, State::Checking { .. }) {
            break;
        }
    }
    assert!(matches!(questions::row(&world, change).change.state, State::Checking { .. }));
    world.close();
}
