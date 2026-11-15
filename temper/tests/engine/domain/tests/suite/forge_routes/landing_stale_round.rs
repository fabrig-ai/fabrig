//! Preparation crossing a stale-round boundary (temper/docs/design/domain/landing.md, 5.1–5.2, 15).
use super::*;

#[test]
fn a_prepare_completed_on_a_stale_base_reprepares_and_lands() {
    let (mut world, change) = produced_turn_change(None);
    world.manual_prepare_jobs();
    for _ in 0..600 {
        world.tick();
        if !world.jobs.is_empty() {
            break;
        }
    }
    assert_eq!(world.jobs.len(), 1);
    let landing = world.jobs[0].workspace.as_ref().expect("prepare workspace").workstream;
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"outside", b"new base", 2)
        .expect("main moves while prepare runs");
    world.automatic_prepare_jobs();
    for _ in 0..1200 {
        world.tick();
        if ended(&world, landing) && ended(&world, change) {
            break;
        }
    }
    assert_fresh_landing(&world, landing, change, base);
}

#[test]
fn a_released_prepare_ending_on_a_stale_base_starts_a_fresh_round() {
    let (mut world, change) = produced_turn_change(None);
    world.manual_prepare_jobs();
    let mut child = None;
    for _ in 0..1200 {
        world.tick();
        if let Some(job) = world.jobs.pop() {
            let task = landing_rows(&world)[0].job.as_ref().expect("admitted prepare").step.task;
            child = Some(task);
            let event = world.protocol.answer_job(
                Token::new(7),
                job.run.raw(),
                job.attempt.raw(),
                host::JobEnding::Failed {
                    failure: host::JobFailure::Failed,
                    detail: b"prepare failed".as_slice().into(),
                    work: host::Work { left: None, saved: None },
                },
            );
            world.send(event);
        }
        if child.is_some_and(|child| matches!(task_row(&world, child).expect("child").phase, tasks::Phase::Held { .. }))
            && matches!(landing_rows(&world)[0].landing.phase, forge_top::LandingPhase::Held { .. })
        {
            break;
        }
    }
    let child = child.expect("prepare child");
    assert!(matches!(task_row(&world, child).expect("child").phase, tasks::Phase::Held { .. }));
    let landing = landing_rows(&world)[0].landing.task;
    assert!(matches!(landing_rows(&world)[0].landing.phase, forge_top::LandingPhase::Held { .. }));
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"outside", b"new base", 2)
        .expect("main advances while prepare is held");
    let revision = task_row(&world, child).expect("held child").revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner"),
        key: [94; 16],
        ask: people::Ask::Release { project: 1, task: child, revision, again: None },
    }));
    world.automatic_prepare_jobs();
    for _ in 0..600 {
        world.tick();
        if ended(&world, child) {
            break;
        }
    }
    assert!(ended(&world, child), "retried prepare ends before its landing resumes");
    let revision = task_row(&world, landing).expect("held landing").revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: world.signed_in.expect("owner"),
        key: [95; 16],
        ask: people::Ask::Release { project: 1, task: landing, revision, again: None },
    }));
    for _ in 0..1200 {
        world.tick();
        if ended(&world, landing) && ended(&world, change) {
            break;
        }
    }
    assert_fresh_landing(&world, landing, change, base);
}

fn assert_fresh_landing(world: &World, landing: u64, change: u64, base: u64) {
    let row = landing_rows(world).into_iter().find(|row| row.landing.task == landing).expect("landing");
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Landed, "{row:?}");
    assert_eq!(row.landing.preparation, 2);
    assert_eq!(row.landing.stale_proofs, 1);
    assert_eq!(row.landing.members[0].base, Some(translate::commit(base)));
    assert!(ended(world, change));
    assert_eq!(
        world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).count(),
        1,
        "only the fresh round is proved"
    );
    let prepares = world
        .store
        .rows
        .values()
        .filter(|record| {
            matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task) | tasks::Stored::Ended(task)))
        if task.requester == tasks::Party::Task(landing)
            && task.executor == tasks::Executor::Job { kind: 1 })
        })
        .count();
    assert_eq!(prepares, 2, "one prepare child per round; retries keep their child");
}
