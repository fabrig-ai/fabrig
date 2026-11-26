//! S7 is a symptom, not a claim about the deployment's lost child's cause.
//! temper/docs/design/domain/landing-liveness.md, 4; landing-preparing.md, 2.
use super::*;
use crate::forge_routes::landing::wakes::{answer_stopped_prepare, cancel, stop, waiting_prepare};

// One durable cancellation and the landing's next decisions, without waiting
// for the prepare's 600-second deadline, must settle within 200 world ticks.
const SETTLE_TICKS: u32 = 200;
const STOP_MEMBER: u8 = 98;
const CANCEL_PREPARE: u8 = 97;

#[test]
fn a_landing_without_its_prepare_task_moves() {
    let (mut world, member, landing, child) = waiting_prepare();
    stop(&mut world, STOP_MEMBER, member);
    assert!(matches!(world.root.tasks().task(member).expect("member").phase, tasks::Phase::Held { .. }));
    assert!(matches!(
        world.root.tasks().task(child).expect("outstanding prepare").phase,
        tasks::Phase::Active(tasks::Active::Running { .. })
    ));
    cancel(&mut world, CANCEL_PREPARE, child);
    answer_stopped_prepare(&mut world, child);
    // Only the member has a person's hold. The landing has not ended at this cut.
    assert!(matches!(
        world.root.tasks().task(landing).expect("live landing at the cut").phase,
        tasks::Phase::Active(_)
    ));
    world.advance_ticks(SETTLE_TICKS);
    assert!(matches!(
        task_row(&world, child).expect("prepare ended").phase,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. })
    ));
    assert!(matches!(task_row(&world, member).expect("member stays held").phase, tasks::Phase::Held { .. }));
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
