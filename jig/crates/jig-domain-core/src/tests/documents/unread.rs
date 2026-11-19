//! Task documents list only directing words no run has recorded.
use crate::tests::*;

fn word(number: u64, from: tasks::Party, words: &[u8]) -> tasks::Word {
    tasks::Word {
        number,
        from,
        kind: tasks::MessageKind::Words,
        words: Box::from(words),
        at: Wall::from_nanos(number.checked_mul(1_000_000_000).unwrap()),
        hits: 1,
        eligible: true,
    }
}

#[test]
fn task_documents_merge_unrecorded_and_inbox_directing_words() {
    let (core, mut limits) = make_tree();
    limits.tasks.message_bytes = 300;
    limits.tasks.inbox_bytes = 1200;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.requester = tasks::Party::Person(42);
    let person = tasks::Party::Person(42);
    row.unrecorded = Box::new([word(77, person, b"Also add a /ready endpoint beside /health.")]);
    row.inbox = Box::new([word(80, person, b"And keep the old route."), word(79, tasks::Party::Task(3), b"news")]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    let expected =
        [(77, true, b"Also add a /ready endpoint beside /health.".as_slice()), (80, false, b"And keep the old route.")];
    assert_eq!(task.unread.len(), 2);
    assert_eq!(task.unread.capacity(), limits.tasks.inbox_messages);
    for (actual, (message, seen, words)) in task.unread.iter().zip(expected) {
        assert_eq!(actual.message, message);
        assert_eq!(actual.by, person);
        assert_eq!(actual.at, Wall::from_nanos(message.checked_mul(1_000_000_000).unwrap()));
        assert_eq!(actual.seen, seen);
        assert_eq!(actual.head.bytes.as_ref(), words);
        assert_eq!(actual.bytes, u32::try_from(words.len()).unwrap());
    }
    // The 256-byte boundary lies inside a three-byte character.
    let words = "界".repeat(100);
    row.unrecorded = Box::new([]);
    row.inbox = Box::new([word(81, person, words.as_bytes())]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    let head = task.unread.get(0).unwrap();
    assert_eq!(head.head.bytes.as_ref(), "界".repeat(85).as_bytes());
    assert_eq!(head.bytes, 300);
    row.requester = tasks::Party::Task(3);
    row.inbox = Box::new([word(82, tasks::Party::Task(3), b"requester"), word(83, tasks::Party::Task(4), b"peer")]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert_eq!(task.unread.len(), 1);
    assert_eq!(task.unread.get(0).unwrap().by, tasks::Party::Task(3));
    row.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) }));
    assert!(task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap().unread.is_empty());
    row.phase = tasks::Phase::Waiting;
    row.inbox = Box::new([]);
    assert!(task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap().unread.is_empty());
}
