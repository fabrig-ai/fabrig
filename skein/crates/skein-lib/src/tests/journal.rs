use crate::{Journal, Queue, Released};

// Two outputs may leave in each call.
const RELEASE: u32 = 2;

#[test]
fn one_numbered_commit_per_writing_decision() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut first = journal.decide();
    first.push_write(3);
    first.push_write(4);
    journal.accept(first);
    let empty = journal.decide();
    journal.accept(empty);
    let mut second = journal.decide();
    second.push_write(5);
    journal.accept(second);
    let mut first_commit = journal.commit().expect("first commit");
    assert_eq!(first_commit.number, 1);
    assert_eq!(first_commit.writes.pop(), Some(3));
    assert_eq!(first_commit.writes.pop(), Some(4));
    let mut second_commit = journal.commit().expect("second commit");
    assert_eq!(second_commit.number, 2);
    assert_eq!(second_commit.writes.pop(), Some(5));
    assert!(journal.commit().is_none());
}

#[test]
fn held_outputs_wait_for_their_commit_and_leave_in_order() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut out = Queue::with_capacity(4);
    let mut first = journal.decide();
    first.push_write(1);
    first.push_held(10);
    journal.accept(first);
    let mut second = journal.decide();
    second.push_write(2);
    second.push_held(20);
    journal.accept(second);
    assert_eq!(journal.release(&mut out), Released::None);
    assert_eq!(journal.commit().expect("first").number, 1);
    assert_eq!(journal.commit().expect("second").number, 2);
    journal.committed(1);
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(10));
    assert_eq!(out.pop(), None);
    journal.committed(2);
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(20));
    assert!(!journal.stopped());
}

#[test]
fn no_write_follows_last_commit_or_is_ready_at_zero() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut out = Queue::with_capacity(3);
    let mut initial = journal.decide();
    initial.push_held(1);
    journal.accept(initial);
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(1));
    let mut writing = journal.decide();
    writing.push_write(7);
    journal.accept(writing);
    let mut following = journal.decide();
    following.push_held(2);
    journal.accept(following);
    assert_eq!(journal.release(&mut out), Released::None);
    assert_eq!(journal.commit().expect("commit").number, 1);
    journal.committed(1);
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(2));
}

#[test]
fn release_is_bounded_per_call() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut out = Queue::with_capacity(4);
    assert_eq!(journal.now(1), Ok(()));
    assert_eq!(journal.now(2), Ok(()));
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(1));
    assert_eq!(out.pop(), Some(2));
    let mut decision = journal.decide();
    decision.push_held(4);
    decision.push_held(5);
    journal.accept(decision);
    assert_eq!(journal.now(3), Ok(()));
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(3));
    assert_eq!(out.pop(), Some(4));
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(5));
}

#[test]
fn failure_stops_release_and_returns_door_outputs() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut out = Queue::with_capacity(2);
    let mut decision = journal.decide();
    decision.push_write(1);
    decision.push_held(10);
    journal.accept(decision);
    assert_eq!(journal.commit().expect("commit").number, 1);
    journal.failed(1);
    assert!(journal.stopped());
    assert_eq!(journal.release(&mut out), Released::Stopped);
    assert_eq!(out.pop(), None);
    assert_eq!(journal.now(2), Err(2));
}

fn journal_with_held_and_door_outputs() -> (Journal<u8, u8>, u64) {
    let mut journal = Journal::uncapped(RELEASE, 0);
    let mut decision = journal.decide();
    decision.push_write(1);
    decision.push_held(10);
    journal.accept(decision);
    let number = journal.commit().expect("sent commit").number;
    assert_eq!(journal.now(20), Ok(()));
    assert_eq!(journal.backlog(), 3);
    (journal, number)
}

fn assert_stopped_outputs(journal: &mut Journal<u8, u8>, number: u64) {
    let mut out = Queue::with_capacity(2);
    journal.committed(number);
    assert_eq!(journal.release(&mut out), Released::Stopped);
    assert_eq!(out.pop(), None);
    assert!(journal.stopped());
    assert_eq!(journal.backlog(), 3, "the sent commit, held output and door output remain");
}

#[test]
fn stop_keeps_held_and_door_outputs_from_leaving() {
    let (mut journal, number) = journal_with_held_and_door_outputs();
    journal.stop();
    assert_stopped_outputs(&mut journal, number);
}

#[test]
fn stopping_twice_is_stopping_once() {
    let (mut journal, number) = journal_with_held_and_door_outputs();
    journal.stop();
    journal.stop();
    assert_stopped_outputs(&mut journal, number);
}

#[test]
fn stop_on_a_journal_with_no_commit_stops_it() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut out = Queue::with_capacity(2);
    journal.stop();
    assert_eq!(journal.release(&mut out), Released::Stopped);
    assert_eq!(out.pop(), None);
    assert!(journal.commit().is_none());
    assert!(journal.stopped());
}

#[test]
fn answer_for_unsent_or_old_commit_stops() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut decision = journal.decide();
    decision.push_write(1);
    journal.accept(decision);
    journal.committed(1);
    assert!(journal.stopped());

    let mut another = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut decision = another.decide();
    decision.push_write(1);
    another.accept(decision);
    assert_eq!(another.commit().expect("commit").number, 1);
    another.committed(1);
    another.committed(1);
    assert!(another.stopped());
}

#[test]
fn restored_journal_continues_numbering_and_releases_after_durability() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 42);
    let mut out = Queue::with_capacity(2);
    assert!(journal.idle());

    let mut before = journal.decide();
    before.push_held(7);
    journal.accept(before);
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(7));

    for (write, output) in [(1, 10), (2, 20)] {
        let mut decision = journal.decide();
        decision.push_write(write);
        decision.push_held(output);
        journal.accept(decision);
    }
    assert_eq!(journal.release(&mut out), Released::None);
    assert_eq!(journal.commit().expect("first").number, 43);
    assert_eq!(journal.commit().expect("second").number, 44);
    journal.committed(43);
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(10));
    assert_eq!(journal.release(&mut out), Released::None);
    journal.committed(44);
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(20));
    assert!(journal.idle());
}

#[test]
fn idle_tracks_open_decisions_commits_outputs_and_stop() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut out = Queue::with_capacity(2);
    assert!(journal.idle());
    let mut decision = journal.decide();
    assert!(!journal.idle());
    decision.push_write(1);
    decision.push_held(10);
    journal.accept(decision);
    assert!(!journal.idle());
    assert_eq!(journal.commit().expect("commit").number, 1);
    assert!(!journal.idle());
    journal.committed(1);
    assert!(!journal.idle());
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(10));
    assert!(journal.idle());
    assert_eq!(journal.now(20), Ok(()));
    assert!(!journal.idle());
    assert_eq!(journal.release(&mut out), Released::Some);
    assert!(journal.idle());
    assert_eq!(out.pop(), Some(20));
    journal.failed(1);
    assert!(journal.stopped());
    assert!(!journal.idle());
}

#[test]
fn restored_maximum_number_allows_unwritten_outputs() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, u64::MAX);
    let mut decision = journal.decide();
    decision.push_held(9);
    journal.accept(decision);
    let mut out = Queue::with_capacity(1);
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(9));
    assert!(journal.idle());
}

#[test]
#[should_panic(expected = "commit numbering exhausted at u64::MAX")]
fn restored_maximum_number_panics_on_writing_accept() {
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, u64::MAX);
    let mut decision = journal.decide();
    decision.push_write(1);
    journal.accept(decision);
}

#[test]
fn compatibility_forms_push_writes_and_held_outputs() {
    // Four held outputs fill two release calls.
    const ITEMS: u8 = 4;
    let mut journal = Journal::<u8, u8>::uncapped(RELEASE, 0);
    let mut decision = journal.decide();
    for value in 0..ITEMS {
        assert_eq!(decision.write(value), Ok(()));
        assert_eq!(decision.hold(value), Ok(()));
    }
    journal.accept(decision);
    let mut commit = journal.commit().expect("one writing decision");
    for value in 0..ITEMS {
        assert_eq!(commit.writes.pop(), Some(value));
    }
    assert!(commit.writes.is_empty());
    journal.committed(commit.number);
    let mut out = Queue::uncapped();
    for [first, second] in [[0, 1], [2, 3]] {
        assert_eq!(journal.release(&mut out), Released::Some);
        assert_eq!(out.pop(), Some(first));
        assert_eq!(out.pop(), Some(second));
        assert!(out.is_empty());
    }
    assert!(journal.idle());
}
