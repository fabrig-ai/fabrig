use super::fixtures::keep::{Harness, branch, comment, current, issue, pull, review, summary, watch};
use super::{LIMITS, REPO, at};
use crate::{Event, Limits, Resource};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Change, Error, Op, Read};
use skein_lib::Duration;

fn empty_listing(now: u64) -> Answer {
    Answer::Items { cut: false, items: Box::new([]), more: false, now: at(now) }
}
fn empty_object() -> Answer {
    Answer::Item { item: summary(), comments: Box::new([]), more: false }
}
fn changed_object() -> Answer {
    let mut item = summary();
    item.updated = at(136);
    Answer::Item { item, comments: Box::new([]), more: false }
}
fn listing_read(since: u64) -> Read {
    Read::Items { single: false, since: at(since), page: 1, kind: None }
}
fn branch_read() -> Read {
    Read::Branch { branch: Box::from(*b"topic") }
}
fn object_read(after: u64) -> Read {
    let since = if after == u64::MAX { skein_lib::Time::from_nanos(u64::MAX) } else { skein_lib::Time::ZERO };
    Read::Item { number: 9, after, since }
}
// These scripts pin a read's object and cursor; keep.rs pins a participating item's comments' `since`.
fn unpinned(op: Op) -> Op {
    if let Op::Read(Read::Item { number, after, .. }) = &op
        && *after != u64::MAX
    {
        return Op::Read(Read::Item { number: *number, after: *after, since: skein_lib::Time::ZERO });
    }
    op
}
fn read_answer(h: &mut Harness, read: Read, result: Result<Answer, Error>) {
    let sent = h.one();
    assert_eq!(unpinned(sent.op.clone()), Op::Read(read));
    h.event(Event::Answered { call: sent.call, cost: 1, free: 0, rate: None, result });
}

// These rows are scripts at the client's boundary, not reads of its live state.
enum RefusalStep {
    Read(Read, Result<Answer, Error>),
    Fire(u64),
    Quiet,
    Hint(Change),
    Writer(bool),
    Pushed,
    UnchangedTip,
    HintOut(Read, Answer),
    Listed(u64),
}
#[test]
#[expect(clippy::too_many_lines, reason = "input refusal scripts share one boundary driver")]
fn a_refusal_is_kept_until_its_inputs_change() {
    use RefusalStep::{Fire, Hint, HintOut, Listed, Pushed, Quiet, Read as R, UnchangedTip, Writer};
    let compare = Read::Commits { before: [2; 32], after: [1; 32], page: 1 };
    let pull_read = Read::Pull { number: 9, ci: true };
    let reviews = Read::Reviews { number: 9, page: 1, skip: 0 };
    let oversized = Answer::Item { item: summary(), comments: Box::new([comment(1), comment(2)]), more: false };
    // One inbox page/row is the exact bound; two comments exceed rows.
    let rows: [(&str, RefusalCase); 8] = [
        (
            "writer comparison",
            (
                branch(),
                false,
                4,
                Box::new([
                    Pushed,
                    Writer(true),
                    R(branch_read(), Ok(Answer::Commit([2; 32]))),
                    R(compare.clone(), Err(Error::TooLarge(api::Excess::unnamed()))),
                    UnchangedTip,
                    Fire(101),
                    Quiet,
                    Fire(160),
                    R(branch_read(), Ok(Answer::Commit([2; 32]))),
                    Quiet,
                    UnchangedTip,
                    Fire(220),
                    R(branch_read(), Ok(Answer::Commit([2; 32]))),
                    Quiet,
                    UnchangedTip,
                    Hint(Change::Branch(Box::from(*b"topic"))),
                    Fire(221),
                    R(branch_read(), Ok(Answer::Commit([2; 32]))),
                    R(compare, Err(Error::TooLarge(api::Excess::unnamed()))),
                    Fire(281),
                    R(branch_read(), Ok(Answer::Commit([3; 32]))),
                    R(
                        Read::Commits { before: [3; 32], after: [1; 32], page: 1 },
                        Err(Error::TooLarge(api::Excess::unnamed())),
                    ),
                    UnchangedTip,
                    Fire(341),
                    R(branch_read(), Ok(Answer::Commit([1; 32]))),
                    Quiet,
                    Fire(401),
                    R(branch_read(), Ok(Answer::Commit([3; 32]))),
                    R(
                        Read::Commits { before: [3; 32], after: [1; 32], page: 1 },
                        Err(Error::TooLarge(api::Excess::unnamed())),
                    ),
                    Writer(false),
                    R(branch_read(), Ok(Answer::Commit([3; 32]))),
                ]),
            ),
        ),
        (
            "comment pages",
            (
                pull(),
                true,
                1,
                Box::new([
                    R(
                        object_read(0),
                        Ok(Answer::Item { item: summary(), comments: Box::new([comment(1)]), more: true }),
                    ),
                    R(pull_read.clone(), Ok(Answer::Pull(current()))),
                    Quiet,
                    Fire(101),
                    Quiet,
                    Fire(105),
                    R(pull_read.clone(), Ok(Answer::Pull(current()))),
                    Quiet,
                    Fire(115),
                    R(pull_read.clone(), Ok(Answer::Pull(current()))),
                    Quiet,
                    Hint(Change::Item(9)),
                    Fire(116),
                    R(object_read(0), Ok(empty_object())),
                ]),
            ),
        ),
        (
            "review pages",
            (
                pull(),
                true,
                1,
                Box::new([
                    R(object_read(0), Ok(empty_object())),
                    R(pull_read.clone(), Ok(Answer::Pull(current()))),
                    R(
                        reviews.clone(),
                        Ok(Answer::Reviews { reviews: Box::new([review(1, 90)]), more: true, cut: false }),
                    ),
                    Fire(101),
                    Quiet,
                    Fire(105),
                    R(object_read(0), Ok(empty_object())),
                    R(pull_read.clone(), Ok(Answer::Pull(current()))),
                    Quiet,
                    Fire(115),
                    R(object_read(0), Ok(empty_object())),
                    R(pull_read, Ok(Answer::Pull(current()))),
                    Quiet,
                    Hint(Change::Item(9)),
                    Fire(116),
                    R(object_read(0), Ok(empty_object())),
                    R(Read::Pull { number: 9, ci: true }, Ok(Answer::Pull(current()))),
                    R(reviews, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false })),
                ]),
            ),
        ),
        (
            "answer rows",
            (
                issue(),
                true,
                1,
                Box::new([
                    R(object_read(0), Ok(oversized)),
                    Fire(101),
                    Quiet,
                    Fire(160),
                    Quiet,
                    Fire(220),
                    Quiet,
                    Hint(Change::Item(9)),
                    Fire(221),
                    R(object_read(0), Ok(empty_object())),
                ]),
            ),
        ),
        (
            "two phase refusals survive together",
            (
                pull(),
                true,
                2,
                Box::new([
                    R(object_read(0), Ok(empty_object())),
                    R(Read::Pull { number: 9, ci: true }, Ok(Answer::Pull(current()))),
                    R(
                        Read::Reviews { number: 9, page: 1, skip: 0 },
                        Ok(Answer::Reviews { reviews: Box::new([review(1, 90)]), more: true, cut: false }),
                    ),
                    Fire(105),
                    R(
                        object_read(0),
                        Ok(Answer::Item { item: summary(), comments: Box::new([comment(1)]), more: true }),
                    ),
                    R(Read::Pull { number: 9, ci: true }, Ok(Answer::Pull(current()))),
                    Quiet,
                    Fire(115),
                    R(Read::Pull { number: 9, ci: true }, Ok(Answer::Pull(current()))),
                    Quiet,
                    Fire(135),
                    R(Read::Pull { number: 9, ci: true }, Ok(Answer::Pull(current()))),
                    Quiet,
                    Hint(Change::Item(9)),
                    Fire(136),
                    R(object_read(0), Ok(changed_object())),
                    R(Read::Pull { number: 9, ci: true }, Ok(Answer::Pull(current()))),
                    R(
                        Read::Reviews { number: 9, page: 1, skip: 0 },
                        Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
                    ),
                ]),
            ),
        ),
        (
            "pull refusal goes on to owed reviews",
            (
                pull(),
                true,
                1,
                Box::new([
                    R(object_read(0), Ok(empty_object())),
                    R(Read::Pull { number: 9, ci: true }, Err(Error::TooLarge(api::Excess::unnamed()))),
                    R(
                        Read::Reviews { number: 9, page: 1, skip: 0 },
                        Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
                    ),
                    Fire(105),
                    R(object_read(0), Ok(empty_object())),
                    Quiet,
                    Fire(115),
                    R(object_read(0), Ok(empty_object())),
                    Quiet,
                    Listed(115),
                    Fire(117),
                    R(object_read(0), Ok(empty_object())),
                    R(Read::Pull { number: 9, ci: true }, Ok(Answer::Pull(current()))),
                ]),
            ),
        ),
        (
            "hint while object is out",
            (
                pull(),
                true,
                1,
                Box::new([
                    HintOut(
                        object_read(0),
                        Answer::Item { item: summary(), comments: Box::new([comment(1)]), more: true },
                    ),
                    R(Read::Pull { number: 9, ci: true }, Ok(Answer::Pull(current()))),
                    Quiet,
                    Fire(101),
                    R(object_read(0), Ok(empty_object())),
                ]),
            ),
        ),
        (
            "branch phase refusal",
            (
                branch(),
                false,
                1,
                Box::new([
                    R(branch_read(), Err(Error::TooLarge(api::Excess::unnamed()))),
                    Fire(160),
                    Quiet,
                    Fire(220),
                    Quiet,
                    Hint(Change::Branch(Box::from(*b"topic"))),
                    Fire(221),
                    R(branch_read(), Ok(Answer::Commit([1; 32]))),
                ]),
            ),
        ),
    ];
    skein_rows::check(rows, refusal_row);
}

type RefusalCase = (Resource, bool, u32, Box<[RefusalStep]>);
fn refusal_row((resource, participating, errors, steps): RefusalCase) {
    use RefusalStep::{Fire, Hint, HintOut, Listed, Pushed, Quiet, Read as R, UnchangedTip, Writer};
    let mut h = Harness::with_limits(Limits {
        inbox: 1,
        rows: 1,
        poll: LIMITS.poll,
        poll_max: Duration::from_secs(1000),
        rate: 100,
        reserve: 0,
        ..LIMITS
    });
    // Pull rests are five seconds; the listing stays out of timed observations.
    h.keep(Box::new([watch(resource.clone(), participating)]));
    for step in steps {
        match step {
            R(read, answer) => {
                pending_listing(&mut h);
                read_answer(&mut h, read, answer);
            }
            Fire(now) => h.fire(now),
            Quiet => {
                pending_listing(&mut h);
                assert!(h.send().is_none(), "refused inputs read again");
            }
            Hint(change) => h.event(Event::Hint { hint: api::Hint { repository: REPO, change, key: None } }),
            Writer(taken) => h.event(Event::Writer { resource: resource.clone(), taken }),
            Pushed => h.event(Event::Pushed { resource: resource.clone(), commit: [1; 32] }),
            HintOut(read, answer) => {
                pending_listing(&mut h);
                let sent = h.one();
                assert_eq!(unpinned(sent.op.clone()), Op::Read(read));
                h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(9), key: None } });
                h.answer(sent, answer);
            }
            Listed(now) => {
                h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(10), key: None } });
                h.fire(now.saturating_add(1));
                let sent = h.one();
                assert_eq!(sent.op, Op::Read(listing_read(100)));
                let mut item = summary();
                item.updated = h.env.now;
                let now = h.env.now;
                h.answer(sent, Answer::Items { cut: false, items: Box::new([item]), more: false, now });
                h.domain.alarms.cancel(crate::domain::Alarm::Repository(REPO));
            }
            UnchangedTip => {
                assert_eq!(h.record(&resource).cached.tip, Some([1; 32]));
                assert_eq!(h.drift, 0);
                assert!(h.news.is_empty());
            }
        }
    }
    assert_eq!(h.failures.len(), errors);
    for error in h.failures.as_slice() {
        let Error::TooLarge(_) = error else {
            panic!("expected a permanent size refusal");
        };
    }
}

#[test]
fn a_failing_read_waits_longer_each_time_up_to_slow_and_an_answer_resets_it() {
    skein_rows::check([("branch read", false), ("issue listing", true)], failing_read_row);
}
fn failing_read_row(listing: bool) {
    let resource = if listing { issue() } else { branch() };
    let mut h = Harness::with_limits(Limits {
        slow: Duration::from_secs(4),
        fallback: Duration::from_secs(120),
        rate: 100,
        ..LIMITS
    });
    h.keep(Box::new([watch(resource.clone(), false)]));
    if !listing {
        pending_listing(&mut h);
    }
    h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(10), key: None } });
    h.domain.alarms.cancel(crate::domain::Alarm::Repository(REPO));
    let read = if listing { listing_read(u64::MAX) } else { branch_read() };
    // Successive waits: 1, 2, 4, 4; success then restores the one-second wait.
    for (failed_at, next) in [(100_u64, 101_u64), (101, 103), (103, 107), (107, 111)] {
        h.fire(failed_at);
        read_answer(&mut h, read.clone(), Err(Error::Unavailable));
        assert_eq!(
            h.failures.len(),
            if listing {
                0
            } else {
                match failed_at {
                    100 => 1,
                    101 => 2,
                    103 => 3,
                    107 => 4,
                    _ => unreachable!(),
                }
            }
        );
        if listing && failed_at == 100 {
            assert_eq!(h.one().op, Op::Read(object_read(u64::MAX)));
        }
        h.fire(next.saturating_sub(1));
        assert!(h.send().is_none(), "listing={listing}: retry before {next}");
        h.fire(next);
    }
    read_answer(&mut h, read, Ok(if listing { empty_listing(111) } else { Answer::Commit([1; 32]) }));
    h.event(Event::Hint {
        hint: api::Hint { repository: REPO, change: Change::Branch(Box::from(*b"topic")), key: None },
    });
    h.fire(112);
    if !listing {
        pending_listing(&mut h);
    }
    read_answer(&mut h, if listing { listing_read(111) } else { branch_read() }, Err(Error::Unavailable));
    h.fire(113);
    assert_eq!(h.one().op, Op::Read(if listing { listing_read(111) } else { branch_read() }));
    assert_eq!(h.failures.len(), if listing { 0 } else { 5 });
}

// Answer a listing only when sent, before and after item-only listings land.
fn pending_listing(h: &mut Harness) {
    if let Some(sent) = h.send() {
        match sent.op {
            Op::Read(Read::Items { .. }) => {
                let now = h.env.now;
                h.answer(sent, Answer::Items { cut: false, items: Box::new([]), more: false, now });
            }
            Op::Read(_) | Op::Write(_) => h.calls.push(sent),
        }
    }
    h.domain.alarms.cancel(crate::domain::Alarm::Repository(REPO));
}

#[test]
fn an_invalid_answer_counts_as_a_failure_in_a_row() {
    skein_rows::check(
        [("double to slow", (1, 4, [101, 103, 107])), ("slow below backoff", (2, 1, [102, 104, 106]))],
        invalid_answer_row,
    );
}
fn invalid_answer_row((backoff, slow, times): (u64, u64, [u64; 3])) {
    let mut h = Harness::with_limits(Limits {
        backoff: Duration::from_secs(backoff),
        slow: Duration::from_secs(slow),
        rate: 100,
        read_attempts: 1,
        ..LIMITS
    });
    h.keep(Box::new([watch(issue(), true)]));
    pending_listing(&mut h);
    let before = h.record(&issue()).clone();
    for next in times {
        read_answer(&mut h, object_read(0), Ok(Answer::Item { item: summary(), comments: Box::new([]), more: true }));
        assert_eq!(h.record(&issue()), &before);
        h.fire(next.saturating_sub(1));
        assert!(h.send().is_none(), "invalid answer reset the wait");
        h.fire(next);
    }
    assert_eq!(h.failures.as_slice(), &[Error::InvalidAnswer; 3]);
    read_answer(&mut h, object_read(0), Ok(empty_object()));
    let after_answer = h.env.now.saturating_add(Duration::from_secs(slow));
    h.env.now = after_answer;
    h.fire(after_answer.as_nanos().checked_div(at(1).as_nanos()).expect("a second is nonzero"));
    read_answer(&mut h, object_read(0), Err(Error::Unavailable));
    let next = after_answer.saturating_add(Duration::from_secs(backoff));
    assert_eq!(h.domain.next_deadline(), Some(next));
}

#[test]
fn missing_resets_the_wait_and_reports_once_at_slow_or_hinted() {
    skein_rows::check([("slow absence", false), ("hinted absence", true)], missing_row);
}
fn missing_row(hinted: bool) {
    let mut h = Harness::with_limits(Limits { slow: Duration::from_secs(4), rate: 100, read_attempts: 1, ..LIMITS });
    h.keep(Box::new([watch(branch(), false)]));
    pending_listing(&mut h);
    for next in [101, 103] {
        read_answer(&mut h, branch_read(), Err(Error::Unavailable));
        h.fire(next);
    }
    if hinted {
        h.event(Event::Hint {
            hint: api::Hint { repository: REPO, change: Change::Branch(Box::from(*b"topic")), key: None },
        });
    }
    read_answer(&mut h, branch_read(), Err(Error::Missing));
    // Two absent answers report once; absence is an answer that resets retries.
    let next: u64 = if hinted { 104 } else { 107 };
    h.fire(next.saturating_sub(1));
    pending_listing(&mut h);
    assert!(h.send().is_none());
    h.fire(next);
    pending_listing(&mut h);
    read_answer(&mut h, branch_read(), Err(Error::Missing));
    assert_eq!(h.failures.as_slice(), &[Error::Unavailable, Error::Unavailable, Error::Missing]);
    h.fire(next.saturating_add(4));
    read_answer(&mut h, branch_read(), Err(Error::Unavailable));
    assert_eq!(h.domain.next_deadline(), Some(at(next.saturating_add(5))));
}
