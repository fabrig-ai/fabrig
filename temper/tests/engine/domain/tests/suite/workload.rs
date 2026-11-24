//! The no-fault workload reaches its durable goals (`temper/docs/design/testing/workloads.md`, 9).
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_world::streams::Streams;
use temper_engine_domain::{Key, Record};
use temper_engine_domain_world::workload;

#[test]
fn an_engine_workload_without_acts_ends_every_goal() {
    // The minimum complete shape: two goals, a two-goal milestone and a two-member
    // train, with no extra ask or repair work. The sweep exercises those acts.
    let run = workload::without_acts(&Streams::new(46)).expect("all goals reach Ends");
    assert!((2..=4).contains(&run.workload.goals.len()));
    let main = run.world.fake.branch(b"org/repo", b"main").expect("main");
    let tree = &run.world.fake.object(main).expect("main commit").tree;
    for goal in &run.workload.goals {
        if goal.train {
            let tasks: Vec<_> = goal.changes.iter().filter_map(|change| run.names.task(&change.name)).collect();
            assert!(
                run.world.store.rows.values().any(|record| match record {
                    temper_engine_domain::Record::Forge { row } => match row.as_ref() {
                        temper_engine_domain_forge::Stored::Landing(row) =>
                            tasks.iter().all(|task| row.landing.members.iter().any(|member| member.change == *task)),
                        _ => false,
                    },
                    temper_engine_domain::Record::Core(_) => false,
                }),
                "train {tasks:?} landed together; draw {goal:?}"
            );
        }
        let goal_task = run.names.task(&goal.name).expect("every goal started");
        assert!(
            matches!(run.world.store.rows.get(&Key::Core(core::Key::Tasks(tasks::Key::Ended(goal_task)))),
            Some(Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) if matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_)))),
            "{} ended successfully",
            goal.name
        );
        for change in &goal.changes {
            let task = run.names.task(&change.name).expect("every change opened");
            let Some(Record::Core(core::Record::Tasks(tasks::Stored::Ended(row)))) =
                run.world.store.rows.get(&Key::Core(core::Key::Tasks(tasks::Key::Ended(task))))
            else {
                panic!("{} ended", change.name)
            };
            for after in &change.after {
                assert!(
                    row.dependencies.contains(&run.names.task(after).expect("dependency named")),
                    "{} keeps after {after}",
                    change.name
                );
            }
            assert_eq!(
                run.world.store.change_hot(task).expect("change").change.repairs,
                u32::from(change.red_once),
                "{} repaired once when red",
                change.name
            );
            for file in &change.files {
                assert_eq!(
                    tree.get(file.path.as_bytes()).map(AsRef::as_ref),
                    Some(file.text.as_bytes()),
                    "{} reached main",
                    change.name
                );
            }
        }
    }
}
