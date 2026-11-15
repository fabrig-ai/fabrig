//! Formation behind an unavailable procedure (temper/docs/design/domain/landing.md, 9, 14).
use super::*;

#[test]
fn drill_20_a_due_change_blocks_formation_without_an_account_in_the_train_view() {
    // An external procedure has no executor in this engine. Its registered change
    // remains due while another change in the same train is ready.
    const UNAVAILABLE: u32 = 99;
    let (mut world, ready) = produced_turn_change(Some(0));
    world.until(Until::Ready);
    let chat = world.assigned[0].clone();
    let (due, producer) = make_change(&mut world, &chat, 101, b"org/repo");
    push_change(&mut world, &producer, b"org/repo");
    finish_producer(&mut world, &producer, forge_world::REPO);
    for _ in 0..PAIR_TICKS {
        world.tick();
    }
    assert!(landing_rows(&world).is_empty());
    world.drain_cut();
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(due)));
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))) = world.store.rows.get_mut(&key)
    else {
        panic!("due task")
    };
    task.phase = tasks::Phase::Active(tasks::Active::Due);
    task.executor = tasks::Executor::Procedure { connector: 0, code: UNAVAILABLE };
    let Some(Record::Forge { row }) = world.store.rows.get_mut(&Key::Forge(forge_top::Key::Change(due))) else {
        panic!("change record")
    };
    let forge_top::Stored::Change(row) = row.as_mut() else { panic!("change") };
    row.change.state = temper_engine_domain_forge_change::State::Gating {
        head: row.change.last_head.expect("delivered head"),
        asked: Box::new([]),
    };
    let task = world.store.rows.get(&key).expect("due task").clone();
    let change = world.store.rows.get(&Key::Forge(forge_top::Key::Change(due))).expect("change").clone();
    world.store.seed(Box::new([task, change]), Box::new([]));
    world.restart_with_batch(1);
    for _ in 0..PAIR_TICKS {
        world.tick();
    }
    assert_eq!(world.root.tasks().procedure_due(due).map(|(_, code, _)| code), Some(UNAVAILABLE));
    assert!(matches!(
        super::super::goal_landing::row(&world, ready).change.state,
        temper_engine_domain_forge_change::State::Ready { .. }
    ));
    assert!(landing_rows(&world).is_empty(), "the train waits for due readiness");
    assert!(!matches!(task_row(&world, due).expect("due task").phase, tasks::Phase::Held { .. }));
    world.signed_in = None;
    world.sign_in();
    let sign_in = world.signed_in.expect("person");
    let Some(forge_top::Document::Changes(view)) = world.oracle_document(sign_in, 1, forge_top::changes_subject(1))
    else {
        panic!("train view")
    };
    let train = view.queues.iter().find(|queue| queue.branch.as_ref() == b"main").expect("main train");
    assert_eq!(train.changes.iter().map(|change| change.task).collect::<Vec<_>>(), [ready]);
    assert_eq!(train.in_flight, None);
    // The current wire has no formation-wait account; neither view names `due`.
    let Some(forge_top::Document::Landings(view)) =
        world.oracle_document(sign_in, 1, forge_top::landings_subject(1, true))
    else {
        panic!("landing view")
    };
    assert!(view.landings.is_empty());
    // The unavailable procedure is intentional; it cannot be drained at teardown.
    world.broken();
    world.close();
}
