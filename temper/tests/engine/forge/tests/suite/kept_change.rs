//! Adversarial kept-change stories against domain/forge/change/README.md, 2 and domain/forge/queues.md.
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

fn landed_panel(world: &World, task: u64) -> forge::Panel {
    let Some(forge::Document::Change(panel)) = world.document(5, task) else {
        panic!("landed change retains its panel")
    };
    assert!(matches!(panel.step, forge::PanelStep::Landed { .. }));
    panel
}

fn land_pending(world: &mut World, task: u64, branch: &[u8], path: &[u8]) -> [u8; 32] {
    world.produce_file(branch, path, b"change");
    let pull = world.open_pull(branch, b"main");
    world.status(branch, skein_fake_forge_domain::api::Check::Pending);
    let head = temper_engine_forge_world::translate::commit(world.branch(branch));
    let mut change = changes::change_row();
    change.task = task;
    change.branch = branch.into();
    change.pull = Some(pull);
    change.change.task = task;
    change.change.last_head = Some(head);
    change.change.state = change::State::Queued { head, ready_since: FIRST_READY };
    change.change.ready_since = Some(FIRST_READY);
    change.change.owns_turn = true;
    world.event(forge::Event::Change { row: change });
    let decision = step(world, task, task, false);
    assert!(matches!(decision, change::Decision::Effect(change::Effect::Merge { .. })), "{decision:?}");
    assert!(
        world
            .stored()
            .iter()
            .any(|(_, record)| matches!(record, forge::Stored::Landed { task: landed, .. } if *landed == task))
    );
    let _decision = step(world, task, task + 100, false);
    assert!(matches!(row(world, task).change.state, change::State::Landed { .. }));
    assert_eq!(landed_panel(world, task).ci, change::Status::Pending, "landing retains its pending observation");
    head
}

#[test]
fn landed_change_without_required_ci_does_not_stay_pending() {
    let mut world = World::new(810);
    world.adopt();
    world.landing_ci(b"main", false);
    let mut heads = Vec::new();
    for (task, branch, path) in
        [(3, b"temper/3".as_slice(), b"three".as_slice()), (5, b"temper/5", b"five"), (10, b"temper/10", b"ten")]
    {
        let head = land_pending(&mut world, task, branch, path);
        assert!(!heads.contains(&head), "each sighting has an independent head");
        heads.push(head);
        world.status(branch, skein_fake_forge_domain::api::Check::Passed);
        world.run_for(301);
        let skein_fake_forge_domain::api::Answer::Statuses(statuses) =
            world.inspect(&skein_fake_forge_domain::api::Read::Statuses { commit: world.branch(branch) })
        else {
            panic!("forge statuses are readable")
        };
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].state, skein_fake_forge_domain::api::Check::Passed);
        assert_eq!(temper_engine_forge_world::translate::commit(world.branch(branch)), head);
        assert!(!landed_panel(&world, task).ci_required, "no-CI landing explicitly says CI is not required");
    }
    required_ci_control();
    world.restart();
    world.run_for(1);
    for task in [3, 5, 10] {
        assert!(!landed_panel(&world, task).ci_required);
        assert_eq!(landed_panel(&world, task).ci, change::Status::Pending);
    }
}

fn required_ci_control() {
    let mut world = World::new(811);
    world.adopt();
    world.landing_ci(b"main", true);
    world.produce_file(b"temper/11", b"control", b"change");
    let pull = world.open_pull(b"temper/11", b"main");
    world.status(b"temper/11", skein_fake_forge_domain::api::Check::Pending);
    let head = temper_engine_forge_world::translate::commit(world.branch(b"temper/11"));
    let mut control = changes::change_row();
    control.task = 11;
    control.branch = b"temper/11".as_slice().into();
    control.pull = Some(pull);
    control.change.task = 11;
    control.change.last_head = Some(head);
    control.change.state = change::State::Checking { head };
    world.event(forge::Event::Change { row: control });
    assert!(matches!(step(&mut world, 11, 211, false), change::Decision::Wait { .. }));
    let Some(forge::Document::Change(panel)) = world.document(5, 11) else { panic!("control panel") };
    assert!(panel.ci_required, "the required-CI rule still waits for the forge");
    assert_eq!(panel.ci, change::Status::Pending);
    world.status(b"temper/11", skein_fake_forge_domain::api::Check::Passed);
    assert_eq!(step(&mut world, 11, 212, false), change::Decision::Ready);
}
