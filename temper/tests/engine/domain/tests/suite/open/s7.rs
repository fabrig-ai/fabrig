//! S7 is a symptom, not a claim about the deployment's lost child's cause.
//! temper/docs/design/domain/landing-preparing.md, 2; landing.md, 4.2.
use super::*;
use crate::forge_routes::landing::restarts::{marked_prepare, start_prepare};

// One durable cancellation and the landing's next decisions, without waiting
// for the prepare's 600-second deadline, must settle within 200 world ticks.
const SETTLE_TICKS: u32 = 200;

fn cancel_prepare(world: &mut World, child: u64) {
    person(
        world,
        97,
        people::Ask::Cancel {
            project: 1,
            task: child,
            reason: b"prepare child removed by its owner".as_slice().into(),
            grace: None,
        },
    );
    let job = world.jobs.pop().expect("running prepare");
    assert_eq!(job.run.raw(), child);
    let event = world.protocol.answer_job(
        Token::new(7),
        child,
        job.attempt.raw(),
        host::JobEnding::Failed {
            failure: host::JobFailure::Cancelled(host::Reason::Engine),
            detail: Box::new([]),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(event);
    for _ in 0..SETTLE_TICKS {
        world.tick();
        if matches!(task_row(world, child).expect("prepare").phase, tasks::Phase::Ended(_)) {
            break;
        }
    }
    assert!(matches!(
        task_row(world, child).expect("prepare ended").phase,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. })
    ));
}

#[test]
fn a_landing_without_its_prepare_task_moves() {
    let (mut world, member, landing) = marked_prepare();
    let (child, _) = start_prepare(&mut world, landing);
    // Stop the landing only during removal, so the fixture cannot replace the
    // child before the person's hold on its member arrives.
    person(
        &mut world,
        95,
        people::Ask::Stop { project: 1, task: landing, reason: Box::from(b"Parking the task".as_slice()) },
    );
    world.advance_ticks(20);
    cancel_prepare(&mut world, child);
    person(
        &mut world,
        98,
        people::Ask::Stop { project: 1, task: member, reason: Box::from(b"Parking the task".as_slice()) },
    );
    world.advance_ticks(20);
    assert!(matches!(task_row(&world, member).expect("member").phase, tasks::Phase::Held { .. }));
    let revision = task_row(&world, landing).expect("landing").revisions.hold;
    person(&mut world, 99, people::Ask::Release { project: 1, task: landing, revision, again: None });
    world.advance_ticks(SETTLE_TICKS);
    let row = landing_rows(&world).into_iter().find(|row| row.landing.task == landing).expect("landing row");
    let ended = matches!(task_row(&world, landing).expect("landing task").phase, tasks::Phase::Ended(_));
    let held_with_words = matches!(&task_row(&world, landing).expect("landing task").phase,
        tasks::Phase::Held { source: Some(source), .. }
        if matches!(source.as_ref(), tasks::HoldSource::Account { words, .. } if !words.is_empty()));
    assert!(
        ended || held_with_words,
        "within {SETTLE_TICKS} ticks, a missing prepare and held member must end the landing or hold with words: {:?}",
        row.landing
    );
    world.close();
}
