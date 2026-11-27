//! The durable comparison oracle (`temper/docs/design/testing/workloads.md`, 9).
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::{Time, Wall};
use skein_world::{
    faults::{FaultLedger, Struck},
    streams::Streams,
    sweep::Sweep,
};
use temper_engine_domain::{Key, Record};
use temper_engine_domain_world::{outcome, workload};
use temper_faults::restarts::{ENGINE_CRASH, ENGINE_CRASH_MID_TURN, ENGINE_RESTART_MID_TURN, SETTLED_STOP};
use temper_workload::{
    draw::{Change, File, Goal, Party, Workload},
    outcome::End,
    parties::{Act, Action, When},
};

fn one_goal() -> Workload {
    Workload {
        tier: false,
        goals: vec![Goal {
            name: "g0".into(),
            party: Party::First,
            after: None,
            train: false,
            changes: vec![Change {
                name: "g0.c0".into(),
                after: vec![],
                asks: false,
                red_once: false,
                files: vec![File { path: "g0/c0/file".into(), text: "landed\n".into() }],
            }],
        }],
    }
}

#[test]
fn outcome_reads_a_landed_goal_and_main() {
    let run = workload::run_actions(&Streams::new(0), one_goal(), vec![], &Sweep::new("outcome"), |_, _, _, _| {})
        .expect("world");
    let outcome = outcome::read(&run.world.store, &run.main, &run.names, &run.workload, &run.ledger);
    assert_eq!(outcome.goals["g0"].end, End::Landed);
    assert_eq!(outcome.goals["g0"].landed, ["g0.c0"]);
    assert_eq!(outcome.main, [("g0/c0/file".into(), "landed\n".into())].into());

    // Closing projection cleanup can erase the closer before the world reads the outcome.
    let mut store = run.world.store;
    let goal = run.names.task("g0").expect("goal");
    let closer = store
        .rows
        .iter()
        .find_map(|(key, record)| {
            let Record::Forge { row } = record else { return None };
            let temper_engine_domain_forge::Stored::Closer(row) = row.as_ref() else { return None };
            (row.goal == goal).then(|| key.clone())
        })
        .expect("the closer projection was present");
    store.rows.remove(&closer);
    let settled = outcome::read(&store, &run.main, &run.names, &run.workload, &run.ledger);
    assert_eq!(settled.goals["g0"].end, End::Landed);
}

#[test]
fn outcome_reads_a_cancelled_goal() {
    let action =
        Action { party: Party::First, act: Act::CancelGoal, target: "g0".into(), when: When::Started, after: 0 };
    let run =
        workload::run_actions(&Streams::new(0), one_goal(), vec![action], &Sweep::new("outcome"), |_, _, _, _| {})
            .expect("world");
    let outcome = outcome::read(&run.world.store, &run.main, &run.names, &run.workload, &run.ledger);
    assert_eq!(outcome.goals["g0"].end, End::Cancelled);
    assert_eq!(outcome.goals["g0"].landed, Vec::<String>::new());
    assert!(outcome.main.is_empty());
}

#[test]
fn outcome_normalizes_only_lost_tries_inside_a_crash() {
    let mut run = workload::run_actions(&Streams::new(0), one_goal(), vec![], &Sweep::new("outcome"), |_, _, _, _| {})
        .expect("world");
    let goal = run.names.task("g0").expect("goal");
    let key = Key::Core(core::Key::Tasks(tasks::Key::Ended(goal)));
    let Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        run.world.store.rows.get_mut(&key).expect("ended")
    else {
        panic!("ended goal")
    };
    row.tries.lost = 2;
    row.tries.run = 3;
    for (attempt, at) in [(900, 10), (901, 20)] {
        let row = tasks::AttemptRow {
            task: goal,
            attempt,
            claimed: None,
            started: None,
            ended: Some(Wall::from_nanos(at)),
            end: Some(tasks::EndKind::Failed { class: tasks::Class::Lost, invalid_result: None }),
            phases: None,
        };
        run.world.store.rows.insert(
            Key::Core(core::Key::Tasks(tasks::Key::Attempt { task: goal, attempt })),
            Record::Core(core::Record::Tasks(tasks::Stored::Attempt(row))),
        );
    }
    for (crash_kind, stop_kind) in [(ENGINE_CRASH, SETTLED_STOP), (ENGINE_CRASH_MID_TURN, ENGINE_RESTART_MID_TURN)] {
        let mut ledger = FaultLedger::new();
        let crash = ledger.inject(crash_kind, Time::from_nanos(10), Struck::default());
        ledger.stop(crash, Time::from_nanos(15)).expect("closed interval");
        let stop = ledger.inject(stop_kind, Time::from_nanos(20), Struck::default());
        ledger.stop(stop, Time::from_nanos(25)).expect("closed interval");
        let read = outcome::read(&run.world.store, &run.main, &run.names, &run.workload, &ledger);
        assert_eq!(read.goals["g0"].tries, 4, "only the lost attempt within {crash_kind:?} is discounted");
    }
    let mut ledger = FaultLedger::new();
    let crash = ledger.inject(ENGINE_CRASH, Time::from_nanos(10), Struck::default());
    ledger.stop(crash, Time::from_nanos(15)).expect("closed interval");
    let Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) =
        run.world.store.rows.get_mut(&key).expect("ended")
    else {
        panic!("ended goal")
    };
    row.tries.lost = 0;
    let read = outcome::read(&run.world.store, &run.main, &run.names, &run.workload, &ledger);
    assert_eq!(read.goals["g0"].tries, 3, "old lost attempts cannot discount another class after a reset");
}

#[test]
fn comparison_names_a_goal_whose_restarted_end_changed() {
    let run = workload::run_actions(&Streams::new(0), one_goal(), vec![], &Sweep::new("outcome"), |_, _, _, _| {})
        .expect("world");
    let outcome = outcome::read(&run.world.store, &run.main, &run.names, &run.workload, &run.ledger);
    let mut restarted = outcome.clone();
    restarted.goals.get_mut("g0").expect("goal").end = End::Failed;
    let finding = outcome.compare(&restarted, &[]).expect_err("changed outcome is found");
    assert!(finding.contains("compare: g0: end Landed without restarts, Failed with them"), "{finding}");
}
