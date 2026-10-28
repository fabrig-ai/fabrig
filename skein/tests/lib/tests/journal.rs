//! The journal against a plain queue model over seeded sequences.

use skein_lib_tests::journal::check;

#[test]
fn journal_follows_the_model() {
    check(0x3A_01, 300);
}

use skein_lib::{Journal, JournalLimits, JournalRoom, Queue, Released};

#[test]
fn an_uncapped_queue_takes_every_push() {
    let mut queue = Queue::uncapped();
    assert_eq!(queue.capacity(), u32::MAX);
    for n in 0..100_000 {
        assert_eq!(queue.room(), u32::MAX - n);
        assert_eq!(queue.try_push(n), Ok(()));
        assert_eq!(queue.room(), u32::MAX - n - 1);
    }
    for n in 0..100_000 {
        assert_eq!(queue.pop(), Some(n));
    }
    assert!(queue.is_empty());
    assert_eq!(queue.room(), u32::MAX);
}

#[test]
fn an_uncapped_journal_takes_every_decision() {
    let mut journal = Journal::uncapped(4, 0);
    for n in 0_u32..1000 {
        let mut decision = journal.decide();
        for write in 0..3 {
            decision.push_write(n * 3 + write);
        }
        decision.push_held(n * 2);
        decision.push_held(n * 2 + 1);
        journal.accept(decision);
        assert_eq!(journal.backlog(), u64::from(n + 1) * 3);
    }
    for n in 0_u32..1000 {
        let mut commit = journal.commit().expect("one commit per decision");
        assert_eq!(commit.number, u64::from(n + 1));
        for write in 0..3 {
            assert_eq!(commit.writes.pop(), Some(n * 3 + write));
        }
        assert!(commit.writes.is_empty());
        assert_eq!(journal.backlog(), 3000);
    }
    assert!(journal.commit().is_none());
    journal.committed(1000);
    assert_eq!(journal.backlog(), 2000);
    let mut out = Queue::uncapped();
    for batch in 0..500 {
        assert_eq!(journal.release(&mut out), Released::Some);
        assert_eq!(out.len(), 4);
        for n in 0..4 {
            assert_eq!(out.pop(), Some(batch * 4 + n));
        }
    }
    assert_eq!(journal.release(&mut out), Released::None);
    assert_eq!(journal.backlog(), 0);
    assert!(journal.idle());
}

#[test]
fn backlog_counts_commits_unanswered_held_and_the_door() {
    let limits = JournalLimits { commits: 2, writes: 2, held: 4, now: 2, release: 1 };
    let mut journal = Journal::new(&limits);
    assert_eq!(journal.backlog(), 0);
    let mut decision = journal.decision(&JournalRoom { writes: 1, held: 1 }).expect("room");
    decision.push_write(10);
    decision.push_held(20);
    assert_eq!(journal.backlog(), 0, "an open decision has not been accepted");
    journal.accept(decision);
    assert_eq!(journal.backlog(), 2);
    let mut decision = journal.decision(&JournalRoom { writes: 0, held: 1 }).expect("room");
    decision.push_held(21);
    journal.accept(decision);
    assert_eq!(journal.backlog(), 3);
    journal.now(30).expect("door room");
    assert_eq!(journal.backlog(), 4);
    let mut out = Queue::uncapped();
    assert_eq!(journal.release(&mut out), Released::Some);
    assert_eq!(out.pop(), Some(30));
    assert_eq!(journal.backlog(), 3);
    assert_eq!(journal.release(&mut out), Released::None);
    assert_eq!(journal.commit().expect("pending commit").number, 1);
    assert_eq!(journal.backlog(), 3, "sent commits remain unanswered");
    journal.committed(1);
    assert_eq!(journal.backlog(), 2);
    for output in [20_u32, 21] {
        assert_eq!(journal.release(&mut out), Released::Some);
        assert_eq!(out.pop(), Some(output));
        assert_eq!(journal.backlog(), u64::from(21 - output));
    }
    assert!(journal.idle());
}

#[test]
fn a_stopped_uncapped_journal_discards_its_decisions() {
    let mut journal = Journal::uncapped(4, 7);
    let mut decision = journal.decide();
    decision.push_write(10);
    decision.push_held(20);
    journal.accept(decision);
    assert_eq!(journal.commit().expect("numbering follows durable").number, 8);
    journal.failed(8);
    assert!(journal.stopped());
    let backlog = journal.backlog();
    let mut decision = journal.decide();
    decision.push_write(11);
    decision.push_held(21);
    journal.accept(decision);
    assert_eq!(journal.backlog(), backlog);
    assert_eq!(journal.now(30), Err(30));
    assert!(journal.commit().is_none());
    assert_eq!(journal.release(&mut Queue::uncapped()), Released::Stopped);
}

#[test]
#[should_panic(expected = "only one decision may be open")]
fn deciding_twice_at_once_is_a_bug() {
    let mut journal = Journal::<u8, u8>::uncapped(4, 0);
    let _first = journal.decide();
    let _second = journal.decide();
}

#[test]
#[should_panic(expected = "decide requires an uncapped journal")]
fn deciding_on_a_capped_journal_is_a_bug() {
    let limits = JournalLimits { commits: 2, writes: 2, held: 4, now: 2, release: 1 };
    let mut journal = Journal::<u8, u8>::new(&limits);
    let _decision = journal.decide();
}

#[test]
fn an_uncapped_journal_refuses_reserved_decisions() {
    let mut journal = Journal::<u8, u8>::uncapped(4, 0);
    let room = JournalRoom { writes: 0, held: 0 };
    assert!(!journal.takes(&room));
    assert!(journal.decision(&room).is_none());
}

#[test]
#[should_panic(expected = "a pushed write fits the decision's room")]
fn pushing_past_reserved_writes_is_a_bug() {
    let limits = JournalLimits { commits: 1, writes: 1, held: 1, now: 1, release: 1 };
    let mut journal = Journal::<u8, u8>::new(&limits);
    let mut decision = journal.decision(&JournalRoom { writes: 1, held: 0 }).expect("room");
    decision.push_write(1);
    decision.push_write(2);
}

#[test]
#[should_panic(expected = "a pushed output fits the decision's room")]
fn pushing_past_reserved_outputs_is_a_bug() {
    let limits = JournalLimits { commits: 1, writes: 1, held: 1, now: 1, release: 1 };
    let mut journal = Journal::<u8, u8>::new(&limits);
    let mut decision = journal.decision(&JournalRoom { writes: 0, held: 1 }).expect("room");
    decision.push_held(1);
    decision.push_held(2);
}
