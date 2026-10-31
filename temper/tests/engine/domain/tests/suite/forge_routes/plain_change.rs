//! A plain repository request goes straight from the chat to a change
//! (temper/docs/design/domain/agent.md, 8.2).
use super::*;

#[test]
fn a_chats_plain_change_request_calls_change() {
    const WORDS: &[u8] = b"Add CSV export to widget";
    let (mut world, chat) = change_tool::chat_world_with_words(true, WORDS);
    let engine::CallAnswer::Delegated(made) = change_tool::change(&mut world, &chat, 93, None, WORDS) else {
        panic!("delegated change")
    };
    let change_task = made[0].task;
    let row = live_task(&world, change_task);
    assert_eq!(row.requester, tasks::Party::Task(chat.task));
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 0, code: 2 });
    assert_eq!(row.spec.words.as_ref(), WORDS);
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    assert_eq!(live_task(&world, producer.task).requester, tasks::Party::Task(change_task));
    assert!(task_brief(&producer).windows(WORDS.len()).any(|part| part == WORDS));
    let branch = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Change(row) if row.task == change_task => Some(row.branch.clone()),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("the chat's change branch");
    change_tool::land_change(&mut world, &producer, &branch);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(change_task)))) {
            break;
        }
    }
    assert_eq!(world.merges, 1, "the plain change lands on the independent fake forge");
    assert!(
        matches!(
            world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(change_task)))),
            Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))))
                if matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. })))
        ),
        "the change reports its landed result: {:?}",
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(change_task))))
    );
    let main = world.fake.branch(b"org/repo", b"main").expect("landed main");
    assert_eq!(
        world.fake.object(main).expect("merge object").tree.get(b"file".as_slice()).map(AsRef::as_ref),
        Some(b"fixed".as_slice())
    );
    assert!(
        !world.store.rows.values().any(|record| matches!(record,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
            if row.tracked.is_some())),
        "no goal task exists"
    );
}
