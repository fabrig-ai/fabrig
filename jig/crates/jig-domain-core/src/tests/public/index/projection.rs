//! Public lines take only their field sources (jig/docs/design/client/domain/contract.md, 12).
use super::*;
use crate::{PublicExecutor, PublicHold, PublicLine, PublicPhase, documents::Text};

#[test]
fn titles_and_mechanics_are_projected_and_follow_amendments() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(7), false);
    assert_eq!(
        ready(&core).line(1),
        Some(&PublicLine {
            task: 1,
            title: Text { bytes: Box::from(b"Public title".as_slice()), cut: false },
            parent: None,
            goal: true,
            priority: Some(7),
            executor: PublicExecutor::Agent,
            phase: PublicPhase::Due,
            hold: None,
            made: 10,
            changed: 10,
            active: None,
            ended: None,
            spent: 0,
            budget: 100,
            delegates: zero(),
        })
    );
    let spec = tasks::Spec {
        job: None,
        words: Box::from(b"New public title\nNew private specification".as_slice()),
        parameters: Box::new([]),
        inputs: Box::new([]),
        asks: false,
    };
    amend_at(&mut core, &limits, 1, None, Some(spec), Wall::from_nanos(20_000_000_000));
    assert_eq!(ready(&core).line(1).unwrap().changed, 20);
    assert_eq!(
        ready(&core).line(1).unwrap().title,
        Text { bytes: Box::from(b"New public title".as_slice()), cut: false }
    );
    send(&mut core, &limits, tasks::Event::Hold { task: 1, why: tasks::Hold::Dependency(777) });
    assert_eq!(ready(&core).line(1).unwrap().phase, PublicPhase::Held);
    assert_eq!(ready(&core).line(1).unwrap().hold, Some(PublicHold::Dependency));
    assert_eq!(ready(&core).public_tasks().as_ref(), &[1]);
}

#[test]
fn titles_are_cut_at_a_character_boundary_and_no_later_line_crosses() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    let mut words = skein_lib::List::with_capacity(limits.tasks.spec_bytes);
    for _ in 0_u16..255 {
        words.push(b'a').unwrap();
    }
    for &byte in b"\xc3\xa9\nPrivate specification" {
        words.push(byte).unwrap();
    }
    let spec = tasks::Spec {
        job: None,
        words: words.into_boxed(),
        parameters: Box::new([]),
        inputs: Box::new([]),
        asks: false,
    };
    amend(&mut core, &limits, 1, None, Some(spec));
    let title = &ready(&core).line(1).unwrap().title;
    assert_eq!(title.bytes.as_ref(), &[b'a'; 255]);
    assert!(title.cut);
}

#[test]
fn a_public_rename_preserves_ended_history_and_updates_the_name() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    end(&mut core, &limits, 1);
    let before = snapshot(ready(&core));
    let mut child = Queue::uncapped();
    child.push(people::Request::Route {
        request: Token::new(903),
        person: 1,
        project: 1,
        role: Some(people::Role::Owner),
        ask: Box::new(people::Ask::Rename { project: 1, name: Box::from("Public project name") }),
        models: Box::new([]),
    });
    let mut work = Queue::uncapped();
    let _out = crate::routing::party::people::tag_people(&mut core, &environment(&limits), child, &mut work);
    assert_eq!(ready(&core).name.as_ref(), "Public project name");
    assert_eq!(snapshot(ready(&core)), before);
    assert_eq!(ready(&core).ended().len(), 1);
    visibility(&mut core, &limits, true);
    assert_eq!(snapshot(ready(&core)), before, "repeating publication preserves the entry");
}

#[test]
fn tree_order_keeps_a_parents_subtree_before_its_next_sibling() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, Some(1), None, false);
    make(&mut core, &limits, 3, Some(1), None, false);
    make(&mut core, &limits, 4, Some(2), None, false);
    let project = ready(&core);
    assert_eq!(project.tree(1), Some(([2, 4, 3].as_slice(), 0)));
    assert_eq!(project.delegates(1), ([2, 3].as_slice(), 0));
    assert_eq!(project.public_tasks().as_ref(), &[1, 2, 3, 4]);
}

#[test]
fn ended_public_lines_keep_their_outcomes() {
    const FAILED_GOAL: u64 = 20;
    const OTHER_FAILED_GOAL: u64 = 21;
    const COMPLETED_GOAL: u64 = 22;
    const CANCELLED_GOAL: u64 = 23;
    const ENDED_GOALS: u16 = 4; // The two failures, completion and cancellation.
    let (mut core, mut limits) = fixture();
    limits.public.ended = ENDED_GOALS;
    visibility(&mut core, &limits, true);
    for (task, result, expected) in [
        (FAILED_GOAL, tasks::TaskResult::Failure { reason: Box::from(*b"private failure") }, PublicPhase::Failed),
        (
            OTHER_FAILED_GOAL,
            tasks::TaskResult::Failure { reason: Box::from(*b"another private failure") },
            PublicPhase::Failed,
        ),
        (COMPLETED_GOAL, tasks::TaskResult::Report { words: Box::from(*b"private result") }, PublicPhase::Done),
    ] {
        make(&mut core, &limits, task, None, Some(1), false);
        finish_public_goal(&mut core, &limits, task, result);
        assert_eq!(ready(&core).line(task).unwrap().phase, expected, "goal {task}");
    }
    make(&mut core, &limits, CANCELLED_GOAL, None, Some(1), false);
    end(&mut core, &limits, CANCELLED_GOAL);
    assert_eq!(ready(&core).line(CANCELLED_GOAL).unwrap().phase, PublicPhase::Cancelled);
    assert_eq!(ready(&core).ended().len(), usize::from(ENDED_GOALS));
}

fn finish_public_goal(core: &mut Core, limits: &Limits, task: u64, result: tasks::TaskResult) {
    send(core, limits, tasks::Event::Prepare { reply_to: reply(), task });
    send(
        core,
        limits,
        tasks::Event::Claim {
            reply_to: reply(),
            task,
            attempt: task,
            resumed: false,
            budget: 10,
            account: None,
            writes: Box::new([]),
        },
    );
    send(
        core,
        limits,
        tasks::Event::Activation {
            reply_to: reply(),
            task,
            attempt: task,
            end: tasks::End::Finished { result, cancel_delegates: false },
            saved: None,
            cause: tasks::Cause::Priced { cumulative: 0 },
        },
    );
    send(core, limits, tasks::Event::EffectsSettled { task });
    send(core, limits, tasks::Event::Settled { task });
}
