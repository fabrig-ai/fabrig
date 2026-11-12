//! Public lines take only their field sources (jig/docs/design/client/domain/contract/README.md, 10).
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
            spent: Some(0),
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
fn a_goals_line_carries_its_spend_and_a_tasks_carries_none() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    make(&mut core, &limits, 2, Some(1), None, false);
    for task in [1, 2] {
        let mut row = core.tasks.task(task).unwrap().clone();
        row.numbers.spent = 5;
        row.numbers.spent_below = 7;
        crate::public::follow(&mut core, &limits, &row);
    }
    let project = ready(&core);
    assert_eq!(project.line(1).unwrap().spent, Some(12));
    assert_eq!(project.line(2).unwrap().spent, None);
}

#[test]
fn an_ended_line_says_how_it_ended() {
    let report = tasks::TaskResult::Report { words: Box::from(b"Private result".as_slice()) };
    for (ending, expected) in [
        (tasks::Ending::Done(report.clone()), PublicPhase::Done),
        (tasks::Ending::Failed { reason: Box::from(b"Private failure".as_slice()) }, PublicPhase::Failed),
        (
            tasks::Ending::Cancelled {
                by: tasks::Party::Person(1),
                at: Wall::EPOCH,
                reason: Box::from(b"Private cancellation".as_slice()),
                result: None,
            },
            PublicPhase::Cancelled,
        ),
        (
            tasks::Ending::Cancelled {
                by: tasks::Party::Person(1),
                at: Wall::EPOCH,
                reason: Box::from(b"Private cancellation".as_slice()),
                result: Some(report),
            },
            PublicPhase::Cancelled,
        ),
    ] {
        let (mut core, limits) = fixture();
        visibility(&mut core, &limits, true);
        make(&mut core, &limits, 1, None, Some(1), false);
        make(&mut core, &limits, 2, Some(1), None, false);
        let mut delegate = core.tasks.task(2).unwrap().clone();
        end(&mut core, &limits, 2);
        delegate.phase = tasks::Phase::Ended(ending.clone());
        delegate.ended_at = Some(Wall::from_nanos(10_000_000_000));
        crate::public::follow(&mut core, &limits, &delegate);
        let project = ready(&core);
        assert_eq!(project.live(), &[1]);
        assert_eq!(project.line(2).unwrap().phase, expected, "live tree: {ending:?}");
        assert_eq!(project.counts(1), PublicCounts { ended: 1, ..zero() }, "live counts: {ending:?}");
        end(&mut core, &limits, 1);
        let project = ready(&core);
        assert_eq!(project.live(), []);
        let [frozen] = project.ended() else { panic!("one frozen goal") };
        assert_eq!(frozen.line.phase, PublicPhase::Cancelled);
        let [line] = frozen.tree.as_ref() else { panic!("one frozen delegate") };
        assert_eq!(line.phase, expected, "frozen tree: {ending:?}");
        assert_eq!(frozen.line.delegates, PublicCounts { ended: 1, ..zero() }, "frozen counts: {ending:?}");
    }
}
