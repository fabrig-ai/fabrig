//! Complete stored ancestry must not keep the live walk's provisional cut.
use super::*;

const PARENT: u64 = 98;
const CHILD: u64 = 99;
// One parent fits the document's one line; two subscriptions exceed it.
const LINES: u32 = 1;

fn task_with_stored_parent(subscriptions: Box<[tasks::Subscription]>) -> Box<Task> {
    let (mut core, mut limits) = watch_core();
    limits.documents.lines = LINES;
    let mut parent = core.tasks.task(1).unwrap().clone();
    parent.number = PARENT;
    parent.root = PARENT;
    parent.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) }));
    let mut child = parent.clone();
    child.number = CHILD;
    child.requester = tasks::Party::Task(PARENT);
    child.depth = 1;
    child.subscriptions = subscriptions;
    let mut row = stored_row(Record::Tasks(tasks::Stored::Ended(Box::new(child))), None);
    row.ancestors = Box::new([PARENT]);
    row.linked = Box::new([Box::new(parent)]);
    let Held::ReadLoad { load, .. } = read_start(&mut core, &limits, 10, reads::Read::Task { task: CHILD }) else {
        panic!("retained task load")
    };
    let reads::Answer::Historical(value) =
        loaded_answer(&mut core, &limits, load, reads::Loaded::Page { rows: Box::new([row]), next: None }).unwrap()
    else {
        panic!("retained task document")
    };
    let reads::Past::Task { document, .. } = value.value else { panic!("task") };
    document
}

#[test]
fn a_complete_stored_lineage_keeps_its_document_whole() {
    let task = task_with_stored_parent(Box::new([]));
    assert!(!task.cut);
    assert_eq!(task.lineage.len(), LINES);
    assert_eq!(task.lineage.get(0).unwrap().number, PARENT);
}

#[test]
fn a_complete_stored_lineage_keeps_another_lists_cut() {
    let task = task_with_stored_parent(Box::new([
        tasks::Subscription { number: 1, kind: tasks::SubscriptionKind::Timer { at: Wall::EPOCH, period: None } },
        tasks::Subscription { number: 2, kind: tasks::SubscriptionKind::Timer { at: Wall::EPOCH, period: None } },
    ]));
    assert!(task.cut);
    assert_eq!(task.subscriptions.len(), LINES);
    assert_eq!(task.lineage.len(), LINES);
    assert_eq!(task.lineage.get(0).unwrap().number, PARENT);
}
