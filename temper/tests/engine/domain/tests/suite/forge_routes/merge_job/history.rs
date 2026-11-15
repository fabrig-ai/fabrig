//! Each job attempt ends by its cause (domain/landing-history.md, 2.2).
use super::*;
use forge_top::history as h;

fn events(world: &World, task: u64) -> Vec<h::Event> {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::HistoryEvent(event)
                    if event.change == task && matches!(event.what, h::What::Job { .. } | h::What::JobEnded { .. }) =>
                {
                    Some(event.clone())
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect()
}

fn ended(world: &mut World, task: u64, job: &host::JobAssignment, outcome: h::JobOutcome) {
    let attempt = u32::try_from(job.attempt.raw()).expect("job attempt fits history");
    for _ in 0..STEPS {
        if events(world, task)
            .iter()
            .any(|event| event.attempt == attempt && event.what == h::What::JobEnded { job: h::Job::Merge, outcome })
        {
            return;
        }
        world.tick();
    }
    panic!(
        "attempt end {outcome:?} missing: {:?}",
        events(world, task).iter().map(|event| (event.attempt, &event.what)).collect::<Vec<_>>()
    );
}

fn failed(world: &mut World, job: &host::JobAssignment, failure: host::JobFailure) {
    let event = world.protocol.answer_job(
        CHANNEL,
        job.run.raw(),
        job.attempt.raw(),
        host::JobEnding::Failed {
            failure,
            detail: b"job failed".as_slice().into(),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(event);
}

#[test]
fn merge_job_attempt_history_names_worker_failures_before_the_retry_and_then_done() {
    for (failure, outcome) in [
        (host::JobFailure::Failed, h::JobOutcome::Failed { class: h::JobFailure::Run }),
        (host::JobFailure::Deadline, h::JobOutcome::Timeout),
        (
            host::JobFailure::Undelivered(host::DeliveryOutcome::Failed),
            h::JobOutcome::Failed { class: h::JobFailure::Undelivered },
        ),
        (
            host::JobFailure::Unprepared(host::Preparation::Transient),
            h::JobOutcome::Failed { class: h::JobFailure::Unprepared },
        ),
        (host::JobFailure::Result, h::JobOutcome::Failed { class: h::JobFailure::Result }),
        (host::JobFailure::Unstarted, h::JobOutcome::Failed { class: h::JobFailure::Unstarted }),
        (host::JobFailure::Cancelled(host::Reason::Engine), h::JobOutcome::Cancelled),
        (host::JobFailure::Cancelled(host::Reason::Contact), h::JobOutcome::Lost),
        (host::JobFailure::Cancelled(host::Reason::Shutdown), h::JobOutcome::Lost),
    ] {
        let (mut world, task, head, base) =
            pushed_change_in(world(false, true, true).with_merge_job(COMMAND), true, GOAL);
        let first = next_job(&mut world, task, false);
        failed(&mut world, &first, failure);
        ended(&mut world, task, &first, outcome);
        if let tasks::Phase::Held { .. } = live_task(&world, first.run.raw()).phase {
            let revision = live_task(&world, first.run.raw()).revisions.hold;
            world.send(engine::Event::Party(engine::PartyInput::Ask {
                models: Box::new([]),
                reply_to: ReplyTo::new(Token::new(94)),
                sign_in: world.signed_in.expect("owner signed in"),
                key: [94; 16],
                ask: people::Ask::Release { project: 1, task: first.run.raw(), revision, again: None },
            }));
        }
        let retry = next_job(&mut world, task, false);
        assert_eq!(retry.run, first.run);
        let merged = push_merge(&mut world, task, head, base);
        finish_job(&mut world, &retry, RESULT);
        landed(&mut world, task, merged);
        let got = events(&world, task);
        assert_eq!(got.len(), 4, "{failure:?}: {got:?}");
        assert_eq!(got[1].what, h::What::JobEnded { job: h::Job::Merge, outcome });
        assert_eq!(got[1].attempt, u32::try_from(first.attempt.raw()).unwrap());
        assert_eq!(got[2].what, h::What::Job { job: h::Job::Merge, task: retry.run.raw() });
        assert_eq!(got[2].attempt, u32::try_from(retry.attempt.raw()).unwrap());
        assert_eq!(got[3].what, h::What::JobEnded { job: h::Job::Merge, outcome: h::JobOutcome::Done });
        world.close();
    }
}

#[test]
fn cancelling_a_merge_job_without_its_worker_answer_names_cancelled() {
    let (mut world, task, _, _) = pushed_change_in(world(false, true, true).with_merge_job(COMMAND), true, GOAL);
    let job = next_job(&mut world, task, false);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [95; 16],
        ask: people::Ask::Cancel {
            project: 1,
            task: job.run.raw(),
            reason: b"stop job".as_slice().into(),
            grace: Some(Duration::ZERO),
        },
    }));
    // No answer comes back from the cancelled worker; the fleet closes its call at loss.
    world.send(engine::Event::Worker(engine::WorkerInput::Link { channel: CHANNEL, link: engine::Link::Lost }));
    world.env.now = world.env.now.saturating_add(world.env.limits.fleet.grace).saturating_add(Duration::from_secs(1));
    ended(&mut world, task, &job, h::JobOutcome::Cancelled);
    assert_eq!(events(&world, task).iter().filter(|event| matches!(event.what, h::What::JobEnded { .. })).count(), 1);
    world.close();
}

#[test]
fn losing_a_merge_jobs_worker_without_an_answer_names_lost_before_retry() {
    let (mut world, task, head, base) = pushed_change_in(world(false, true, true).with_merge_job(COMMAND), true, GOAL);
    let first = next_job(&mut world, task, false);
    world.send(engine::Event::Worker(engine::WorkerInput::Link { channel: CHANNEL, link: engine::Link::Lost }));
    world.env.now = world.env.now.saturating_add(world.env.limits.fleet.grace).saturating_add(Duration::from_secs(1));
    ended(&mut world, task, &first, h::JobOutcome::Lost);
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let retry = next_job(&mut world, task, false);
    let merged = push_merge(&mut world, task, head, base);
    finish_job(&mut world, &retry, RESULT);
    landed(&mut world, task, merged);
    let got = events(&world, task);
    assert_eq!(got.len(), 4);
    assert_eq!(got[1].what, h::What::JobEnded { job: h::Job::Merge, outcome: h::JobOutcome::Lost });
    assert_eq!(got[2].attempt, u32::try_from(retry.attempt.raw()).unwrap());
    assert_eq!(got[3].what, h::What::JobEnded { job: h::Job::Merge, outcome: h::JobOutcome::Done });
    world.close();
}

#[test]
fn a_merge_attempt_end_restarts_before_readback_and_is_written_once() {
    let configured = world(false, true, true).with_merge_job(COMMAND);
    let mut retry = configured.env.limits.tasks.retries.transient;
    retry.retries = 0;
    let (mut world, task, head, base) = pushed_change_in(configured.with_transient_retry(retry), true, GOAL);
    let first = next_job(&mut world, task, false);
    world.held_reads = Some(Vec::new());
    world.hold_reads_at_any_step = true;
    failed(&mut world, &first, host::JobFailure::Undelivered(host::DeliveryOutcome::Failed));
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, task).job.as_ref().is_some_and(|job| job.step.outcome.is_some())
            && world.store.pending.is_empty()
            && world.pending.is_empty()
        {
            break;
        }
    }
    let kept = change_row(&world, task).job.unwrap();
    assert_eq!(
        kept.step.outcome,
        Some(h::JobOutcome::Failed { class: h::JobFailure::Undelivered }),
        "kept {kept:?}, events {:?}, child {:?}, pending store {}, API {}",
        events(&world, task),
        world.root.tasks().task(first.run.raw()).map(|row| &row.phase),
        world.store.pending.len(),
        world.pending.len()
    );
    assert!(!kept.written);
    world.restart_same();
    // Reads belong to the stopped process. Startup makes its own read-back.
    world.held_reads = None;
    for _ in 0..STEPS {
        world.tick();
        if world.root.ready() && world.store.pending.is_empty() {
            break;
        }
    }
    let revision = live_task(&world, first.run.raw()).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [94; 16],
        ask: people::Ask::Release { project: 1, task: first.run.raw(), revision, again: None },
    }));
    let retry = next_job(&mut world, task, false);
    let merged = push_merge(&mut world, task, head, base);
    finish_job(&mut world, &retry, RESULT);
    landed(&mut world, task, merged);
    let got = events(&world, task);
    assert_eq!(got.len(), 4);
    assert_eq!(
        got[1].what,
        h::What::JobEnded { job: h::Job::Merge, outcome: h::JobOutcome::Failed { class: h::JobFailure::Undelivered } }
    );
    assert_eq!(got[2].attempt, u32::try_from(retry.attempt.raw()).unwrap());
    assert_eq!(got[3].what, h::What::JobEnded { job: h::Job::Merge, outcome: h::JobOutcome::Done });
    world.close();
}
