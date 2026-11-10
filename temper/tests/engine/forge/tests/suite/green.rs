//! A green head ends CI repair alone (temper/docs/design/domain/forge/change/repairs.md).
use skein_fake_forge_domain::api as raw;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::changes::{change_row, change_step, change_step_task_running, change_step_with_release};
use temper_engine_forge_world::{World, translate};

const TASK: u64 = 51;
const REPAIR: u64 = 52;
const BRANCH: &[u8] = b"temper/51";

fn repair(why: change::Repair, requested: bool, live: bool) -> (World, skein_forge::api::Commit) {
    let mut world = World::new(219);
    world.adopt();
    world.landing_ci(b"main", true);
    world.produce(BRANCH);
    world.status(BRANCH, raw::Check::Passed);
    let pull = world.open_pull(BRANCH, b"main");
    let head = translate::commit(world.branch(BRANCH));
    let mut row = change_row();
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = change::State::Repairing { requested, why };
    row.change.repairs = u32::from(requested);
    if live {
        row.delegate = Some((REPAIR, change::Delegate::Repair(why)));
    }
    world.event(top::Event::Change { row });
    (world, head)
}

fn row(world: &World) -> &top::ChangeRow {
    let Some(top::Stored::Change(row)) = world.stored().get(&top::Key::Change(TASK)) else { panic!("change row") };
    row
}

fn released(world: &mut World, why: change::Hold) {
    let mut saved = row(world).clone();
    saved.change.state = change::State::Held { was: Box::new(saved.change.state), why };
    saved.delegate = None;
    saved.delegate_status = change::Status::Failed;
    world.event(top::Event::Restore { record: top::Stored::Change(Box::new(saved)) });
}

#[test]
fn a_green_head_ends_a_due_ci_repair() {
    for why in [change::Repair::Ci, change::Repair::Semantic] {
        let (mut world, _) = repair(why, false, false);
        assert_eq!(change_step(&mut world, 1, change::Status::Unknown, Box::new([])), change::Decision::Ready);
        assert_eq!(row(&world).change.repairs, 0);
        assert!(row(&world).delegate.is_none());
    }
}

#[test]
fn a_green_head_cancels_a_repair_not_running_then_goes_on() {
    let (mut world, head) = repair(change::Repair::Ci, true, true);
    let decision = change_step(&mut world, 1, change::Status::Pending, Box::new([]));
    assert_eq!(decision, change::Decision::CancelRepair { head });
    world.event(top::Event::DelegateResult {
        task: TASK,
        child: REPAIR,
        status: change::Status::Failed,
        no_change: false,
        failed: false,
        cancelled: true,
        words: b"CI passed at head".as_slice().into(),
    });
    assert_eq!(change_step(&mut world, 2, change::Status::Unknown, Box::new([])), change::Decision::Ready);
    assert!(row(&world).producer_failure.is_none());
    assert_eq!(row(&world).delegate_status, change::Status::Unknown);
    assert_eq!(row(&world).change.repairs, 1);
}

#[test]
fn a_run_in_progress_is_left_to_finish() {
    let (mut world, head) = repair(change::Repair::Ci, true, true);
    let decision =
        change_step_task_running(&mut world, TASK, 1, change::Status::Pending, Box::new([]), false, false, true);
    let change::Decision::Wait { until: Some(until) } = decision else { panic!("running repair waits: {decision:?}") };
    // The fixture's client poll is five seconds.
    assert!(until.as_nanos() <= world.wall().as_nanos().saturating_add(5_000_000_000));
    assert_eq!(row(&world).delegate, Some((REPAIR, change::Delegate::Repair(change::Repair::Ci))));
    world.push(BRANCH, b"repair delivered");
    let pushed = translate::commit(world.branch(BRANCH));
    assert_ne!(head, pushed);
    world.status(BRANCH, raw::Check::Passed);
    let decision = change_step(&mut world, 2, change::Status::Passed, Box::new([]));
    assert_eq!(decision, change::Decision::Ready);
    assert!(matches!(row(&world).change.state, change::State::Queued { head, .. } if head == pushed));
    assert_eq!(row(&world).result.head, Some(pushed));
    assert_eq!(row(&world).change.repairs, 1);
}

#[test]
fn a_release_at_a_green_head_makes_no_fresh_repair() {
    let (mut world, _) = repair(change::Repair::Ci, true, false);
    released(&mut world, change::Hold::Failed);
    assert_eq!(
        change_step_with_release(&mut world, 1, change::Status::Unknown, Box::new([]), true),
        change::Decision::Ready
    );
    assert_eq!(row(&world).change.repairs, 0);
    assert!(row(&world).delegate.is_none());
}

#[test]
fn a_release_at_a_failing_head_still_asks_a_fresh_repair() {
    let (mut world, _) = repair(change::Repair::Ci, true, false);
    world.status(BRANCH, raw::Check::Failed);
    released(&mut world, change::Hold::Failed);
    assert_eq!(
        change_step_with_release(&mut world, 1, change::Status::Unknown, Box::new([]), true),
        change::Decision::Delegate(change::Delegate::Repair(change::Repair::Ci))
    );
    assert_eq!(row(&world).change.repairs, 1);
}

#[test]
fn ci_passed_on_an_earlier_head_changes_nothing() {
    let (mut world, _) = repair(change::Repair::Ci, true, true);

    let earlier = translate::commit(world.branch(BRANCH));
    world.push(BRANCH, b"new repair head");
    world.run_for(3);
    world.status(BRANCH, raw::Check::Failed);
    world.event(top::Event::Enqueue {
        entry: skein_forge_client::Entry {
            number: 90,
            task: TASK,
            repository: temper_engine_forge_world::REPO,
            effect: skein_forge_client::Effect {
                write: skein_forge::api::Write::Status {
                    commit: earlier,
                    context: b"build".as_slice().into(),
                    check: skein_forge::api::Check::Passed,
                    description: Box::new([]),
                    link: Box::new([]),
                },
                condition: skein_forge_client::Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    });
    world.event(top::Event::Committed { entry: 90 });
    world.run_for(1);
    let decision = change_step(&mut world, 91, change::Status::Pending, Box::new([]));
    assert!(matches!(decision, change::Decision::Wait { .. }), "{decision:?}, {:?}", row(&world));
    assert!(matches!(row(&world).change.state, change::State::Repairing { .. }));
}

#[test]
fn a_review_repair_is_untouched_at_a_green_head() {
    let (mut world, _) = repair(change::Repair::Gate(9), true, true);
    assert!(matches!(change_step(&mut world, 1, change::Status::Pending, Box::new([])), change::Decision::Wait { .. }));
    released(&mut world, change::Hold::Failed);
    assert_eq!(
        change_step_with_release(&mut world, 2, change::Status::Unknown, Box::new([]), true),
        change::Decision::Delegate(change::Delegate::Repair(change::Repair::Gate(9)))
    );
}
