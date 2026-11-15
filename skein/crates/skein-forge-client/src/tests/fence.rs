//! A fence leaves kept facts untouched until a read sent after it (forge/client.md, 4).
use super::fixtures::keep::{Harness, branch, current, pull, summary, watch};
use super::{LIMITS, at};
use crate::{Event, Limits};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Error, Op, Read};
use skein_lib::Duration;

#[test]
fn a_fenced_pull_answer_keeps_its_open_pull_then_reads_at_hinted() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(pull(), false)]));
    h.clock();
    let item = h.one();
    h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let read = h.one();
    h.answer(read, Answer::Pull(current()));
    h.event(Event::Fence { resource: pull() });
    h.fire(101);
    let item = h.one();
    h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let old = h.one();
    assert_eq!(old.op, Op::Read(Read::Pull { number: 9, ci: true }));
    h.news.clear();
    let kept = h.record(&pull()).clone();
    h.event(Event::Fence { resource: pull() });
    let mut closed = current();
    closed.state = api::State::Closed;
    h.answer(old, Answer::Pull(closed.clone()));
    assert!(h.news.is_empty(), "a closed answer sent before the fence is not news");
    assert_eq!(h.record(&pull()), &kept);
    assert_eq!(h.domain.cached(&pull()), Some(kept.cached.as_ref()));
    assert_eq!(h.domain.next_deadline(), Some(at(102)));
    assert!(h.send().is_none());
    h.fire(102);
    let item = h.one();
    assert_eq!(
        item.op,
        Op::Read(Read::Item { number: 9, after: u64::MAX, since: skein_lib::Time::from_nanos(u64::MAX) })
    );
    h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let fresh = h.one();
    assert_eq!(fresh.op, Op::Read(Read::Pull { number: 9, ci: true }));
    h.answer(fresh, Answer::Pull(closed.clone()));
    assert_eq!(h.news.as_slice(), &[Answer::Pull(closed)]);
}

#[test]
fn a_fenced_branch_answer_cannot_move_its_tip_or_report_drift() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(branch(), false)]));
    let initial = h.one();
    h.answer(initial, Answer::Commit([1; 32]));
    h.event(Event::Pushed { resource: branch(), commit: [1; 32] });
    h.event(Event::Fence { resource: branch() });
    h.fire(101);
    let old = h.one();
    h.event(Event::Fence { resource: branch() });
    h.news.clear();
    let kept = h.record(&branch()).clone();
    h.answer(old, Answer::Commit([2; 32]));
    assert_eq!(h.drift, 0);
    assert!(h.news.is_empty());
    assert_eq!(h.domain.cached(&branch()), Some(kept.cached.as_ref()));
    assert_eq!(h.record(&branch()), &kept);
    assert_eq!(h.domain.next_deadline(), Some(at(102)));
    h.fire(102);
    let fresh = h.one();
    h.answer(fresh, Answer::Commit([3; 32]));
    assert_eq!(h.drift, 1, "a read sent after the fence still judges drift");
    assert_eq!(h.news.as_slice(), &[Answer::Commit([3; 32])]);
}

#[test]
fn a_fence_of_an_unwatched_resource_changes_nothing() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(branch(), false)]));
    h.event(Event::Pushed { resource: branch(), commit: [1; 32] });
    let read = h.one();
    let record = h.record(&branch()).clone();
    let deadline = h.domain.next_deadline();
    h.event(Event::Fence { resource: pull() });
    assert!(h.domain.cached(&pull()).is_none());
    assert_eq!(h.record(&branch()), &record);
    assert_eq!(h.domain.next_deadline(), deadline);
    h.answer(read, Answer::Commit([2; 32]));
    assert_eq!(h.drift, 1);
    assert_eq!(h.news.as_slice(), &[Answer::Commit([2; 32])]);
}

#[test]
fn a_keep_read_queued_before_a_fence_but_sent_after_it_is_current() {
    let mut h = Harness::with_limits(Limits { rate: 1, reserve: 0, window: Duration::from_secs(2), ..LIMITS });
    h.keep(Box::new([watch(branch(), false)]));
    h.event(Event::Pushed { resource: branch(), commit: [1; 32] });
    let initial = h.one();
    h.answer(initial, Answer::Commit([1; 32]));
    h.event(Event::Fence { resource: branch() });
    h.fire(101);
    assert!(h.send().is_none(), "the due read is queued behind the spent window");
    h.event(Event::Fence { resource: branch() });
    h.fire(102);
    let fresh = h.one();
    h.answer(fresh, Answer::Commit([2; 32]));
    assert_eq!(h.drift, 1);
    assert_eq!(h.news.as_slice(), &[Answer::Commit([2; 32])]);
}

#[test]
fn a_rate_refused_keep_read_retried_after_a_fence_is_current() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(branch(), false)]));
    h.event(Event::Pushed { resource: branch(), commit: [1; 32] });
    let old = h.one();
    h.event(Event::Fence { resource: branch() });
    h.event(Event::Answered {
        call: old.call,
        cost: 1,
        free: 0,
        rate: None,
        result: Err(Error::RateLimited { after: Duration::from_secs(1) }),
    });
    h.fire(101);
    let retry = h.one();
    assert_ne!(old.call, retry.call);
    h.answer(retry, Answer::Commit([2; 32]));
    assert_eq!(h.drift, 1);
    assert_eq!(h.news.as_slice(), &[Answer::Commit([2; 32])]);
}
