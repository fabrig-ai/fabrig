//! A head already behind takes its base before CI (temper/docs/design/domain/forge/change/README.md, 2, step 3).
use super::early_start::{make, observe_push, push, stop};
use super::goal_branch::{branch_tip, goal_request};
use super::light_rules::change_row;
use super::reviews::next_assignment;
use super::*;
use temper_engine_domain_forge_change as change;

const STEPS: u32 = 1000; // A full landing with fresh reads and hold release.

// The early-start goal fixture, with this story's landing rule on the goal branch.
pub(super) fn goal_world(up_to_date: bool) -> (World, engine::Assignment, Box<[u8]>, u64) {
    goal_world_stall(up_to_date, None)
}

pub(super) fn goal_world_stall(
    up_to_date: bool,
    stall: Option<Duration>,
) -> (World, engine::Assignment, Box<[u8]>, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { slots: Some(4), stall, ..StoryLimits::default() },
    )
    .with_landing_deployment(Box::new([engine::LandingRule {
        sets: Box::new([]),
        connector: 0,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from).into(),
            last: authority::Last::Open(b"temper".as_slice().into()),
        },
        ci: up_to_date,
        up_to_date,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]));
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 4, Box::new([])));
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    let main = branch_tip(&mut world, &branch);
    (world, goal, branch, main)
}

pub(super) fn finish(world: &mut World, producer: &engine::Assignment) {
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Add change".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

pub(super) fn status(world: &mut World, head: u64, state: raw::Check) {
    assert_eq!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: b"build".as_slice().into(),
                state,
                description: Box::new([]),
                link: Box::new([]),
            })
        ),
        raw::Answer::Done
    );
}

pub(super) fn move_base(world: &mut World, branch: &[u8]) -> u64 {
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"other", b"base moved", 1)
        .expect("base moved after the producer pushed")
}

pub(super) fn assert_landed(world: &mut World, task: u64, head: u64, updates: u32) {
    for _ in 0..STEPS {
        world.tick();
        if matches!(change_row(world, task).change.state, change::State::Landed { .. }) {
            break;
        }
    }
    let row = change_row(world, task);
    assert!(matches!(row.change.state, change::State::Landed { .. }), "{row:?}");
    assert_eq!(row.change.updates, updates);
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { head: merged, .. } if *merged == translate::commit(head))));
}

#[test]
fn a_change_opened_behind_its_base_is_updated_before_its_ci_concludes() {
    let (mut world, goal, branch, main) = goal_world(true);
    let task = make(&mut world, &goal, b"Add change", &[]);
    let producer = next_assignment(&mut world, 1);
    let head = push(&mut world, &producer, main, b"change");
    status(&mut world, head, raw::Check::Pending);
    let base = move_base(&mut world, &branch);
    observe_push(&mut world, &producer);
    finish(&mut world, &producer);
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, task).change.updates != 0 {
            break;
        }
    }
    assert_eq!(change_row(&world, task).change.updates, 1);
    let number = change_row(&world, task).pull.expect("opened pull");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("pull before update");
    };
    assert_eq!(pull.commit, head);
    assert!(pull.statuses.iter().any(|row| row.context.as_ref() == b"build" && row.state == raw::Check::Pending));
    assert!(matches!(change_row(&world, task).change.state,
        change::State::Updating { from, base: taken, .. }
        if from == translate::commit(head) && taken == translate::commit(base)));
    for _ in 0..STEPS {
        world.tick();
        if matches!(change_row(&world, task).change.state, change::State::Checking { .. }) {
            break;
        }
    }
    let change::State::Checking { head: checked } = change_row(&world, task).change.state else {
        panic!("updated head is checking: {:?}", change_row(&world, task));
    };
    let updated = world.fake.branch(b"org/repo", &change_row(&world, task).branch).expect("updated branch");
    assert_ne!(updated, head);
    assert_eq!(change_row(&world, task).change.updates, 1);
    assert_eq!(checked, translate::commit(updated));
    let raw::Answer::Comparison { contains_base, .. } =
        world.external(1, raw::Op::Read(raw::Read::Compare { base, head: updated, page: 1, limit: 10 }))
    else {
        panic!("updated ancestry");
    };
    assert!(contains_base);
    status(&mut world, updated, raw::Check::Passed);
    assert_landed(&mut world, task, updated, 1);
    stop(&mut world, &goal);
}

#[test]
fn a_green_head_behind_its_base_keeps_its_result_until_its_turn() {
    let (mut world, goal, branch, main) = goal_world(true);
    let task = make(&mut world, &goal, b"Add green change", &[]);
    let producer = next_assignment(&mut world, 1);
    let head = push(&mut world, &producer, main, b"change");
    observe_push(&mut world, &producer);
    finish(&mut world, &producer);
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, task).pull.is_some()
            && matches!(change_row(&world, task).change.state, change::State::Checking { .. })
        {
            break;
        }
    }
    assert!(matches!(change_row(&world, task).change.state, change::State::Checking { .. }));
    status(&mut world, head, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
        assert_eq!(change_row(&world, task).change.updates, 0);
        if matches!(change_row(&world, task).change.state, change::State::Queued { .. }) {
            break;
        }
    }
    assert!(
        matches!(change_row(&world, task).change.state, change::State::Queued { .. }),
        "{:?}",
        change_row(&world, task)
    );
    move_base(&mut world, &branch);
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, task).change.updates != 0 {
            break;
        }
    }
    assert_eq!(change_row(&world, task).change.updates, 1);
    assert!(matches!(change_row(&world, task).change.state, change::State::Updating { .. }));
    assert!(change_row(&world, task).change.owns_turn);
    stop(&mut world, &goal);
}

#[test]
fn a_head_behind_under_rules_asking_no_update_is_not_updated() {
    let (mut world, goal, branch, main) = goal_world(false);
    let task = make(&mut world, &goal, b"Add unchecked change", &[]);
    let producer = next_assignment(&mut world, 1);
    let head = push(&mut world, &producer, main, b"change");
    status(&mut world, head, raw::Check::Pending);
    move_base(&mut world, &branch);
    observe_push(&mut world, &producer);
    finish(&mut world, &producer);
    assert_landed(&mut world, task, head, 0);
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Update { .. })));
    stop(&mut world, &goal);
}
