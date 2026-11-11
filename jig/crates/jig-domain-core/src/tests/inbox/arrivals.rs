//! Root-facing arrivals use the same held boundary as results.
use crate::tests::*;

#[test]
fn a_requesters_question_arrival_is_a_held_output() {
    let (mut core, limits) = core();
    let first = people::Entry {
        task: 7,
        project: 1,
        whom: people::Whom::Person(1),
        kind: people::EntryKind::Question { message: 11 },
        at: Wall::EPOCH,
    };
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::Waiting { task: 7, entries: Box::new([first]) }),
    );
    assert_arrival(&out, 1);
    let second = people::Entry { kind: people::EntryKind::Question { message: 12 }, ..first };
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::Waiting { task: 7, entries: Box::new([first, second]) }),
    );
    assert_arrival(&out, 2);
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::Waiting { task: 7, entries: Box::new([first, second]) }),
    );
    assert!(out.is_empty(), "the same task saved again tells nothing");
}

fn assert_arrival(out: &[Request], expected: u32) {
    let [Request::Held(held)] = out else { panic!("one held arrival: {out:?}") };
    let Held::NeedArrived { party, project, task, needs } = held.as_ref() else { panic!("arrival: {held:?}") };
    assert_eq!((*party, *project, *task, *needs), (1, 1, 7, expected));
}
