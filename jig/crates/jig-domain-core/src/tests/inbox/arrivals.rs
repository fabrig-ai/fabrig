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
        Event::People(people::Event::Waiting { source: people::NeedSource::Task(7), entries: Box::new([first]) }),
    );
    assert_arrival(&out, 1);
    let second = people::Entry { kind: people::EntryKind::Question { message: 12 }, ..first };
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::Waiting {
            source: people::NeedSource::Task(7),
            entries: Box::new([first, second]),
        }),
    );
    assert_arrival(&out, 2);
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::People(people::Event::Waiting {
            source: people::NeedSource::Task(7),
            entries: Box::new([first, second]),
        }),
    );
    assert!(out.is_empty(), "the same task saved again tells nothing");
}

fn assert_arrival(out: &[Request], expected: u32) {
    let [Request::Held(held)] = out else { panic!("one held arrival: {out:?}") };
    let Held::NeedArrived { party, project, task, needs } = held.as_ref() else { panic!("arrival: {held:?}") };
    assert_eq!((*party, *project, *task, *needs), (1, 1, 7, expected));
}

#[test]
fn a_stall_offer_arrives_as_an_ordinary_person_task_need() {
    const GOAL: u64 = 1;
    const TASK: u64 = 2;
    const REQUESTER: u64 = 1;
    const ADDRESSEE: u64 = 6;
    let (mut core, limits) = tree_core();
    let root = new_row(&core, GOAL, tasks::Funder::Period { project: 1, period: 1 }, Some(1));
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(REQUESTER), Box::new([root]));
    let mut task = new_row(&core, TASK, tasks::Funder::Task(GOAL), None);
    task.executor = tasks::Executor::Person(tasks::PersonAddress::Person(ADDRESSEE));
    fixtures::make_batch(&mut core, &limits, tasks::Party::Task(GOAL), Box::new([task]));
    let env = Env {
        now: Time::ZERO.saturating_add(limits.tasks.person_stall),
        wall: Wall::from_nanos(limits.tasks.person_stall.as_nanos()),
        limits,
    };
    let mut pending = Queue::uncapped();
    tasks::fire(&mut core.tasks, &env.with(limits.tasks), &mut pending);
    let mut out = crate::routing::task::tag_task_outputs(&mut core, &env, pending);
    let mut arrived = None;
    for _ in 0..out.len() {
        if let Some(Request::Held(held)) = out.pop()
            && let Held::NeedArrived { party, project, task, needs } = *held
        {
            assert!(arrived.is_none(), "one arrival at the new holder");
            arrived = Some((party, project, task, needs));
        }
    }
    assert_eq!(arrived, Some((REQUESTER, 1, TASK, 1)));
    let inbox = core.people.inbox_needs(REQUESTER).expect("requester inbox");
    assert_eq!(inbox.first().expect("offered entry").kind, people::EntryKind::PersonTask);
}
