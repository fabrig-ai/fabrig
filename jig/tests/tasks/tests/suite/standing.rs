//! Standing creation forms (jig/docs/design/domain/tasks/standing.md, 2 and 7).
use jig_domain_tasks::{self as tasks, EndedBy, Executor, Party, Problem, Refusal};
use jig_tasks_world::{LIMITS, Reply, World, task};

fn member() -> tasks::New {
    let mut member = task(1, &[]);
    member.flags.ended_by = EndedBy::Standing;
    member
}

#[test]
fn standing_shape_is_retained_and_malformed_batches_are_refused() {
    let mut world = World::new(863, LIMITS);
    assert_eq!(world.make(Party::Person(1), vec![member()]), Reply::Made(vec![1]));
    assert_eq!(world.record(1).flags.ended_by, EndedBy::Standing);
    assert_eq!(world.record(1).session, None);
    let record = world.record(1).clone();
    world.restart();
    assert_eq!(world.record(1), &record);

    let mut conversation = member();
    conversation.flags.conversation = true;
    let mut tracked = member();
    tracked.tracked = Some(1);
    for invalid in [conversation, tracked] {
        refused(Party::Person(1), invalid);
    }
    for executor in [
        Executor::Person(tasks::PersonAddress::Person(1)),
        Executor::Procedure { connector: 1, code: 6 },
        Executor::Job { kind: 7 },
    ] {
        let mut invalid = member();
        invalid.executor = executor;
        refused(Party::Person(1), invalid);
    }
    for requester in [Party::Task(2), Party::Deployment { project: 1 }, Party::Person(0)] {
        refused(requester, member());
    }
}

fn refused(requester: Party, member: tasks::New) {
    let mut world = World::new(863, LIMITS);
    assert_eq!(world.make(Party::Person(1), vec![task(2, &[])]), Reply::Made(vec![2]));
    let before = world.records.clone();
    assert_eq!(world.make(requester, vec![member]), Reply::Refused(Problem::new(Some(1), Refusal::Contract)));
    assert_eq!(world.records, before);
}

#[test]
fn standing_party_batch_survives_restart_and_rejects_tracking() {
    use jig_core_kit::{
        fixtures,
        world::{Setup, World as CoreWorld},
    };
    use jig_domain_people as people;
    let mut world = CoreWorld::new(Setup { seed: 863, ..Setup::default() });
    let people::Ask::Create { project, mut batch } = fixtures::chat(&world, 1, b"stand".as_slice().into()) else {
        panic!("create fixture");
    };
    batch[0].flags.conversation = false;
    batch[0].flags.ended_by = people::EndedBy::Standing;
    batch[0].tracked = None;
    let mut tracked = batch.clone();
    tracked[0].tracked = Some(1);
    fixtures::ask(&mut world, 91, people::Ask::Create { project, batch: tracked });
    fixtures::refusal(&world, people::Refusal::NotWellFormed { at: 0, why: people::Malformed::EndedBy });
    fixtures::ask(&mut world, 92, people::Ask::Create { project, batch });
    let Some(people::Reply::Outcome(people::Outcome::Created { first: number, .. })) = world.people_answers.last()
    else {
        panic!("standing batch was not made: {:?}", world.people_answers);
    };
    let number = *number;
    assert_eq!(fixtures::live_row(&world, number).flags.ended_by, EndedBy::Standing);
    assert_eq!(fixtures::live_row(&world, number).session, None);
    world.restart(863, false);
    assert_eq!(fixtures::live_row(&world, number).flags.ended_by, EndedBy::Standing);
    assert_eq!(fixtures::live_row(&world, number).session, None);
}
