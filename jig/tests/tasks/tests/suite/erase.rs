use jig_domain_tasks::{Event, Key, Party, Request, Stored};
use jig_tasks_world::{LIMITS, World, task};

const TASK: u64 = 1;
const REQUESTER: u64 = 9;
const SEED: u64 = 1301;

fn ended_world() -> World {
    let mut world = World::new_start_restore(SEED, LIMITS);
    let mut listed = task(TASK, &[]);
    listed.flags.listed = true;
    world.make(Party::Person(REQUESTER), vec![listed]);
    world.claim(TASK, TASK);
    world.finish(TASK);
    world.settle(TASK);
    world
}

#[test]
fn an_erased_task_leaves_no_ended_index_after_a_restart() {
    let mut world = ended_world();
    let Some(Stored::Ended(record)) = world.records.get(&Key::Ended(TASK)) else {
        panic!("ended task is durable");
    };
    let record = record.clone();
    let keys = [
        Key::Ended(TASK),
        jig_domain_tasks::EndedByParty {
            party: REQUESTER,
            label: record.label,
            ended_at: record.ended_at.expect("ended record has its ending time"),
            task: TASK,
        }
        .key(),
        jig_domain_tasks::EndedByProject {
            project: record.project,
            label: record.label,
            ended_at: record.ended_at.expect("ended record has its ending time"),
            task: TASK,
        }
        .key(),
    ];
    world.send(Event::Erase { record });
    for key in keys {
        assert!(!world.records.contains_key(&key), "erased {key:?}");
    }
    let before = world.records.clone();
    world.restart();
    assert_eq!(world.records, before);
    assert_eq!(world.live(), 0);
    assert!(!world.records.contains_key(&Key::Stub(TASK)));
    assert!(world.activations.is_empty());
}

#[test]
fn erase_referee_requires_an_explicit_erase_and_keeps_history_immutable() {
    let world = ended_world();
    let mut bad = world.records.clone();
    bad.remove(&Key::Ended(TASK));
    let mut judge = jig_tasks_world::accounting_referee::Accounting::default();
    judge.reset(&world.records);
    assert_eq!(judge.committed(&bad, &[]), Err("immutable accounting row changed"));

    let mut bad = world.records.clone();
    let Some(Stored::Ended(ended)) = bad.get_mut(&Key::Ended(TASK)) else { panic!("ended fixture") };
    ended.label += 1;
    let mut judge = jig_tasks_world::accounting_referee::Accounting::default();
    judge.reset(&world.records);
    assert_eq!(
        judge.committed(&bad, &[Request::Erase { key: Key::Ended(TASK) }]),
        Err("immutable accounting row changed")
    );
}
