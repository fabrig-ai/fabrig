//! Adversarial kept-change stories against domain/forge.md, 8.2 and 9.
use skein_lib::Wall;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{World, changes};

const FIRST: u64 = 51;
const SECOND: u64 = 52;
const FIRST_READY: Wall = Wall::from_nanos(100_000_000_000);
const SECOND_READY: Wall = Wall::from_nanos(101_000_000_000);

fn queued(world: &mut World, task: u64, branch: &[u8], path: &[u8], ready: Wall) {
    world.produce_file(branch, path, b"change");
    world.status(branch, skein_fake_forge_domain::api::Check::Passed);
    let pull = world.open_pull(branch, b"main");
    let head = temper_engine_forge_world::translate::commit(world.branch(branch));
    let mut row = changes::change_row();
    row.task = task;
    row.branch = branch.into();
    row.pull = Some(pull);
    row.change.task = task;
    row.change.state = change::State::Queued { head, ready_since: ready };
    row.change.ready_since = Some(ready);
    row.change.last_head = Some(head);
    row.change.owns_turn = task == FIRST;
    world.event(forge::Event::Change { row });
}

fn step(world: &mut World, task: u64, entry: u64, kept: bool) -> change::Decision {
    changes::change_step_task(world, task, entry, change::Status::Unknown, Box::new([]), false, kept)
}

fn row(world: &World, task: u64) -> &forge::ChangeRow {
    let Some(forge::Stored::Change(row)) = world.stored().get(&forge::Key::Change(task)) else {
        panic!("change {task} remains durable")
    };
    row
}

#[test]
fn kept_first_yields_to_second_without_losing_its_work() {
    let mut world = World::new(805);
    world.adopt();
    queued(&mut world, FIRST, b"temper/51", b"first", FIRST_READY);
    queued(&mut world, SECOND, b"temper/52", b"second", SECOND_READY);
    let head = row(&world, FIRST).change.last_head;
    assert!(
        matches!(step(&mut world, FIRST, 1, true), change::Decision::Wait { .. }),
        "a kept change must not request a merge"
    );
    assert!(!row(&world, FIRST).change.owns_turn, "kept first must release the turn");
    assert_eq!(row(&world, FIRST).change.last_head, head, "keep preserves the pushed head");
    assert_eq!(row(&world, FIRST).change.ready_since, Some(FIRST_READY), "keep preserves first ready time");
    assert!(
        matches!(step(&mut world, SECOND, 2, false), change::Decision::Effect(change::Effect::Merge { .. })),
        "the second change must progress while the first is kept"
    );
    assert!(
        world.stored().iter().any(|(_, record)| matches!(record, forge::Stored::Landed { task: SECOND, .. })),
        "the fake forge must actually merge the second change"
    );
}

fn returns_at_original_place(restart: bool) {
    let mut world = World::new(806);
    world.adopt();
    queued(&mut world, FIRST, b"temper/51", b"first", FIRST_READY);
    queued(&mut world, SECOND, b"temper/52", b"second", SECOND_READY);
    assert!(matches!(step(&mut world, FIRST, 1, true), change::Decision::Wait { .. }));
    assert!(matches!(step(&mut world, SECOND, 2, true), change::Decision::Wait { .. }));
    if restart {
        world.restart();
        world.run_for(1);
    }
    assert!(
        matches!(step(&mut world, FIRST, 3, false), change::Decision::Ready),
        "Met makes the kept head ready without producing it again"
    );
    assert!(matches!(step(&mut world, SECOND, 4, false), change::Decision::Ready));
    assert!(
        matches!(step(&mut world, SECOND, 5, false), change::Decision::Wait { .. }),
        "later-ready change cannot overtake a returning earlier change"
    );
    assert_eq!(row(&world, FIRST).change.ready_since, Some(FIRST_READY), "return must retain the original queue age");
    assert!(
        matches!(step(&mut world, FIRST, 6, false), change::Decision::Effect(change::Effect::Merge { .. })),
        "returning first must own the next merge"
    );
}

#[test]
fn met_restores_original_queue_order() {
    returns_at_original_place(false);
}

#[test]
fn restart_rebuilds_original_queue_order_after_met() {
    returns_at_original_place(true);
}

#[test]
fn restart_does_not_put_a_still_kept_change_back_in_the_queue() {
    let mut world = World::new(807);
    world.adopt();
    queued(&mut world, FIRST, b"temper/51", b"first", FIRST_READY);
    queued(&mut world, SECOND, b"temper/52", b"second", SECOND_READY);
    assert!(matches!(step(&mut world, FIRST, 1, true), change::Decision::Wait { .. }));
    world.restart();
    world.run_for(1);
    assert!(
        matches!(step(&mut world, SECOND, 2, false), change::Decision::Effect(change::Effect::Merge { .. })),
        "rebuilding the queue must exclude a kept change even before it steps again"
    );
    assert_eq!(
        row(&world, FIRST).change.last_head,
        Some(temper_engine_forge_world::translate::commit(world.branch(b"temper/51"))),
        "restart preserves the waiting branch head"
    );
}

fn reviewed_step(world: &mut World, entry: u64, head: skein_forge::api::Commit, kept: bool) -> change::Decision {
    changes::change_step_task(
        world,
        FIRST,
        entry,
        change::Status::Unknown,
        Box::new([change::GateReport { number: 9, head, status: change::Status::Passed }]),
        false,
        kept,
    )
}

#[test]
fn same_head_keeps_its_blocking_verdict_across_keep_and_restart() {
    let (mut world, head) = changes::reviewed_change(false, change::Freshness::Exact);
    assert!(matches!(reviewed_step(&mut world, 1, head, false), change::Decision::Ready));
    let ready_since = row(&world, FIRST).change.ready_since;
    assert!(matches!(reviewed_step(&mut world, 2, head, true), change::Decision::Wait { .. }));
    world.restart();
    world.run_for(1);
    assert!(
        matches!(reviewed_step(&mut world, 3, head, false), change::Decision::Ready),
        "Met restores readiness at the unchanged head without another review"
    );
    assert_eq!(row(&world, FIRST).change.ready_since, ready_since);
    assert!(
        matches!(reviewed_step(&mut world, 4, head, false), change::Decision::Effect(change::Effect::Merge { .. })),
        "the retained approval permits landing after Met"
    );
}

#[test]
fn met_cannot_make_a_gate_verdict_for_another_head_ready() {
    let (mut world, head) = changes::reviewed_change(false, change::Freshness::Exact);
    assert!(matches!(reviewed_step(&mut world, 1, head, false), change::Decision::Ready));
    assert!(matches!(reviewed_step(&mut world, 2, head, true), change::Decision::Wait { .. }));
    let other_head = temper_engine_forge_world::translate::commit(world.branch(b"main"));
    assert_ne!(other_head, head);
    let decision = reviewed_step(&mut world, 3, other_head, false);
    assert!(
        !matches!(decision, change::Decision::Ready | change::Decision::Effect(change::Effect::Merge { .. })),
        "an approval for a different head cannot restore readiness: {decision:?}"
    );
}
