use super::fixtures::keep::{
    Harness, branch, comment, current, issue, matches_write, participation, pull, review, summary, watch,
};
use super::{LIMITS, REPO, at};
use crate::{Condition, Effect, Entry, Event, Key, Limits, Made, Outcome, Resource, Stored, What};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Change, Error, Op, Read, Write};
use skein_lib::{List, Token};

#[test]
fn live_keys_follow_keep_order_after_restore_and_drop_a_retired_row() {
    use super::fixtures::{Harness, recovery};
    use crate::{RecoveryClock, Request};

    let other_repo = api::Repository { repository: 6, ..REPO };
    let other = Resource { repository: other_repo, what: What::Repository };
    let mut h = Harness::new(LIMITS);
    assert!(h.domain.live_keys().is_empty());
    let template = recovery::live();
    for key in [other.clone(), issue(), pull()] {
        let mut row = template.clone();
        row.watch = watch(key, false);
        assert!(h.step(Event::Restore { record: Stored::Live(row) }).is_empty());
    }
    for repository in [other_repo, REPO] {
        let record = crate::RepositoryRecord { repository, ..recovery::repository() };
        assert!(h.step(Event::Restore { record: Stored::Repository(record) }).is_empty());
    }
    assert!(h.step(Event::Restored { clock: RecoveryClock::Monotonic }).is_empty());
    assert!(!h.domain.restoration_failed());
    let keys = h.domain.live_keys();
    assert_eq!(keys.as_ref(), &[pull(), issue(), other.clone()]);
    for key in &keys {
        assert_eq!(h.domain.cached(key), Some(template.cached.as_ref()));
    }

    let requests = h.step(Event::Keep {
        owner: Token::new(1),
        watches: Box::new([watch(pull(), false), watch(other.clone(), false)]),
    });
    assert!(requests.contains(&Request::Kept { owner: Token::new(1), result: Ok(()) }));
    assert!(requests.contains(&Request::Erase { key: Key::Live(issue()) }));
    assert_eq!(h.domain.live_keys().as_ref(), &[pull(), other]);
    assert!(h.domain.cached(&issue()).is_none());
}

#[test]
fn watch_batches_are_atomic_and_leaving_an_outstanding_resource_consumes_its_terminal() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), true)]));
    h.clock();
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Item { number: 9, after: 0, since: skein_lib::Time::ZERO }));
    let before = h.records.len();
    h.keep(Box::new([watch(issue(), true), watch(issue(), true)]));
    assert_eq!(h.kept.as_slice().last(), Some(&Err(Error::Refused)));
    assert_eq!(h.records.len(), before);
    h.keep(Box::new([]));
    assert!(h.domain.cached(&issue()).is_none());
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(1)]), more: false });
    assert!(h.news.is_empty());
    assert!(h.records.is_empty());
    assert!(!h.domain.is_ready());
}
#[test]
fn a_full_batch_is_refused_before_any_partial_admission() {
    let mut h = Harness::new();
    h.env.limits.resources = 1;
    h.keep(Box::new([watch(issue(), true), watch(branch(), false)]));
    assert_eq!(h.kept.as_slice(), &[Err(Error::TooLarge(api::Excess::exact(api::Bound::Forge, 1, 2)))]);
    assert!(h.records.is_empty());
    assert!(!h.domain.is_ready());
}
#[test]
fn owned_pull_keeps_ci_and_base_on_backoff_without_reading_forge_reviews() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(pull(), false)]));
    h.clock();
    let item = h.one();
    assert_eq!(
        item.op,
        Op::Read(Read::Item { number: 9, after: u64::MAX, since: skein_lib::Time::from_nanos(u64::MAX) })
    );
    h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Pull { number: 9, ci: true }));
    h.answer(read, Answer::Pull(current()));
    assert!(!h.domain.is_ready());
    h.fire(105);
    let listing = h.one();
    assert_eq!(listing.op, Op::Read(Read::Items { single: false, since: at(100), page: 1, kind: None }));
    h.answer(listing, Answer::Items { cut: false, items: Box::new([]), more: false, now: at(105) });
    let item = h.one();
    h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let ci = h.one();
    let mut failed = current();
    failed.ci = api::Ci::Failed;
    h.answer(ci, Answer::Pull(failed));
    assert_eq!(h.domain.cached(&pull()).expect("pull kept").pull.as_ref().expect("pull read").ci, api::Ci::Failed);
}
#[test]
fn a_nonparticipating_pull_rests_its_full_quiet_backoff_without_inbox_repairs() {
    use skein_lib::Duration;
    let mut h = Harness::with_limits(Limits {
        rate: 100,
        reserve: 0,
        poll: Duration::from_secs(5),
        poll_max: Duration::from_secs(30),
        slow: Duration::from_secs(40),
        ..LIMITS
    });
    h.keep(Box::new([watch(pull(), false)]));
    let mut pulls = List::with_capacity(40);
    for second in 100..400 {
        h.fire(second);
        for _ in 0_u32..8 {
            let Some(read) = h.send() else { break };
            let answer = if let Op::Read(Read::Items { .. }) = read.op {
                Answer::Items { cut: false, items: Box::new([]), more: false, now: at(second) }
            } else if let Op::Read(Read::Item { .. }) = read.op {
                Answer::Item { item: summary(), comments: Box::new([]), more: false }
            } else {
                assert_eq!(read.op, Op::Read(Read::Pull { number: 9, ci: true }));
                pulls.push(second).unwrap();
                Answer::Pull(current())
            };
            h.answer(read, answer);
        }
    }
    for since in 220..340 {
        let mut count = 0_u32;
        for read in &pulls {
            if *read >= since && *read < since + 60 {
                count += 1;
            }
        }
        assert!(count <= 3, "quiet minute at {since}: {pulls:?}");
    }
}

#[test]
fn own_pushes_and_held_writer_hints_are_not_drift_but_a_later_external_tip_is() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(branch(), false)]));
    let read = h.one();
    h.answer(read, Answer::Commit([1; 32]));
    h.event(Event::Writer { resource: branch(), taken: true });
    h.event(Event::Hint {
        hint: api::Hint { repository: REPO, change: Change::Branch(Box::from(&b"topic"[..])), key: None },
    });
    h.event(Event::Pushed { resource: branch(), commit: [2; 32] });
    h.event(Event::Writer { resource: branch(), taken: false });
    let read = h.one();
    h.answer(read, Answer::Commit([2; 32]));
    assert_eq!(h.drift, 0);
    h.event(Event::Hint {
        hint: api::Hint { repository: REPO, change: Change::Branch(Box::from(&b"topic"[..])), key: None },
    });
    h.fire(101);
    let read = h.one();
    h.answer(read, Answer::Commit([3; 32]));
    assert_eq!(h.drift, 1);
}
#[test]
fn own_comment_echo_never_skips_an_intervening_unread_human_comment() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), true)]));
    h.clock();
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(5)]), more: false });
    h.news.clear();
    h.event(Event::Make {
        entry: Entry {
            number: 1,
            task: 7,
            repository: REPO,
            effect: Effect {
                write: Write::Post { number: 9, key: Box::from(&b"own-key"[..]), body: Box::from(&b"done"[..]) },
                condition: Condition::None,
            },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    });
    h.clock();
    let write = h.one();
    h.answer(write, Answer::Commented(7));
    assert_eq!(h.outcomes.as_slice(), &[Outcome::Made { made: Made::Commented(7), found: false }]);
    assert_eq!(h.record(&issue()).position.comment, 5);
    h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(9), key: None } });
    h.fire(101);
    let listing = h.one();
    h.answer(listing, Answer::Items { cut: false, items: Box::new([]), more: false, now: at(101) });
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Item { number: 9, after: 0, since: skein_lib::Time::ZERO }));
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(6), comment(7)]), more: false });
    assert_eq!(h.record(&issue()).position.comment, 7);
    assert!(h.record(&issue()).echoes.is_empty());
    match h.news.as_slice() {
        [Answer::Item { comments, .. }] => {
            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].id, 6);
        }
        _ => panic!("one human comment delivered"),
    }
}
#[test]
fn time_paging_keeps_the_tie_replay_point_when_an_item_moves_during_the_pass() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    let _item_in_flight = h.one();
    h.fire(105);
    let first = h.one();
    assert_eq!(first.op, Op::Read(Read::Items { single: false, since: at(100), page: 1, kind: None }));
    h.answer(first, Answer::Items { cut: false, items: Box::new([summary()]), more: true, now: at(105) });
    let tied = h.one();
    assert_eq!(tied.op, Op::Read(Read::Items { single: false, since: at(100), page: 2, kind: None }));
    let mut moved = summary();
    moved.updated = at(105);
    h.answer(tied, Answer::Items { cut: false, items: Box::new([moved.clone()]), more: true, now: at(106) });
    let next = h.one();
    assert_eq!(next.op, Op::Read(Read::Items { single: false, since: at(105), page: 1, kind: None }));
    h.answer(next, Answer::Items { cut: false, items: Box::new([moved]), more: false, now: at(107) });
    match h.records.get(&Key::Repository(REPO)).expect("repository saved") {
        Stored::Repository(record) => {
            assert_eq!(record.mark, Some(at(100)));
            assert_eq!(record.clock, at(107));
        }
        Stored::Live(_) | Stored::Writer(_) => panic!("repository row"),
    }
}
#[test]
fn cut_time_pages_resume_after_every_kept_row_then_restore_full_pages() {
    for full_before_cut in [false, true] {
        let mut h = Harness::with_limits(Limits { rows: 8, rate: 100, reserve: 0, ..LIMITS });
        h.keep(Box::new([watch(issue(), false)]));
        h.clock();
        let _item_in_flight = h.one();
        h.fire(105);
        let mut next_number = 1;
        if full_before_cut {
            let first = h.one();
            let mut items = List::with_capacity(8);
            for number in 1..=8 {
                items.push(api::Summary { number, ..summary() }).unwrap();
            }
            let items = items.into_boxed();
            h.answer(first, Answer::Items { items, more: true, cut: false, now: at(105) });
            next_number = 9;
        }
        let cut = h.one();
        assert_eq!(
            cut.op,
            Op::Read(Read::Items {
                since: at(100),
                page: if full_before_cut { 2 } else { 1 },
                kind: None,
                single: false
            })
        );
        let mut items = List::with_capacity(3);
        for number in next_number..next_number + 3 {
            items.push(api::Summary { number, ..summary() }).unwrap();
        }
        let items = items.into_boxed();
        h.answer(cut, Answer::Items { items, more: true, cut: true, now: at(105) });
        for number in next_number + 3..=next_number + 4 {
            let single = h.one();
            assert_eq!(
                single.op,
                Op::Read(Read::Items {
                    since: at(100),
                    page: u32::try_from(number).unwrap(),
                    kind: None,
                    single: true
                })
            );
            h.answer(
                single,
                Answer::Items {
                    items: Box::new([api::Summary { number, ..summary() }]),
                    more: true,
                    cut: false,
                    now: at(105),
                },
            );
        }
        let later = h.one();
        assert_eq!(
            later.op,
            Op::Read(Read::Items {
                since: at(100),
                page: u32::try_from(next_number + 5).unwrap(),
                kind: None,
                single: true
            })
        );
        let moved = api::Summary { number: next_number + 5, updated: at(104), ..summary() };
        h.answer(later, Answer::Items { items: Box::new([moved.clone()]), more: true, cut: false, now: at(105) });
        let full = h.one();
        assert_eq!(full.op, Op::Read(Read::Items { since: at(104), page: 1, kind: None, single: false }));
        h.answer(full, Answer::Items { items: Box::new([moved]), more: false, cut: false, now: at(105) });
        assert!(h.calls.is_empty(), "a short uncut page ends the pass");
        assert_eq!(h.repository().mark, Some(at(104)));
    }
}

#[test]
fn a_cut_page_ending_later_restarts_full_pages_at_its_last_time() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    let _item_in_flight = h.one();
    h.fire(105);
    let cut = h.one();
    let later = api::Summary { updated: at(104), ..summary() };
    h.answer(cut, Answer::Items { items: Box::new([later.clone()]), more: true, cut: true, now: at(105) });
    let full = h.one();
    assert_eq!(full.op, Op::Read(Read::Items { since: at(104), page: 1, kind: None, single: false }));
    h.answer(full, Answer::Items { items: Box::new([later]), more: false, cut: false, now: at(105) });
    assert!(h.calls.is_empty());
    assert_eq!(h.repository().mark, Some(at(104)));
}

#[test]
fn time_paging_bounds_future_items_by_each_answers_clock() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    let _item_in_flight = h.one();
    h.fire(105);
    let first = h.one();
    let mut future = summary();
    future.updated = at(150);
    h.answer(first, Answer::Items { cut: false, items: Box::new([future.clone()]), more: true, now: at(110) });
    let next = h.one();
    assert_eq!(next.op, Op::Read(Read::Items { single: false, since: at(110), page: 1, kind: None }));
    h.answer(next, Answer::Items { cut: false, items: Box::new([future.clone()]), more: true, now: at(110) });
    let tied = h.one();
    assert_eq!(tied.op, Op::Read(Read::Items { single: false, since: at(110), page: 2, kind: None }));
    h.answer(tied, Answer::Items { cut: false, items: Box::new([future.clone()]), more: true, now: at(100) });
    let next = h.one();
    assert_eq!(next.op, Op::Read(Read::Items { single: false, since: at(100), page: 3, kind: None }));
    h.answer(next, Answer::Items { cut: false, items: Box::new([future]), more: false, now: at(120) });
    assert_eq!(h.repository().mark, Some(at(100)));
}

#[test]
fn a_moved_listings_mark_stays_within_its_answers_clock() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    let _item_in_flight = h.one();
    h.fire(105);
    let listing = h.one();
    let mut future = summary();
    future.updated = at(150);
    h.answer(listing, Answer::Items { cut: false, items: Box::new([future]), more: false, now: at(100) });
    assert_eq!(h.repository().mark, Some(at(100)));
    assert!(h.repository().mark <= Some(h.repository().clock));
}

#[test]
fn a_quiet_listings_mark_stays_within_its_final_answers_clock() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    let _item_in_flight = h.one();
    h.fire(105);
    let first = h.one();
    let mut future = summary();
    future.updated = at(150);
    h.answer(first, Answer::Items { cut: false, items: Box::new([future.clone()]), more: true, now: at(200) });
    let last = h.one();
    h.answer(last, Answer::Items { cut: false, items: Box::new([future]), more: false, now: at(100) });
    assert_eq!(h.repository().mark, Some(at(100)));
}

#[test]
fn a_tied_listings_mark_stays_within_its_final_answers_clock() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    let _item_in_flight = h.one();
    h.fire(105);
    let mut future = summary();
    future.updated = at(150);
    for _ in 0_u32..2 {
        let page = h.one();
        h.answer(page, Answer::Items { cut: false, items: Box::new([future.clone()]), more: true, now: at(150) });
    }
    let last = h.one();
    assert_eq!(last.op, Op::Read(Read::Items { single: false, since: at(150), page: 2, kind: None }));
    h.answer(last, Answer::Items { cut: false, items: Box::new([future]), more: false, now: at(100) });
    assert_eq!(h.repository().mark, Some(at(100)));
}

#[test]
fn a_restored_mark_is_bounded_by_its_rows_clock_before_listing() {
    use super::fixtures::{Harness, recovery};
    use crate::RecoveryClock;

    for fresh in [false, true] {
        let mut h = Harness::new(LIMITS);
        let mut live = recovery::live();
        live.watch = watch(issue(), false);
        h.step(Event::Restore { record: Stored::Live(live) });
        h.step(Event::Restore {
            record: Stored::Repository(crate::RepositoryRecord { mark: Some(at(200)), ..recovery::repository() }),
        });
        h.step(Event::Restored { clock: RecoveryClock::Monotonic });
        if fresh {
            h.step(Event::ReadAfresh);
        }
        let listing = h.one();
        assert_eq!(listing.op, Op::Read(Read::Items { single: false, since: at(100), page: 1, kind: None }));
    }
}

#[test]
fn the_first_listing_probes_past_time_but_marks_only_its_answers_clock() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    assert_eq!(h.repository().mark, Some(at(100)));
    assert_eq!(h.repository().clock, at(100));
}

#[test]
fn a_pending_review_submitted_later_is_delivered_despite_its_older_id() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(pull(), true)]));
    h.clock();
    participation(&mut h, Box::new([review(2, 100)]));
    h.news.clear();
    h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(9), key: None } });
    h.fire(101);
    let listing = h.one();
    h.answer(listing, Answer::Items { cut: false, items: Box::new([]), more: false, now: at(101) });
    participation(&mut h, Box::new([review(1, 101), review(2, 100)]));
    match h.news.as_slice() {
        [Answer::Reviews { reviews, .. }] => {
            assert_eq!(reviews.len(), 1);
            assert_eq!(reviews[0].id, 1);
        }
        _ => panic!("one newly submitted review"),
    }
    assert_eq!(h.record(&pull()).reviews.len(), 2);
}
#[test]
fn a_full_echo_ledger_blocks_new_writes_until_the_inbox_reads_past_it() {
    let mut h = Harness::new();
    h.env.limits.rows = 1;
    h.keep(Box::new([watch(issue(), true)]));
    h.clock();
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    for entry in 1..=2_u64 {
        h.event(Event::Make {
            entry: Entry {
                number: entry,
                task: 7,
                repository: REPO,
                effect: Effect {
                    write: Write::Post {
                        number: 9,
                        key: Box::new([u8::try_from(entry).unwrap()]),
                        body: Box::from(&b"done"[..]),
                    },
                    condition: Condition::None,
                },
                start: None,
                attempt: None,
                failures: 0,
                queued: false,
            },
        });
        h.clock();
        if entry == 1 {
            let write = h.one();
            h.answer(write, Answer::Commented(7));
        }
    }
    assert!(!h.domain.is_ready());
    assert_eq!(h.record(&issue()).echoes.len(), 1);
    h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(9), key: None } });
    h.fire(101);
    let listing = h.one();
    h.answer(listing, Answer::Items { cut: false, items: Box::new([]), more: false, now: at(101) });
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(6)]), more: true });
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(7)]), more: false });
    let write = h.one();
    assert!(matches_write(&write.op));
    let entry = h.entries.get(&2).expect("top received entry progress");
    assert_eq!(entry.start.expect("write position saved").comment, 7);
}
#[test]
fn a_rate_refused_terminal_after_departure_never_requeues_the_removed_resource() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), true)]));
    h.clock();
    let read = h.one();
    h.keep(Box::new([]));
    h.event(Event::Answered {
        call: read.call,
        cost: 1,
        free: 0,
        rate: None,
        result: Err(Error::RateLimited { after: skein_lib::Duration::from_secs(1) }),
    });
    h.fire(101);
    assert!(!h.domain.is_ready());
    assert!(h.records.is_empty());
}
#[test]
fn first_adoption_suppresses_only_framed_historical_keys_from_our_writer() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), true)]));
    h.clock();
    let mut own = comment(1);
    own.author = 1;
    own.key = Some(Box::from(&b"\x01\x00\x03kitold"[..]));
    let mut foreign = comment(2);
    foreign.author = 1;
    foreign.key = Some(Box::from(&b"\x01\x00\x04elseold"[..]));
    let mut copied = comment(3);
    copied.author = 2;
    copied.key = own.key.clone();
    let human = comment(4);
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([own, foreign, copied, human]), more: true });
    let mut malformed = comment(5);
    malformed.author = 1;
    malformed.key = Some(Box::from(&b"\x01\x00\x05testold"[..]));
    let mut own = comment(6);
    own.author = 1;
    own.key = Some(Box::from(&b"\x01\x00\x03kitnext"[..]));
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([malformed, own]), more: false });
    match h.news.as_slice() {
        [Answer::Item { comments: first, .. }, Answer::Item { comments: second, .. }] => {
            assert_eq!(first.len(), 3);
            assert_eq!(first[0].id, 2);
            assert_eq!(first[1].id, 3);
            assert_eq!(first[2].id, 4);
            assert_eq!(second.len(), 1);
            assert_eq!(second[0].id, 5);
        }
        _ => panic!("foreign, copied, human and malformed keys remain news"),
    }
    assert_eq!(h.record(&issue()).position.comment, 6);
}
#[test]
fn first_adoption_of_reviews_requires_namespace_and_authenticated_author() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(pull(), true)]));
    h.clock();
    let mut own = review(1, 90);
    own.author = 1;
    own.key = Some(Box::from(&b"\x01\x00\x03kitold"[..]));
    let mut foreign = review(2, 90);
    foreign.author = 1;
    foreign.key = Some(Box::from(&b"\x01\x00\x04elseold"[..]));
    let mut copied = review(3, 90);
    copied.key = own.key.clone();
    participation(&mut h, Box::new([own, foreign, copied]));
    match h.news.as_slice() {
        [Answer::Item { .. }, Answer::Pull(_), Answer::Reviews { reviews, .. }] => {
            assert_eq!(reviews.len(), 2);
            assert_eq!(reviews[0].id, 2);
            assert_eq!(reviews[1].id, 3);
        }
        _ => panic!("foreign and copied reviews remain news"),
    }
}

#[test]
fn malformed_review_pages_are_rejected_before_any_delivery_or_cursor_mutation() {
    for ids in [[1, 1], [2, 1], [0, 1]] {
        let mut h = Harness::new();
        h.keep(Box::new([watch(pull(), true)]));
        h.clock();
        let read = h.one();
        h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
        let read = h.one();
        h.answer(read, Answer::Pull(current()));
        h.news.clear();
        let before = h.record(&pull()).clone();
        let read = h.one();
        h.answer(
            read,
            Answer::Reviews { reviews: Box::new([review(ids[0], 90), review(ids[1], 90)]), more: false, cut: false },
        );
        assert!(h.news.is_empty());
        assert_eq!(h.record(&pull()), &before);
        assert_eq!(h.failures.as_slice(), &[Error::InvalidAnswer]);
    }
}
#[test]
fn a_repeated_listing_item_cannot_overflow_a_tiny_decisions_output_bound() {
    let mut h =
        Harness::with_limits(Limits { resources: 1, repositories: 1, rows: 10, rate: 100, reserve: 0, ..LIMITS });
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let before = h.record(&issue()).clone();
    h.fire(105);
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Items { single: false, since: at(100), page: 1, kind: None }));
    let mut repeated = List::with_capacity(10);
    for _ in 0..10_u32 {
        let mut item = summary();
        item.updated = at(105);
        repeated.push(item).expect("bounded malformed page");
    }
    h.answer(read, Answer::Items { cut: false, items: repeated.into_boxed(), more: false, now: at(105) });
    assert_eq!(h.record(&issue()), &before);
    assert_eq!(h.domain.next_deadline(), Some(at(106)));
}
#[test]
fn a_reserved_background_share_is_admitted_even_with_one_pending_slot_and_replenished_outbox() {
    let mut h = Harness::with_limits(Limits { pending: 1, calls: 1, rate: 8, reserve: 4, ..LIMITS });
    h.keep(Box::new([watch(issue(), false)]));
    for round in 0..2_u64 {
        for index in 1..=2_u64 {
            h.event(Event::Make {
                entry: Entry {
                    number: round * 3 + index,
                    task: 7,
                    repository: REPO,
                    effect: Effect { write: Write::Close { number: 9 }, condition: Condition::None },
                    start: None,
                    attempt: None,
                    failures: 0,
                    queued: false,
                },
            });
            h.clock();
            let write = h.one();
            assert_eq!(write.op, Op::Write(Write::Close { number: 9 }));
            h.answer(write, Answer::Done);
        }
        h.event(Event::Make {
            entry: Entry {
                number: round * 3 + 3,
                task: 7,
                repository: REPO,
                effect: Effect { write: Write::Close { number: 9 }, condition: Condition::None },
                start: None,
                attempt: None,
                failures: 0,
                queued: false,
            },
        });
        h.event(Event::Read {
            owner: Token::new(round + 10),
            repository: REPO,
            read: Read::Branch { branch: Box::from(&b"fresh"[..]) },
        });
        assert_eq!(h.read_refusals.as_slice().last(), Some(&Error::Busy));
        let read = h.one();
        let since = if round == 0 { at(u64::MAX) } else { at(100) };
        assert_eq!(read.op, Op::Read(Read::Items { single: false, since, page: 1, kind: None }));
        h.answer(read, Answer::Items { cut: false, items: Box::new([]), more: false, now: h.env.now });
        let read = h.one();
        assert_eq!(
            read.op,
            Op::Read(Read::Item { number: 9, after: u64::MAX, since: skein_lib::Time::from_nanos(u64::MAX) })
        );
        h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
        h.clock();
        let write = h.one();
        assert_eq!(write.op, Op::Write(Write::Close { number: 9 }));
        h.answer(write, Answer::Done);
        assert_eq!(h.outcomes.len(), u32::try_from((round + 1) * 3).expect("small test count"));
        if round == 0 {
            h.fire(160);
        }
    }
}
#[test]
fn a_restart_between_review_pages_never_repeats_delivered_rows_or_metadata() {
    for cut in [false, true] {
        let mut h = Harness::new();
        h.keep(Box::new([watch(pull(), true)]));
        h.clock();
        let read = h.one();
        h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
        let read = h.one();
        h.answer(read, Answer::Pull(current()));
        let read = h.one();
        h.answer(read, Answer::Reviews { reviews: Box::new([review(1, 90), review(2, 90)]), more: true, cut });
        assert_eq!(h.record(&pull()).review_page, if cut { 1 } else { 2 });
        if cut {
            let read = h.one();
            assert_eq!(read.op, Op::Read(Read::Reviews { number: 9, page: 1, skip: 2 }));
            h.answer(read, Answer::Reviews { reviews: Box::new([review(3, 91)]), more: true, cut: true });
            let read = h.one();
            assert_eq!(read.op, Op::Read(Read::Reviews { number: 9, page: 1, skip: 3 }));
        }
        h.news.clear();
        h.restart();
        let read = h.one();
        h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
        let read = h.one();
        h.answer(read, Answer::Pull(current()));
        let read = h.one();
        assert_eq!(read.op, Op::Read(Read::Reviews { number: 9, page: if cut { 1 } else { 2 }, skip: 0 }));
        // A pending review became visible before our page: the shifted page
        // repeats an older row, whose durable delivery stamp suppresses it.
        let last = if cut { 4 } else { 3 };
        h.answer(
            read,
            Answer::Reviews { reviews: Box::new([review(2, 90), review(last, 91)]), more: false, cut: false },
        );
        match h.news.as_slice() {
            [Answer::Reviews { reviews, .. }] => {
                assert_eq!(reviews.len(), 1);
                assert_eq!(reviews[0].id, last);
            }
            _ => panic!("one new review after recovery, no cached metadata repeat"),
        }
        assert_eq!(h.record(&pull()).review_page, 1);
    }
}
#[test]
fn comment_scan_progress_and_edited_old_versions_are_durable() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(issue(), true)]));
    h.clock();
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(1)]), more: true });
    assert_eq!(h.record(&issue()).comment_after, 1);
    h.news.clear();
    h.restart();
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Item { number: 9, after: 1, since: skein_lib::Time::ZERO }));
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(2)]), more: false });
    assert_eq!(h.record(&issue()).comment_after, 2);
    h.news.clear();
    h.restart();
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Item { number: 9, after: 0, since: skein_lib::Time::ZERO }));
    let mut edited = comment(1);
    edited.revision = 2;
    edited.body = Box::from(&b"edited"[..]);
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([edited.clone(), comment(2)]), more: false });
    match h.news.as_slice() {
        [Answer::Item { comments, .. }] => assert_eq!(comments.as_ref(), &[edited.clone()]),
        _ => panic!("exactly one edited version"),
    }
    h.news.clear();
    h.restart();
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([edited, comment(2)]), more: false });
    assert!(h.news.is_empty());
}
#[test]
fn inbox_overflow_never_partially_delivers_or_advances_a_page() {
    let mut h = Harness::new();
    h.env.limits.inbox = 1;
    h.keep(Box::new([watch(issue(), true)]));
    h.clock();
    let read = h.one();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(1)]), more: false });
    h.news.clear();
    h.restart();
    let read = h.one();
    let saved = h.record(&issue()).clone();
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(1), comment(2)]), more: false });
    assert_eq!(h.failures.as_slice(), &[Error::TooLarge(api::Excess::unnamed())]);
    assert!(h.news.is_empty());
    assert_eq!(h.record(&issue()), &saved);
    h.env.limits.inbox = 2;
    h.restart();
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Item { number: 9, after: 0, since: skein_lib::Time::ZERO }));
    h.answer(read, Answer::Item { item: summary(), comments: Box::new([comment(1), comment(2)]), more: false });
    match h.news.as_slice() {
        [Answer::Item { comments, .. }] => assert_eq!(comments.as_ref(), &[comment(2)]),
        _ => panic!("once after capacity reconciliation"),
    }
}
#[test]
fn a_first_inbox_read_never_hides_an_edited_or_unknown_completed_own_echo() {
    for provenance in [api::Provenance::Revised, api::Provenance::Unknown] {
        let mut h = Harness::new();
        h.keep(Box::new([watch(issue(), true)]));
        h.clock();
        let read = h.one();
        h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
        h.news.clear();
        h.event(Event::Make {
            entry: Entry {
                number: 1,
                task: 7,
                repository: REPO,
                effect: Effect {
                    write: Write::Post {
                        number: 9,
                        key: Box::from(&b"\x01\x00\x03kitown"[..]),
                        body: Box::from(&b"done"[..]),
                    },
                    condition: Condition::None,
                },
                start: None,
                attempt: None,
                failures: 0,
                queued: false,
            },
        });
        h.clock();
        let write = h.one();
        h.answer(write, Answer::Commented(7));
        h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(9), key: None } });
        h.fire(101);
        let listing = h.one();
        h.answer(listing, Answer::Items { cut: false, items: Box::new([]), more: false, now: at(101) });
        let read = h.one();
        let mut changed = comment(7);
        changed.author = 1;
        changed.provenance = provenance;
        changed.key = Some(Box::from(&b"\x01\x00\x03kitown"[..]));
        h.answer(read, Answer::Item { item: summary(), comments: Box::new([changed.clone()]), more: false });
        match h.news.as_slice() {
            [Answer::Item { comments, .. }] => assert_eq!(comments.as_ref(), &[changed]),
            _ => panic!("changed or unproven body remains news before first inbox read"),
        }
    }
}

#[test]
fn writer_moves_require_forward_ancestry_and_authenticated_identity() {
    let contained = Ok(Answer::Commits { commits: Box::new([]), more: false });
    let listed = api::CommitMessage { id: [1; 32], parents: Box::new([]), message: Box::new([]) };
    for (actor, result, hint_head, expected_drift, failure) in [
        (1, contained.clone(), [2; 32], 0, None),
        (2, contained.clone(), [2; 32], 1, None),
        (2, contained, [3; 32], 0, None),
        (1, Ok(Answer::Commits { commits: Box::new([listed.clone()]), more: false }), [2; 32], 1, None),
        (1, Ok(Answer::Commits { commits: Box::new([listed]), more: true }), [2; 32], 1, None),
        (1, Err(Error::TooLarge(api::Excess::unnamed())), [2; 32], 0, Some(Error::TooLarge(api::Excess::unnamed()))),
        (1, Ok(Answer::Commits { commits: Box::new([]), more: true }), [2; 32], 0, Some(Error::InvalidAnswer)),
        (1, Err(Error::Unavailable), [2; 32], 0, Some(Error::Unavailable)),
        (1, Err(Error::Forbidden), [2; 32], 0, Some(Error::Forbidden)),
        (1, Err(Error::Missing), [2; 32], 0, Some(Error::Missing)),
    ] {
        let mut h = Harness::new();
        h.env.limits.read_attempts = 1;
        h.keep(Box::new([watch(branch(), false)]));
        let read = h.one();
        h.answer(read, Answer::Commit([1; 32]));
        h.event(Event::Writer { resource: branch(), taken: true });
        h.event(Event::Hint {
            hint: api::Hint {
                repository: REPO,
                change: Change::BranchMoved { branch: Box::from(&b"topic"[..]), head: hint_head, actor },
                key: None,
            },
        });
        h.fire(101);
        let read = h.one();
        assert_eq!(read.op, Op::Read(Read::Branch { branch: Box::from(&b"topic"[..]) }));
        h.answer(read, Answer::Commit([2; 32]));
        let compare = h.one();
        assert_eq!(compare.op, Op::Read(Read::Commits { before: [2; 32], after: [1; 32], page: 1 }));
        h.event(Event::Answered { call: compare.call, cost: 1, free: 0, rate: None, result });
        assert_eq!(h.failures.as_slice(), failure.as_slice());
        assert_eq!(h.record(&branch()).cached.tip, Some(if failure.is_some() { [1; 32] } else { [2; 32] }));
        assert_eq!(h.drift, expected_drift);
        assert_eq!(h.news.as_slice(), &[Answer::Commit([1; 32])]);
    }
}

fn missing(h: &mut Harness, sent: super::Sent) {
    h.event(Event::Answered { call: sent.call, cost: 1, free: 0, rate: None, result: Err(Error::Missing) });
}

#[test]
fn a_branch_read_from_before_our_push_is_read_again() {
    for writer in [false, true] {
        let mut h = Harness::new();
        h.keep(Box::new([watch(branch(), false)]));
        let read = h.one();
        if writer {
            h.event(Event::Writer { resource: branch(), taken: true });
        }
        h.event(Event::Pushed { resource: branch(), commit: [1; 32] });
        missing(&mut h, read);
        assert_eq!(h.drift, 0, "an absence read before our push cannot judge it");
        assert!(h.failures.is_empty());
        assert_eq!(h.record(&branch()).cached.tip, Some([1; 32]));
        let read = h.one();
        assert_eq!(read.op, Op::Read(Read::Branch { branch: Box::from(&b"topic"[..]) }));
        h.answer(read, Answer::Commit([1; 32]));
        assert_eq!(h.drift, 0);
        assert_eq!(h.record(&branch()).cached.tip, Some([1; 32]));
    }
}
#[test]
fn a_branch_still_missing_on_its_read_after_our_push_drifts() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(branch(), false)]));
    let read = h.one();
    h.event(Event::Pushed { resource: branch(), commit: [1; 32] });
    missing(&mut h, read);
    assert_eq!(h.drift, 0);
    let read = h.one();
    missing(&mut h, read);
    assert_eq!(h.drift, 1, "a branch deleted after our push still drifts");
    assert_eq!(h.failures.as_slice(), &[Error::Missing]);
    assert_eq!(h.record(&branch()).cached.tip, None);
}
#[test]
fn missing_branch_rests_at_slow_and_reports_each_absence_once() {
    for (seen, fallback) in [(false, 0), (false, 30), (false, 120), (true, 120)] {
        let mut h = Harness::new();
        h.env.limits.fallback = skein_lib::Duration::from_secs(fallback);
        h.env.limits.poll = skein_lib::Duration::from_secs(1000);
        h.env.limits.poll_max = h.env.limits.poll;
        h.keep(Box::new([watch(branch(), false)]));
        let start = if seen {
            let read = h.one();
            h.answer(read, Answer::Commit([1; 32]));
            h.fire(160);
            160
        } else {
            100
        };
        let read = h.one();
        missing(&mut h, read);
        assert_eq!(h.failures.as_slice(), &[Error::Missing]);
        assert_eq!(h.drift, u32::from(seen));
        assert_eq!(h.record(&branch()).cached.tip, None);
        let interval = if seen { 60 } else { 60.max(fallback) };
        for now in [start + interval, start + interval * 2, start + interval * 3] {
            h.fire(now - 1);
            assert!(h.send().is_none(), "absence waits the whole slow/fallback rest");
            h.fire(now);
            let read = h.one();
            assert_eq!(read.op, Op::Read(Read::Branch { branch: Box::from(&b"topic"[..]) }));
            missing(&mut h, read);
            assert_eq!(h.failures.as_slice(), &[Error::Missing]);
            assert_eq!(h.drift, u32::from(seen));
        }
        let present = start + interval * 4;
        h.fire(present);
        let read = h.one();
        h.answer(read, Answer::Commit([2; 32]));
        assert_eq!(h.news.as_slice().last(), Some(&Answer::Commit([2; 32])));
        let drift = h.drift;
        h.fire(present + 60);
        let read = h.one();
        missing(&mut h, read);
        assert_eq!(h.failures.as_slice(), &[Error::Missing, Error::Missing]);
        assert_eq!(h.drift, drift + 1);
        h.restart();
        let read = h.one();
        missing(&mut h, read);
        assert_eq!(h.failures.as_slice(), &[Error::Missing, Error::Missing, Error::Missing]);
        assert_eq!(h.drift, drift + 1, "cleared stored tip prevents deletion drift after restart");
        assert_eq!(h.record(&branch()).cached.tip, None);
    }
}

#[test]
fn missing_issue_and_pull_rest_at_slow_and_suppress_repeated_news() {
    for resource in [issue(), pull()] {
        let mut h = Harness::new();
        h.env.limits.poll = skein_lib::Duration::from_secs(1000);
        h.env.limits.poll_max = h.env.limits.poll;
        h.keep(Box::new([watch(resource.clone(), false)]));
        h.clock();
        let read = h.one();
        missing(&mut h, read);
        for now in [160, 220] {
            h.fire(now - 1);
            assert!(h.send().is_none());
            h.fire(now);
            let read = h.one();
            missing(&mut h, read);
            assert_eq!(h.failures.as_slice(), &[Error::Missing]);
        }
        // Finding the watched resource ends the absence.
        h.fire(280);
        let read = h.one();
        h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
        if resource == pull() {
            let read = h.one();
            h.answer(read, Answer::Pull(current()));
        }
        h.fire(340);
        let read = h.one();
        missing(&mut h, read);
        assert_eq!(h.failures.as_slice(), &[Error::Missing, Error::Missing]);
    }
}

#[test]
fn missing_uses_hinted_rest_and_unavailable_keeps_backoff() {
    for pushed in [false, true] {
        let mut h = Harness::with_limits(Limits { fallback: FALLBACK, ..LIMITS });
        h.keep(Box::new([watch(branch(), false)]));
        let read = h.one();
        missing(&mut h, read);
        let event = if pushed {
            Event::Pushed { resource: branch(), commit: [1; 32] }
        } else {
            Event::Hint {
                hint: api::Hint { repository: REPO, change: Change::Branch(Box::from(&b"topic"[..])), key: None },
            }
        };
        h.event(event);
        assert_eq!(h.domain.next_deadline(), Some(at(101)), "unseen branch, pushed={pushed}");
        h.fire(101);
        assert_eq!(h.one().op, Op::Read(branch_read()));
    }
    let mut h = Harness::new();
    h.keep(Box::new([watch(branch(), false)]));
    let read = h.one();
    h.event(Event::Hint {
        hint: api::Hint { repository: REPO, change: Change::Branch(Box::from(&b"topic"[..])), key: None },
    });
    missing(&mut h, read);
    h.fire(101);
    let read = h.one();
    missing(&mut h, read);
    assert_eq!(h.failures.as_slice(), &[Error::Missing]);

    let mut h = Harness::new();
    h.keep(Box::new([watch(branch(), false)]));
    for now in [101, 103, 107] {
        let read = h.one();
        h.event(Event::Answered { call: read.call, cost: 1, free: 0, rate: None, result: Err(Error::Unavailable) });
        assert!(h.send().is_none());
        h.fire(now);
    }
    assert_eq!(h.failures.as_slice(), &[Error::Unavailable; 3]);
    assert_eq!(h.one().op, Op::Read(Read::Branch { branch: Box::from(&b"topic"[..]) }));
}

#[test]
fn missing_pull_stays_absent_across_successful_item_phases() {
    let mut h = Harness::new();
    h.env.limits.poll = skein_lib::Duration::from_secs(1000);
    h.env.limits.poll_max = h.env.limits.poll;
    h.keep(Box::new([watch(pull(), false)]));
    h.clock();
    for now in [100, 160, 220] {
        h.fire(now);
        let item = h.one();
        h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
        let read = h.one();
        assert_eq!(read.op, Op::Read(Read::Pull { number: 9, ci: true }));
        missing(&mut h, read);
        assert_eq!(h.failures.as_slice(), &[Error::Missing]);
        h.fire(now + 59);
        assert!(h.send().is_none());
    }
    h.fire(280);
    let item = h.one();
    h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let read = h.one();
    h.answer(read, Answer::Pull(current()));
    h.fire(340);
    let item = h.one();
    h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let read = h.one();
    missing(&mut h, read);
    assert_eq!(h.failures.as_slice(), &[Error::Missing, Error::Missing]);
}

#[test]
fn repository_listing_waits_for_hints_within_the_poll_interval() {
    // temper's timer fixture: a 20 s story, polling one ms beyond it,
    // 10 ms hints and 100 ms slow reconciliation (case.rs, fixture_check).
    const STORY: skein_lib::Duration = skein_lib::Duration::from_secs(20);
    const POLL: skein_lib::Duration = skein_lib::Duration::from_millis(20_001);
    const HINTED: skein_lib::Duration = skein_lib::Duration::from_millis(10);
    const SLOW: skein_lib::Duration = skein_lib::Duration::from_millis(100);
    let mut h = Harness::with_limits(Limits { poll: POLL, poll_max: POLL, hinted: HINTED, slow: SLOW, ..LIMITS });
    h.keep(Box::new([watch(issue(), false)]));
    h.clock();
    let _item_in_flight = h.one();
    let end = h.env.now.saturating_add(STORY);
    // Cross several of the ordinary five-second poll deadlines without
    // shortening the timer fixture's configured poll interval.
    for now in [105, 110, 115, 120] {
        h.fire(now);
        assert!(h.send().is_none(), "elapsed time adds no listing");
    }
    h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(1), key: None } });
    assert!(h.send().is_none(), "the hint waits its coalescing delay");
    h.env.now = end.saturating_add(HINTED);
    while h.domain.is_due(h.env.now) {
        let mut out = skein_lib::Queue::uncapped();
        crate::fire(&mut h.domain, &h.env, &mut out);
        h.take(&mut out);
    }
    let listing = h.one();
    assert_eq!(listing.op, Op::Read(Read::Items { single: false, since: at(100), page: 1, kind: None }));
    h.answer(listing, Answer::Items { cut: false, items: Box::new([]), more: false, now: h.env.now });
    assert!(h.send().is_none(), "one hint makes one listing");
}

const FALLBACK: skein_lib::Duration = skein_lib::Duration::from_secs(120);

#[test]
fn timed_reads_fall_back_while_their_repository_hears() {
    use skein_lib::Duration;
    // A large poll keeps the listing out of the slow-rest observations.
    const LONG_POLL: Duration = Duration::from_secs(1000);
    for (name, resource, participating, poll, result, own, listing) in [
        ("listing repeat", issue(), false, LIMITS.poll, None, 110, true),
        ("pull rest", pull(), false, LIMITS.poll, None, 110, false),
        ("branch slow rest", branch(), false, LONG_POLL, None, 165, false),
        ("unseen missing rest", branch(), false, LONG_POLL, Some(Error::Missing), 225, false),
        ("participating repair", pull(), true, Duration::from_secs(100), None, 165, false),
        ("failure backoff", branch(), false, LONG_POLL, Some(Error::Unavailable), 106, false),
    ] {
        for hears in [false, true] {
            let mut h = Harness::with_limits(Limits { fallback: FALLBACK, poll, poll_max: poll, rate: 100, ..LIMITS });
            h.keep(Box::new([watch(resource.clone(), participating)]));
            if resource != branch() {
                h.clock();
            }
            if hears {
                h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(10), key: None } });
            }
            h.fire(105);
            if resource != branch() && (hears || poll == LIMITS.poll) {
                let read = h.one();
                assert_eq!(read.op, Op::Read(Read::Items { single: false, since: at(100), page: 1, kind: None }));
                h.answer(read, Answer::Items { cut: false, items: Box::new([]), more: false, now: at(105) });
            }
            if !listing {
                let read = h.one();
                if let Some(error) = result {
                    h.event(Event::Answered { call: read.call, cost: 1, free: 0, rate: None, result: Err(error) });
                } else if resource == branch() {
                    h.answer(read, Answer::Commit([1; 32]));
                } else {
                    h.answer(read, Answer::Item { item: summary(), comments: Box::new([]), more: false });
                    let read = h.one();
                    if participating {
                        h.env.now = at(115);
                    }
                    h.answer(read, Answer::Pull(current()));
                    if participating {
                        let read = h.one();
                        h.answer(read, Answer::Reviews { reviews: Box::new([]), more: false, cut: false });
                    }
                }
                // Observe just this resource's armed deadline, not the listing.
                h.domain.alarms.cancel(crate::domain::Alarm::Repository(REPO));
            }
            let expected = if hears && result != Some(Error::Unavailable) { own.max(220) } else { own };
            assert_eq!(h.domain.next_deadline(), Some(at(expected)), "{name}, hears={hears}");
        }
    }
}

#[test]
fn a_repository_that_stops_hearing_polls_at_its_own_pace_again() {
    let mut h = Harness::with_limits(Limits { fallback: FALLBACK, rate: 100, ..LIMITS });
    h.keep(Box::new([watch(branch(), false)]));
    h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(10), key: None } });
    h.fire(101);
    let read = h.one();
    h.answer(read, Answer::Commit([1; 32]));
    assert_eq!(h.domain.next_deadline(), Some(at(220)));
    h.fire(219);
    assert!(h.send().is_none());
    h.fire(220);
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Branch { branch: Box::from(&b"topic"[..]) }));
    h.answer(read, Answer::Commit([1; 32]));
    assert_eq!(h.domain.next_deadline(), Some(at(280)));
}

#[test]
fn a_hint_still_reads_at_hinted_while_its_repository_hears() {
    let mut h = Harness::with_limits(Limits { fallback: FALLBACK, rate: 100, ..LIMITS });
    h.keep(Box::new([watch(branch(), false)]));
    let read = h.one();
    h.answer(read, Answer::Commit([1; 32]));
    h.event(Event::Hint {
        hint: api::Hint { repository: REPO, change: Change::Branch(Box::from(&b"topic"[..])), key: None },
    });
    assert_eq!(h.domain.next_deadline(), Some(at(101)));
    h.fire(101);
    let read = h.one();
    assert_eq!(read.op, Op::Read(Read::Branch { branch: Box::from(&b"topic"[..]) }));
    // A hint received in flight still asks for a hinted rest, not fallback.
    h.event(Event::Hint {
        hint: api::Hint { repository: REPO, change: Change::Branch(Box::from(&b"topic"[..])), key: None },
    });
    h.answer(read, Answer::Commit([1; 32]));
    assert_eq!(h.domain.next_deadline(), Some(at(102)));
    h.fire(102);
    let read = h.one();
    h.answer(read, Answer::Commit([1; 32]));
    assert_eq!(h.domain.next_deadline(), Some(at(221)));
}

fn empty_listing(now: u64) -> Answer {
    Answer::Items { cut: false, items: Box::new([]), more: false, now: at(now) }
}
fn empty_object() -> Answer {
    Answer::Item { item: summary(), comments: Box::new([]), more: false }
}
fn listing_read(since: u64) -> Read {
    Read::Items { single: false, since: at(since), page: 1, kind: None }
}
fn branch_read() -> Read {
    Read::Branch { branch: Box::from(*b"topic") }
}
fn drain_kept_reads(h: &mut Harness) -> Box<[Read]> {
    let mut reads = List::with_capacity(20); // each tiny watch set has at most four phases
    for _ in 0_u32..20 {
        let Some(sent) = h.send() else { break };
        let Op::Read(read) = &sent.op else { panic!("keeping reads only") };
        reads.push(read.clone()).expect("bounded fixture calls");
        let answer = match read {
            Read::Items { .. } => empty_listing(100),
            Read::Item { .. } => empty_object(),
            Read::Pull { .. } => Answer::Pull(current()),
            Read::Branch { .. } => Answer::Commit([1; 32]),
            Read::Locate { .. }
            | Read::PullFor { .. }
            | Read::Reviews { .. }
            | Read::Statuses { .. }
            | Read::Remarks { .. }
            | Read::Branches { .. }
            | Read::PullFiles { .. }
            | Read::Compare { .. }
            | Read::Checks { .. }
            | Read::Job { .. }
            | Read::File { .. }
            | Read::Protection { .. }
            | Read::Settings
            | Read::Collaborators { .. }
            | Read::Permission { .. }
            | Read::Runs { .. }
            | Read::Run { .. }
            | Read::Tree { .. }
            | Read::Artifact { .. }
            | Read::Commits { .. } => panic!("unexpected keeping phase"),
        };
        h.answer(sent, answer);
    }
    reads.into_boxed()
}
#[test]
fn a_listing_runs_only_while_an_issue_or_pull_watch_lives() {
    let repository = Resource { repository: REPO, what: What::Repository };
    for (name, resource, first, listed) in [
        ("branch only", branch(), Some(branch_read()), false),
        ("repository only", repository, None, false),
        ("issue", issue(), Some(listing_read(u64::MAX)), true),
        ("pull", pull(), Some(listing_read(u64::MAX)), true),
    ] {
        let mut h = Harness::new();
        h.keep(Box::new([watch(resource, false)]));
        let reads = drain_kept_reads(&mut h);
        assert_eq!(reads.first(), first.as_ref(), "{name}: first call");
        assert_eq!(reads.contains(&listing_read(u64::MAX)), listed, "{name}: bootstrap");
        for now in [105, 110] {
            h.fire(now);
            let reads = drain_kept_reads(&mut h);
            assert_eq!(reads.contains(&listing_read(100)), listed, "{name}: each poll");
        }
        h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(9), key: None } });
        h.fire(111);
        let reads = drain_kept_reads(&mut h);
        assert_eq!(reads.contains(&listing_read(100)), listed, "{name}: hint");
        // restart's fixture checks the listing's stored mark when one is owed.
        h.restart();
        let reads = drain_kept_reads(&mut h);
        assert!(!has_listing(&reads), "{name}: no extra listing after restart");
        h.event(Event::ReadAfresh);
        let reads = drain_kept_reads(&mut h);
        assert_eq!(reads.contains(&listing_read(100)), listed, "{name}: fresh listing from stored mark");
        assert_eq!(h.fresh_done, 1, "{name}: fresh pass completes");
    }
}

fn has_listing(reads: &[Read]) -> bool {
    for read in reads {
        if is_listing(read) {
            return true;
        }
    }
    false
}

fn is_listing(read: &Read) -> bool {
    match read {
        Read::Items { .. } => true,
        Read::Item { .. }
        | Read::Pull { .. }
        | Read::Branch { .. }
        | Read::Locate { .. }
        | Read::PullFor { .. }
        | Read::Reviews { .. }
        | Read::Statuses { .. }
        | Read::Remarks { .. }
        | Read::Branches { .. }
        | Read::PullFiles { .. }
        | Read::Compare { .. }
        | Read::Checks { .. }
        | Read::Job { .. }
        | Read::File { .. }
        | Read::Protection { .. }
        | Read::Settings
        | Read::Collaborators { .. }
        | Read::Permission { .. }
        | Read::Runs { .. }
        | Read::Run { .. }
        | Read::Tree { .. }
        | Read::Artifact { .. }
        | Read::Commits { .. } => false,
    }
}

#[test]
fn a_departed_repository_holds_its_room_and_rejoins_with_no_listing_mark() {
    for rejoin in [false, true] {
        let mut h = Harness::with_limits(Limits { repositories: 1, rate: 100, reserve: 0, ..LIMITS });
        h.keep(Box::new([watch(issue(), false)]));
        drain_kept_reads(&mut h);
        h.fire(105);
        let sent = h.one();
        assert_eq!(sent.op, Op::Read(listing_read(100)));
        h.keep(Box::new([]));
        assert!(h.records.get(&Key::Repository(REPO)).is_none(), "departed row erased");
        let other = Resource { repository: api::Repository { repository: 6, ..REPO }, what: What::Repository };
        h.keep(Box::new([watch(other.clone(), false)]));
        assert_eq!(h.kept.as_slice().last(), Some(&Err(Error::Busy)), "out call holds its room");
        if rejoin {
            h.keep(Box::new([watch(issue(), false)]));
            let saved = h.records.get(&Key::Repository(REPO)).expect("repository saved on rejoin");
            match saved {
                Stored::Repository(row) => assert_eq!(row.mark, None, "rejoin clears the old mark"),
                Stored::Live(_) | Stored::Writer(_) => panic!("repository record"),
            }
            assert!(!has_listing(&drain_kept_reads(&mut h)), "old call still out");
        }
        h.event(Event::Answered {
            call: sent.call,
            cost: 1,
            free: 0,
            rate: None,
            result: Err(Error::RateLimited { after: skein_lib::Duration::from_secs(1) }),
        });
        h.fire(106);
        if rejoin {
            assert!(drain_kept_reads(&mut h).contains(&listing_read(u64::MAX)), "new watch bootstraps");
        } else {
            h.keep(Box::new([watch(other, false)]));
            assert_eq!(h.kept.as_slice().last(), Some(&Ok(())), "answer releases the room");
            assert!(h.send().is_none(), "no departed listing is requeued");
        }
    }
}

#[test]
fn listing_departures() {
    enum CallState {
        Due,
        Queued,
        Out,
        RateRefused,
    }
    for (name, state, result, rejoin) in [
        ("due", CallState::Due, Ok(empty_listing(999)), false),
        ("queued", CallState::Queued, Ok(empty_listing(999)), false),
        ("out", CallState::Out, Ok(empty_listing(999)), false),
        ("already rate refused", CallState::RateRefused, Ok(empty_listing(999)), false),
        ("rate refused", CallState::Out, Err(Error::RateLimited { after: skein_lib::Duration::from_secs(1) }), false),
        ("rejoined before answer", CallState::Out, Ok(empty_listing(999)), true),
        (
            "rejoined before rate refusal",
            CallState::Out,
            Err(Error::RateLimited { after: skein_lib::Duration::from_secs(1) }),
            true,
        ),
    ] {
        let mut h = Harness::new();
        h.keep(Box::new([watch(issue(), false), watch(branch(), false)]));
        drain_kept_reads(&mut h);
        h.fire(105);
        let sent = match state {
            CallState::Due => None,
            CallState::Queued => {
                let mut out = skein_lib::Queue::uncapped();
                crate::keep::poll::pump(&mut h.domain, &h.env, &mut out);
                h.take(&mut out);
                None
            }
            CallState::Out => Some(h.one()),
            CallState::RateRefused => {
                let sent = h.one();
                h.event(Event::Answered {
                    call: sent.call,
                    cost: 1,
                    free: 0,
                    rate: None,
                    result: Err(Error::RateLimited { after: skein_lib::Duration::from_secs(1) }),
                });
                None
            }
        };
        h.keep(Box::new([watch(branch(), false)]));
        let saved = match h.records.get(&Key::Repository(REPO)).expect("repository retained") {
            Stored::Repository(record) => *record,
            Stored::Live(_) | Stored::Writer(_) => panic!("repository record"),
        };
        assert_eq!(saved.mark, None, "{name}");
        if rejoin {
            h.keep(Box::new([watch(issue(), false), watch(branch(), false)]));
        }
        let reads = drain_kept_reads(&mut h);
        assert!(!has_listing(&reads), "{name}: stopped call holds the listing off");
        if let Some(sent) = sent {
            h.event(Event::Answered { call: sent.call, cost: 1, free: 0, rate: None, result });
            assert_eq!(
                h.records.get(&Key::Repository(REPO)),
                Some(&Stored::Repository(saved)),
                "{name}: old answer dropped"
            );
        }
        h.fire(106);
        let reads = drain_kept_reads(&mut h);
        assert_eq!(reads.contains(&listing_read(u64::MAX)), rejoin, "{name}: new watch bootstraps");
        assert!(!reads.contains(&listing_read(100)), "{name}: no old listing");
        assert!(!has_listing(&reads) || rejoin, "{name}: stopped listing stays off");
        if !rejoin {
            h.keep(Box::new([watch(issue(), false), watch(branch(), false)]));
            assert!(drain_kept_reads(&mut h).contains(&listing_read(u64::MAX)), "{name}: later return bootstraps");
        }
    }
}

#[test]
fn item_polls_name_since_only_after_a_completed_comment_scan() {
    #[derive(Clone, Copy)]
    enum Next {
        Ready,
        Listing,
        Slow,
        Repair,
    }
    let updated = skein_lib::Time::from_nanos(1_700_000_000_000_000_000);
    for (name, participating, answers, next, after, since) in [
        ("no cached item", true, &[][..], Next::Ready, 0, skein_lib::Time::ZERO),
        ("finished", true, &[(false, true)][..], Next::Listing, 1, updated),
        ("finished empty", true, &[(false, false)][..], Next::Listing, 0, updated),
        ("scan continues", true, &[(true, true)][..], Next::Ready, 1, skein_lib::Time::ZERO),
        (
            "scan ended on later answer",
            true,
            &[(true, true), (false, true)][..],
            Next::Listing,
            2,
            skein_lib::Time::from_nanos(updated.as_nanos() + 1),
        ),
        ("slow rescans from zero", true, &[(false, true)][..], Next::Slow, 0, skein_lib::Time::ZERO),
        ("hint repairs from zero", true, &[(false, true)][..], Next::Repair, 0, skein_lib::Time::ZERO),
        ("not participating", false, &[][..], Next::Ready, u64::MAX, skein_lib::Time::from_nanos(u64::MAX)),
        (
            "not participating with cache",
            false,
            &[(false, false)][..],
            Next::Listing,
            u64::MAX,
            skein_lib::Time::from_nanos(u64::MAX),
        ),
    ] {
        let mut h = Harness::new();
        h.keep(Box::new([watch(issue(), participating)]));
        h.clock();
        for (index, (more, with_comment)) in answers.iter().enumerate() {
            let read = h.one();
            let mut item = summary();
            item.updated = skein_lib::Time::from_nanos(updated.as_nanos() + u64::try_from(index).unwrap());
            let comments: Box<[api::Comment]> =
                if *with_comment { Box::new([comment(u64::try_from(index).unwrap() + 1)]) } else { Box::new([]) };
            h.answer(read, Answer::Item { item, comments, more: *more });
        }
        match next {
            Next::Ready => {}
            Next::Repair => {
                h.event(Event::Hint { hint: api::Hint { repository: REPO, change: Change::Item(9), key: None } });
                h.fire(101);
                let listing = h.one();
                h.answer(listing, Answer::Items { cut: false, items: Box::new([]), more: false, now: at(101) });
            }
            Next::Listing | Next::Slow => {
                let now = match next {
                    Next::Slow => 160,
                    Next::Listing => 105,
                    Next::Ready | Next::Repair => unreachable!("listing or slow trigger"),
                };
                h.fire(now);
                let listing = h.one();
                let mut item = summary();
                item.updated = updated;
                h.answer(listing, Answer::Items { cut: false, items: Box::new([item]), more: false, now: at(now) });
                h.fire(now + 1);
            }
        }
        let read = h.one();
        assert_eq!(read.op, Op::Read(Read::Item { number: 9, after, since }), "{name}");
    }
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture keeps only a pull and its repository")]
fn hints_read_only_the_pulls_whose_branch_facts_can_move() {
    skein_rows::check(
        [
            ("head commit", (Change::Commit([1; 32]), false)),
            ("base commit", (Change::Commit([2; 32]), false)),
            ("head branch", (Change::Branch(Box::from(b"topic".as_slice())), true)),
            ("base branch", (Change::Branch(Box::from(b"main".as_slice())), true)),
            ("other branch", (Change::Branch(Box::from(b"other".as_slice())), false)),
        ],
        |(change, reads_pull)| {
            let mut h = Harness::new();
            h.keep(Box::new([watch(pull(), false)]));
            h.clock();
            let item = h.one();
            h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
            let read = h.one();
            h.answer(read, Answer::Pull(current()));
            h.event(Event::Hint { hint: api::Hint { repository: REPO, change, key: None } });
            h.fire(101);
            let mut pulls = 0;
            for _ in 0_u32..4 {
                let Some(read) = h.send() else { break };
                let answer = match read.op {
                    Op::Read(Read::Items { .. }) => {
                        Answer::Items { cut: false, items: Box::new([]), more: false, now: at(101) }
                    }
                    Op::Read(Read::Item { .. }) => {
                        Answer::Item { item: summary(), comments: Box::new([]), more: false }
                    }
                    Op::Read(Read::Pull { ci: true, .. }) => {
                        pulls += 1;
                        Answer::Pull(current())
                    }
                    _ => panic!("unexpected hinted read: {read:?}"),
                };
                h.answer(read, answer);
            }
            assert_eq!(pulls, u32::from(reads_pull));
        },
    );
}

#[test]
fn pull_size_alone_is_saved_without_news_and_a_new_head_carries_it() {
    let mut h = Harness::new();
    h.keep(Box::new([watch(pull(), false)]));
    h.clock();
    let item = h.one();
    h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
    let read = h.one();
    h.answer(read, Answer::Pull(current()));
    h.news.clear();
    let mut observed = current();
    observed.size = Some(api::Size { files: 3, additions: 3, deletions: 0 });
    for new_head in [false, true] {
        h.restart();
        let item = h.one();
        h.answer(item, Answer::Item { item: summary(), comments: Box::new([]), more: false });
        let read = h.one();
        assert_eq!(read.op, Op::Read(Read::Pull { number: 9, ci: true }));
        if new_head {
            observed.commit = [7; 32];
        }
        h.answer(read, Answer::Pull(observed.clone()));
        assert_eq!(h.record(&pull()).cached.pull.as_ref(), Some(&observed));
        assert_eq!(h.domain.cached(&pull()).expect("kept pull").pull.as_ref(), Some(&observed));
        if new_head {
            assert_eq!(h.news.as_slice(), &[Answer::Pull(observed.clone())]);
        } else {
            assert!(h.news.is_empty(), "a size alone is not watch news");
        }
    }
}
