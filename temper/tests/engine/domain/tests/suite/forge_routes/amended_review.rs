//! Re-ask at the same head after amendment (gates.md, 1, Valid at a spec revision).
use super::producer_amendment::amend;
use super::questions::{make_change, next_producer, plan_world_with_gate, row};
use super::reviews::{assert_review_head, finish_review, next_assignment, produce};
use super::*;
use temper_engine_domain_forge_change::{Delegate, State, Status};

const TICKS: u32 = 1000;

fn approved() -> tasks::TaskResult {
    tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() }
}

fn reviewing() -> (World, engine::Assignment, u64, engine::Assignment, [u8; 32]) {
    // An outstanding person's approval keeps the first review from merging.
    let (mut world, requester) = plan_world_with_gate(
        None,
        Gate { agent: true, approval: Some(engine::Freshness::Exact), ..Gate::default() },
        &StoryLimits { batch: Some(2), ..StoryLimits::default() },
        false,
    );
    let change = make_change(&mut world, &requester, 93, engine::ChangeQuestions::None);
    let producer = next_producer(&mut world, change);
    let branch = row(&world, change).branch.clone();
    let head = produce(&mut world, &producer, &branch);
    let first = next_assignment(&mut world, 3);
    assert_review_head(&world, &first, head);
    assert_eq!(row(&world, change).gate_revision, 0);
    (world, requester, change, first, head)
}

fn finish_and_land(world: &mut World, change: u64, second: &engine::Assignment) {
    finish_review(world, second, approved());
    for _ in 0..TICKS {
        world.tick();
        if row(world, change).verdicts.iter().any(|report| report.revision == 1 && report.status == Status::Passed) {
            break;
        }
    }
    assert_eq!(world.merges, 0, "the person's approval is still pending");
    let pull = row(world, change).pull.expect("opened pull");
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: b"Approved".as_slice().into()
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(pull),
            key: None,
        },
    })));
    for _ in 0..TICKS {
        world.tick();
        if matches!(row(world, change).change.state, State::Landed { .. }) {
            break;
        }
    }
    let change = row(world, change);
    assert!(matches!(change.change.state, State::Landed { .. }), "{change:?}");
    assert_eq!(world.merges, 1);
    assert_eq!(change.change.repairs, 0);
    assert!(change.change.gate_failures.is_empty());
}

#[test]
fn an_approved_review_is_asked_again_after_the_change_is_amended() {
    let (mut world, requester, change, first, head) = reviewing();
    finish_review(&mut world, &first, approved());
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).verdicts.iter().any(|report| report.status == Status::Passed) {
            break;
        }
    }
    assert_eq!(row(&world, change).verdicts[0].revision, 0);
    assert_eq!(world.merges, 0);
    amend(&mut world, &requester, change, 94, Some(b"Review the CLI too"), None);
    let second = next_assignment(&mut world, 4);
    assert_ne!(second.task, first.task);
    assert_review_head(&world, &second, head);
    assert_eq!(row(&world, change).delegate, Some((second.task, Delegate::Gate { number: 1, head })));
    finish_and_land(&mut world, change, &second);
}

#[test]
fn a_review_running_during_amendment_finishes_at_its_old_revision() {
    let (mut world, requester, change, first, head) = reviewing();
    amend(&mut world, &requester, change, 94, Some(b"Review the CLI too"), None);
    assert_eq!(
        row(&world, change).delegate,
        Some((first.task, Delegate::Gate { number: 1, head })),
        "the old review runs on"
    );
    finish_review(&mut world, &first, approved());
    let second = next_assignment(&mut world, 4);
    assert_ne!(second.task, first.task);
    assert_review_head(&world, &second, head);
    assert_eq!(row(&world, change).verdicts[0].revision, 0);
    assert_eq!(row(&world, change).gate_revision, 1);
    finish_and_land(&mut world, change, &second);
}
