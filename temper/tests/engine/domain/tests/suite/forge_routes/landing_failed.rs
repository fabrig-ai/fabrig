//! Jig keeps the task's failure while the forge ends its landing as cancelled (temper/docs/design/domain/landing-liveness.md, 5).
use super::*;
use skein_lib::Queue;
use temper_engine_domain_forge_change as change;

const PROOF_TIME: Duration = Duration::from_secs(120);
const STEPS: u32 = 1200; // The forge world's full landing and hold-release horizon.
const REASON: &[u8] = b"the deployment failed the landing during proof";

fn proving() -> (World, u64, u64) {
    // Keep the failed row beside its successor.
    const LANDINGS: u32 = 2;
    let (mut world, change) = produced_turn_change_with(&StoryLimits {
        turn: true,
        landings: Some(LANDINGS),
        landing_gate_latency: Some(PROOF_TIME),
        ..StoryLimits::default()
    });
    for _ in 0..STEPS {
        world.tick();
        if let Some(row) = landing_rows(&world)
            .into_iter()
            .find(|row| row.follows().iter().any(|follow| follow.entry.run.is_some_and(|run| !run.ended)))
        {
            let landing = row.landing.task;
            world.drain_cut();
            return (world, change, landing);
        }
    }
    panic!("landing did not reach proof");
}

fn fail_at_commit(world: &mut World, task: u64) {
    // The deployment's Fail is internal to jig. Form its closing record in
    // jig's task domain, then open the root at that cut before forge settlement.
    let row = task_row(world, task).expect("proving task");
    let by = tasks::Party::Deployment { project: row.project };
    let env = world.env.with(world.env.limits.tasks);
    let mut jig = tasks::Domain::new(
        &env.limits,
        tasks::Kept { revisions: 1, versions: 0 },
        task,
        row.settings.task,
        Box::new([]),
        Box::new([]),
    );
    let mut out = Queue::uncapped();
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ledger(row.funder)));
    let Some(Record::Core(jig_domain_core::Record::Tasks(ledger))) = world.store.rows.get(&key) else {
        panic!("landing's project ledger");
    };
    for record in [ledger.clone(), tasks::Stored::Live(Box::new(row.clone()))] {
        tasks::step(&mut jig, &env, tasks::Event::Restore { record }, &mut out);
    }
    tasks::step(&mut jig, &env, tasks::Event::Check, &mut out);
    tasks::step(&mut jig, &env, tasks::Event::Restored, &mut out);
    tasks::step(
        &mut jig,
        &env,
        tasks::Event::Fail {
            reply_to: ReplyTo::new(Token::new(task)),
            by,
            operation: tasks::Fail { task, reason: REASON.into() },
        },
        &mut out,
    );
    let closing = jig.task(task).expect("closing task");
    let tasks::Phase::Closing(_) = closing.phase else { panic!("jig did not close the task: {out:?}") };
    let saved = Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(closing.clone()))));
    world.store.seed(Box::new([saved]), Box::new([]));
    world.restart_same();
    world.send(hello_prepare(world.env.limits.fleet.slots));
}

#[test]
fn jigs_failed_landing_releases_its_branch_and_the_next_lands() {
    let (mut world, change, failed) = proving();
    let ready = world.store.change(change).expect("member change").change.state;
    let holdings = task_row(&world, failed).expect("proving task").holdings.clone();
    fail_at_commit(&mut world, failed);
    for _ in 0..STEPS {
        world.tick();
        if world.root.forge().landing(failed).is_some_and(|row| row.landing.phase == forge_top::LandingPhase::Cancelled)
        {
            break;
        }
    }
    let row = world.root.forge().landing(failed).expect("failed landing kept");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Cancelled);
    assert_eq!(world.store.change(change).expect("member change").change.state, ready);
    assert!(matches!(ready, change::State::Ready { .. }));

    for _ in 0..STEPS {
        super::restarts::tick(&mut world);
        if ended(&world, failed) && ended(&world, change) {
            break;
        }
    }
    assert_eq!(
        task_row(&world, failed).expect("failed task kept").phase,
        tasks::Phase::Ended(tasks::Ending::Failed { reason: REASON.into() })
    );
    let [tasks::Holding::Write { resource, .. }] = holdings.as_ref() else { panic!("one round branch holding") };
    assert_eq!(world.root.tasks().holder(resource), None);
    let successor = landing_rows(&world)
        .into_iter()
        .find(|row| row.landing.task != failed && row.landing.members.iter().any(|member| member.change == change))
        .expect("member's replacement landing");
    assert_eq!(successor.landing.phase, forge_top::LandingPhase::Landed);
    assert!(matches!(world.store.change(change).expect("landed change").change.state, change::State::Landed { .. }));
    world.close();
}
