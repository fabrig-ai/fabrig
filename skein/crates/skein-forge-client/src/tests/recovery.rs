use super::fixtures::{Harness, configured};
use super::fixtures::{keep as kept, outbox as writing, recovery as fixture};
use super::{LIMITS, REPO, at};
use crate::{Condition, Echo, Event, Fact, Limits, Made, Outcome, RecoveryClock, Request, Resource, Stored, What};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Change, Error, Op, Read, Write};
use skein_lib::{Duration, List, Token, Wall};

fn updating(style: api::UpdateStyle) -> writing::Harness {
    let mut h = writing::Harness::new();
    h.make(1, Write::Update { number: 9, style }, Condition::Update { head: [1; 32], base: [2; 32] });
    h.clock();
    check_update(&mut h, writing::pull([1; 32]));
    h
}

fn check_update(h: &mut writing::Harness, pull: api::Pull) {
    let check = h.one();
    assert_eq!(check.op, Op::Read(Read::Pull { number: 9, ci: false }));
    h.answer(check, Ok(Answer::Pull(pull)));
}

fn look_update(h: &mut writing::Harness, head: [u8; 32], found: bool) {
    check_update(h, writing::pull(head));
    let compare = h.one();
    assert_eq!(compare.op, Op::Read(Read::Commits { before: head, after: [2; 32], page: 1 }));
    h.answer(
        compare,
        Ok(Answer::Commits {
            commits: if found {
                Box::new([])
            } else {
                Box::new([api::CommitMessage { id: [2; 32], parents: Box::new([]), message: Box::new([]) }])
            },
            more: false,
        }),
    );
}

fn lose_update(h: &mut writing::Harness, style: api::UpdateStyle) {
    let write = h.one();
    assert_eq!(write.op, Op::Write(Write::Update { number: 9, style }));
    h.answer(write, Err(Error::Timeout));
}

fn miss_update(h: &mut writing::Harness) {
    look_update(h, [1; 32], false);
    let deadline = h.store.get(&1).expect("uncertain update").attempt.expect("saved attempt").deadline;
    assert_eq!(h.domain.next_deadline(), Some(deadline));
    assert!(h.send().is_none(), "wait for the original attempt's deadline");
    h.env.wall = Wall::from_nanos(deadline.as_nanos());
    h.fire(deadline.as_nanos().checked_div(at(1).as_nanos()).expect("positive fixture second"));
    look_update(h, [1; 32], false);
    assert!(!h.outcomes.as_slice().contains(&(1, Outcome::Held)));
}

#[test]
fn a_lost_update_not_found_is_checked_and_made_again() {
    for style in [api::UpdateStyle::Merge, api::UpdateStyle::Rebase] {
        let mut h = updating(style);
        lose_update(&mut h, style);
        let original = h.store.get(&1).expect("first attempt").clone();
        miss_update(&mut h);
        assert!(!h.outcomes.as_slice().contains(&(1, Outcome::Held)));
        check_update(&mut h, writing::pull([1; 32]));
        lose_update(&mut h, style);
        let remade = h.store.get(&1).expect("second attempt");
        let attempt = remade.attempt.expect("new attempt");
        assert_eq!(remade.start, original.start);
        assert_eq!(attempt.sent, h.env.now);
        assert_eq!(attempt.deadline, attempt.sent.saturating_add(LIMITS.lifetime).saturating_add(LIMITS.clock_margin));
        assert_eq!(attempt.wall, h.env.wall);
        assert_eq!(
            attempt.expires,
            Wall::from_nanos(
                attempt
                    .wall
                    .as_nanos()
                    .saturating_add(LIMITS.lifetime.as_nanos())
                    .saturating_add(LIMITS.clock_margin.as_nanos())
            )
        );
        assert_ne!(remade.attempt, original.attempt);
        assert_eq!(h.writes, 2);
    }
}

#[test]
fn a_lost_update_fails_timeout_after_write_attempts() {
    for style in [api::UpdateStyle::Merge, api::UpdateStyle::Rebase] {
        let mut h = updating(style);
        for attempt in 1..=LIMITS.write_attempts {
            lose_update(&mut h, style);
            miss_update(&mut h);
            assert!(!h.outcomes.as_slice().contains(&(1, Outcome::Held)));
            if attempt < LIMITS.write_attempts {
                check_update(&mut h, writing::pull([1; 32]));
            }
        }
        assert_eq!(h.writes, LIMITS.write_attempts);
        assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Failed(Error::Timeout))));
        assert!(h.store.is_empty());
        assert!(h.send().is_none());
    }
}

#[test]
fn an_update_made_again_is_found_by_a_later_look() {
    for style in [api::UpdateStyle::Merge, api::UpdateStyle::Rebase] {
        let mut h = updating(style);
        lose_update(&mut h, style);
        miss_update(&mut h);
        check_update(&mut h, writing::pull([1; 32]));
        lose_update(&mut h, style);
        look_update(&mut h, [3; 32], true);
        assert_eq!(
            h.outcomes.as_slice().last(),
            Some(&(1, Outcome::Made { made: Made::Updated([3; 32]), found: true }))
        );
        assert_eq!(h.writes, 2);
        assert!(h.store.is_empty());
    }
}

#[test]
fn an_update_checked_again_fails_stale_when_its_head_or_base_moved() {
    for style in [api::UpdateStyle::Merge, api::UpdateStyle::Rebase] {
        for moved_head in [true, false] {
            let mut h = updating(style);
            lose_update(&mut h, style);
            miss_update(&mut h);
            let mut pull = writing::pull(if moved_head { [3; 32] } else { [1; 32] });
            if !moved_head {
                pull.base_commit = Some([4; 32]);
            }
            check_update(&mut h, pull);
            assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Failed(Error::Stale))));
            assert_eq!(h.writes, 1);
            assert!(h.store.is_empty());
            assert!(h.send().is_none());
        }
    }
}

#[test]
fn a_held_update_remade_is_looked_for_first() {
    for style in [api::UpdateStyle::Merge, api::UpdateStyle::Rebase] {
        for found in [true, false] {
            let mut h = updating(style);
            lose_update(&mut h, style);
            let saved = h.store.get(&1).expect("held entry's started attempt").clone();
            h.domain = configured(&h.env.limits);
            h.step(Event::Restored { clock: RecoveryClock::Monotonic });
            h.step(Event::Remake { entry: saved.clone() });
            look_update(&mut h, if found { [3; 32] } else { [1; 32] }, found);
            assert_eq!(h.writes, 1, "remake looks before writing");
            if found {
                assert_eq!(
                    h.outcomes.as_slice().last(),
                    Some(&(1, Outcome::Made { made: Made::Updated([3; 32]), found: true }))
                );
            } else {
                assert_eq!(h.domain.next_deadline(), Some(saved.attempt.expect("saved deadline").deadline));
                h.fire(111);
                look_update(&mut h, [1; 32], false);
                check_update(&mut h, writing::pull([1; 32]));
                lose_update(&mut h, style);
                assert_eq!(h.writes, 2);
            }
            assert!(!h.outcomes.as_slice().contains(&(1, Outcome::Held)));
        }
    }
}

#[test]
fn restart_reads_before_settling_the_outbox() {
    let mut h = kept::Harness::new();
    let mut row = fixture::live();
    row.watch = kept::watch(kept::issue(), false);
    h.domain = configured(&h.env.limits);
    h.event(Event::Restore { record: Stored::Live(row) });
    h.event(Event::Restore { record: Stored::Repository(fixture::repository()) });
    h.event(Event::Restored { clock: RecoveryClock::Monotonic });
    h.event(Event::Writer { resource: kept::issue(), taken: true });
    h.event(Event::PauseOutbox);
    let saved = fixture::uncertain_comment().store.get(&1).expect("saved attempt").clone();
    h.event(Event::Make { entry: saved });
    h.event(Event::ReadAfresh);
    let listing = h.one();
    assert_eq!(listing.op, Op::Read(Read::Items { single: false, since: at(fixture::START_AT), page: 1, kind: None }));
    h.answer(listing, Answer::Items { cut: false, items: Box::new([]), more: false, now: at(fixture::START_AT) });
    let object = h.one();
    assert_eq!(
        object.op,
        Op::Read(Read::Item { number: 9, after: u64::MAX, since: skein_lib::Time::from_nanos(u64::MAX) })
    );
    h.answer(object, fixture::empty_item());
    assert_eq!(h.fresh_done, 1);
    assert!(h.send().is_none());
    h.event(Event::SettleOutbox);
    let find = h.one();
    assert_eq!(find.op, Op::Read(Read::Item { number: 9, after: 0, since: skein_lib::Time::ZERO }));
    h.answer(find, fixture::empty_item());
    assert_eq!(h.outbox_done, 1);
    assert!(h.send().is_none());
    assert_eq!(h.fresh_done, 1);
}
#[test]
fn a_fresh_pass_ends_when_its_last_pending_watch_is_dropped() {
    let mut h = kept::Harness::new();
    h.domain = configured(&h.env.limits);
    h.event(Event::Restored { clock: RecoveryClock::Monotonic });
    h.keep(Box::new([kept::watch(kept::issue(), false)]));
    h.event(Event::ReadAfresh);
    assert_eq!(h.fresh_done, 0);
    h.keep(Box::new([]));
    assert_eq!(h.fresh_done, 1);
    h.keep(Box::new([]));
    assert_eq!(h.fresh_done, 1);
}

#[test]
fn remake_releases_a_held_entry() {
    let (mut h, saved) = fixture::held_comment();
    h.store.insert(1, saved.clone()).expect("durable held entry");
    h.step(Event::Remake { entry: saved });
    let find = h.one();
    h.answer(find, Ok(fixture::empty_item()));
    let write = h.one();
    assert_eq!(write.op, Op::Write(writing::comment()));
    h.answer(write, Ok(Answer::Commented(9)));
    assert_eq!(h.writes, 2);
    assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Made { made: Made::Commented(9), found: false })));
}
#[test]
#[expect(clippy::too_many_lines, reason = "the named input records share one refusal body")]
fn invalid_saved_records_leave_restoration_failed_and_busy() {
    skein_rows::check(
        [
            ("restore_refuses_bad_pages: comment zero", {
                let mut row = fixture::live();
                row.comment_page = 0;
                saved_pair(row)
            }),
            ("restore_refuses_bad_pages: comment overflow", {
                let mut row = fixture::live();
                row.comment_page = fixture::EXCESS_INBOX;
                saved_pair(row)
            }),
            ("restore_refuses_bad_pages: review zero", {
                let mut row = fixture::live();
                row.review_page = 0;
                saved_pair(row)
            }),
            ("restore_refuses_bad_pages: review overflow", {
                let mut row = fixture::live();
                row.review_page = fixture::EXCESS_INBOX;
                saved_pair(row)
            }),
            ("restore_refuses_bad_stamps: comments overflow", {
                let mut row = fixture::live();
                row.comments = fixture::deliveries(fixture::EXCESS_INBOX, false);
                saved_pair(row)
            }),
            ("restore_refuses_bad_stamps: comments repeated", {
                let mut row = fixture::live();
                row.comments = fixture::deliveries(2, true);
                saved_pair(row)
            }),
            ("restore_refuses_bad_stamps: reviews overflow", {
                let mut row = fixture::live();
                row.reviews = fixture::deliveries(fixture::EXCESS_INBOX, false);
                saved_pair(row)
            }),
            ("restore_refuses_bad_stamps: reviews repeated", {
                let mut row = fixture::live();
                row.reviews = fixture::deliveries(2, true);
                saved_pair(row)
            }),
            ("restore_refuses_oversized_caches: item bytes", {
                let mut row = fixture::live();
                let mut value = kept::summary();
                value.title = fixture::repeated(b'a', fixture::EXCESS_ANSWER_BYTES);
                row.cached.item = Some(value);
                saved_pair(row)
            }),
            ("restore_refuses_oversized_caches: item rows", {
                let mut row = fixture::live();
                let mut value = kept::summary();
                value.labels = fixture::repeated(Box::<[u8]>::default(), fixture::EXCESS_ROWS);
                row.cached.item = Some(value);
                saved_pair(row)
            }),
            ("restore_refuses_oversized_caches: pull bytes", {
                let mut row = fixture::live();
                let mut value = kept::current();
                value.head = fixture::repeated(b'a', fixture::EXCESS_ANSWER_BYTES);
                row.cached.pull = Some(value);
                saved_pair(row)
            }),
            ("restore_refuses_oversized_caches: pull rows", {
                let mut row = fixture::live();
                let mut value = kept::current();
                value.reviewers = fixture::repeated(7, fixture::EXCESS_ROWS);
                row.cached.pull = Some(value);
                saved_pair(row)
            }),
            ("restore_refuses_oversized_echoes", {
                let mut row = fixture::live();
                row.echoes = fixture::repeated(Echo::Comment(1), fixture::EXCESS_ROWS);
                saved_pair(row)
            }),
            (
                "restore_refuses_unpaired_rows: missing repository",
                Box::<[Stored]>::from([Stored::Live(fixture::live())]),
            ),
            (
                "restore_refuses_unpaired_rows: missing live",
                Box::<[Stored]>::from([Stored::Repository(fixture::repository())]),
            ),
            ("restore_refuses_capacity_overflow: resources", {
                let mut rows = List::with_capacity(fixture::EXCESS_RESOURCES + 1);
                for index in 0..fixture::EXCESS_RESOURCES {
                    let mut row = fixture::live();
                    row.watch.resource.what = What::Issue(u64::from(index));
                    rows.push(Stored::Live(row)).expect("fixture capacity");
                }
                rows.push(Stored::Repository(fixture::repository())).expect("fixture capacity");
                rows.into_boxed()
            }),
            ("restore_refuses_capacity_overflow: repositories", {
                let mut rows = List::with_capacity(fixture::EXCESS_REPOSITORIES + 1);
                for index in 0..fixture::EXCESS_REPOSITORIES {
                    let mut row = fixture::repository();
                    row.repository.repository += index;
                    rows.push(Stored::Repository(row)).expect("fixture capacity");
                }
                rows.into_boxed()
            }),
            ("restore_refuses_invalid_watches: empty branch", {
                let mut row = fixture::live();
                row.watch = kept::watch(Resource { repository: REPO, what: What::Branch(Box::new([])) }, false);
                saved_pair(row)
            }),
            ("restore_refuses_invalid_watches: long branch", {
                let mut row = fixture::live();
                row.watch = kept::watch(
                    Resource {
                        repository: REPO,
                        what: What::Branch(fixture::repeated(b'a', fixture::EXCESS_OP_BYTES)),
                    },
                    false,
                );
                saved_pair(row)
            }),
            ("restore_refuses_invalid_watches: participating repository", {
                let mut row = fixture::live();
                row.watch = kept::watch(Resource { repository: REPO, what: What::Repository }, true);
                saved_pair(row)
            }),
            ("restore_refuses_invalid_watches: participating branch", {
                let mut row = fixture::live();
                row.watch = kept::watch(Resource { repository: REPO, what: kept::branch().what }, true);
                saved_pair(row)
            }),
            ("restore_refuses_exact_one_past_cached_bytes", {
                let mut row = fixture::live();
                let mut current = kept::current();
                current.head = fixture::repeated(b'a', fixture::PULL_HEAD_BOUND + 1);
                row.cached.pull = Some(current);
                saved_pair(row)
            }),
        ],
        |records| fixture::failed_restore("saved records", records),
    );
}

#[test]
fn keep_refuses_invalid_participation() {
    for what in [What::Repository, kept::branch().what] {
        let mut h = fixture::ready();
        let watch = kept::watch(Resource { repository: REPO, what }, true);
        assert_eq!(
            h.step(Event::Keep { owner: Token::new(1), watches: Box::new([watch]) }).as_ref(),
            &[Request::Kept { owner: Token::new(1), result: Err(Error::Refused) }]
        );
        assert!(h.send().is_none());
    }
}
#[test]
fn keep_refuses_invalid_branch_names() {
    for branch in [Box::from(&b""[..]), fixture::repeated(b'a', fixture::EXCESS_OP_BYTES)] {
        let mut h = fixture::ready();
        let length = skein_lib::len_u64(branch.len());
        let watch = kept::watch(Resource { repository: REPO, what: What::Branch(branch) }, false);
        assert_eq!(
            h.step(Event::Keep { owner: Token::new(1), watches: Box::new([watch]) }).as_ref(),
            &[Request::Kept {
                owner: Token::new(1),
                result: Err(Error::TooLarge(if length == 0 {
                    api::Excess::unnamed()
                } else {
                    api::Excess::exact(api::Bound::OpBytes, 256, length)
                }))
            }]
        );
        assert!(h.domain.cached(&kept::branch()).is_none());
        assert!(h.send().is_none());
    }
}
#[test]
fn keep_is_busy_around_failed_restore() {
    let mut h = Harness::new(LIMITS);
    fixture::assert_busy(&mut h, "before restore");
    let mut bad = fixture::live();
    bad.comment_page = 0;
    h.step(Event::Restore { record: Stored::Live(bad) });
    fixture::paired(&mut h, fixture::live());
    h.step(Event::Restored { clock: RecoveryClock::Monotonic });
    fixture::assert_busy(&mut h, "corrected rows stay failed");
    h.step(Event::Read { owner: Token::new(1), repository: REPO, read: Read::Pull { number: 9, ci: false } });
    let sent = h.one();
    h.answer(&sent, 1, Ok(Answer::Pull(kept::current())));
    assert!(h.domain.restoration_failed());
    fixture::assert_busy(&mut h, "after successful read");
}
#[test]
fn oversized_hints_read_nothing() {
    for (name, resource, change, key, answer) in [
        (
            "branch",
            kept::branch(),
            Change::Branch(fixture::repeated(b'a', fixture::EXCESS_OP_BYTES)),
            None,
            Answer::Commit([1; 32]),
        ),
        (
            "key",
            kept::issue(),
            Change::Item(9),
            Some(fixture::repeated(b'a', fixture::EXCESS_OP_BYTES)),
            fixture::empty_item(),
        ),
    ] {
        let mut h = kept::Harness::new();
        h.keep(Box::new([kept::watch(resource.clone(), false)]));
        if resource != kept::branch() {
            h.clock();
        }
        let sent = h.one();
        h.answer(sent, answer);
        while h.domain.pop_fact().is_some() {}
        let deadline = h.domain.next_deadline();
        h.event(Event::Hint { hint: api::Hint { repository: REPO, change, key } });
        assert_eq!(h.domain.pop_fact(), Some(Fact::Refused), "{name}");
        assert_eq!(h.domain.pop_fact(), None, "{name}");
        assert_eq!(h.domain.next_deadline(), deadline, "{name}");
        assert!(h.send().is_none(), "{name}");
    }
}
#[test]
fn valid_rejects_invalid_limits() {
    for (name, limits) in [
        ("pending", Limits { pending: 0, ..LIMITS }),
        ("calls", Limits { calls: 0, ..LIMITS }),
        ("rate", Limits { rate: 0, ..LIMITS }),
        ("window", Limits { window: Duration::ZERO, ..LIMITS }),
        ("read attempts", Limits { read_attempts: 0, ..LIMITS }),
        ("backoff", Limits { backoff: Duration::ZERO, ..LIMITS }),
        ("entries", Limits { entries: 0, ..LIMITS }),
        ("write attempts", Limits { write_attempts: 0, ..LIMITS }),
        ("lifetime", Limits { lifetime: Duration::ZERO, ..LIMITS }),
        ("margin", Limits { clock_margin: Duration::ZERO, ..LIMITS }),
        ("rows", Limits { rows: 0, ..LIMITS }),
        ("inbox", Limits { inbox: 0, ..LIMITS }),
        ("resources", Limits { resources: 0, ..LIMITS }),
        ("repositories", Limits { repositories: 0, ..LIMITS }),
        ("poll", Limits { poll: Duration::ZERO, ..LIMITS }),
        ("hinted", Limits { hinted: Duration::ZERO, ..LIMITS }),
        ("slow", Limits { slow: Duration::ZERO, ..LIMITS }),
        ("settle", Limits { settle: Duration::ZERO, ..LIMITS }),
        ("parallel", Limits { calls: LIMITS.pending + 1, ..LIMITS }),
        ("reserve", Limits { reserve: LIMITS.rate + 1, ..LIMITS }),
        ("backoff order", Limits { backoff: LIMITS.backoff_max.saturating_add(Duration::from_secs(1)), ..LIMITS }),
        ("poll order", Limits { poll: LIMITS.poll_max.saturating_add(Duration::from_secs(1)), ..LIMITS }),
        ("backoff max", Limits { backoff_max: Duration::ZERO, ..LIMITS }),
        ("poll max", Limits { poll_max: Duration::ZERO, ..LIMITS }),
    ] {
        assert!(!crate::valid(&limits), "{name}");
    }
}
#[test]
fn valid_accepts_permitted_zeroes() {
    for (name, limits) in [
        ("op bytes", Limits { op_bytes: 0, ..LIMITS }),
        ("answer bytes", Limits { answer_bytes: 0, ..LIMITS }),
        ("reserve", Limits { reserve: 0, ..LIMITS }),
        ("facts", Limits { facts: 0, ..LIMITS }),
    ] {
        assert!(crate::valid(&limits), "{name}");
    }
}
#[test]
fn invalid_reads_make_no_call() {
    for read in [
        Read::Items { single: false, since: at(fixture::START_AT), page: 0, kind: None },
        Read::Reviews { number: 9, page: 0, skip: 0 },
        Read::Statuses { commit: [1; 32], page: 0 },
        Read::Remarks { number: 9, review: 1, page: 0, skip: 0 },
        Read::PullFiles { number: 9, head: [1; 32], page: 0, skip: 0, sides: true },
        Read::Collaborators { page: 0 },
        Read::File { head: [1; 32], path: Box::from(&b"kit"[..]), max_bytes: 0 },
        Read::File { head: [1; 32], path: Box::from(&b"kit"[..]), max_bytes: fixture::EXCESS_ANSWER_BYTES },
        Read::Job {
            part: api::LogPart::End,
            attempt: api::JobAttempt { head: [1; 32], run: 1, job: 1, attempt: 1 },
            max_bytes: 0,
        },
        Read::Job {
            part: api::LogPart::End,
            attempt: api::JobAttempt { head: [1; 32], run: 1, job: 1, attempt: 1 },
            max_bytes: fixture::EXCESS_ANSWER_BYTES,
        },
    ] {
        fixture::refused_read(read);
    }
}
#[test]
fn rate_refused_writes_save_a_new_attempt() {
    let mut h = writing::Harness::new();
    h.make(1, writing::comment(), Condition::None);
    h.clock();
    let sent = h.one();
    let saved = h.store.get(&1).expect("saved attempt").clone();
    assert_eq!(saved.attempt.expect("saved attempt").deadline, at(fixture::ATTEMPT_DEADLINE));
    let call = sent.call;
    h.answer(sent, Err(Error::RateLimited { after: Duration::from_secs(2) }));
    h.env.wall = Wall::from_nanos(at(fixture::RETRY_AT).as_nanos());
    h.fire(fixture::RETRY_AT);
    let resent = h.one();
    assert_ne!(resent.call, call);
    assert_eq!(resent.op, Op::Write(writing::comment()));
    let entry = h.store.get(&1).expect("resent attempt");
    assert_eq!(entry.start, saved.start);
    assert_eq!(entry.failures, 0);
    assert_eq!(
        entry.attempt,
        Some(crate::Attempt {
            sent: at(fixture::RETRY_AT),
            deadline: at(fixture::RETRY_DEADLINE),
            wall: h.env.wall,
            expires: Wall::from_nanos(at(fixture::RETRY_DEADLINE).as_nanos())
        })
    );
    h.answer(resent, Err(Error::Timeout));
    let find = h.one();
    assert_eq!(find.op, Op::Read(Read::Item { number: 9, after: 0, since: skein_lib::Time::ZERO }));
    assert_eq!(
        h.store.get(&1).expect("find keeps attempt").attempt.expect("attempt").deadline,
        at(fixture::RETRY_DEADLINE)
    );
}
#[test]
fn attempted_or_unknown_withdrawals_are_noops() {
    let mut h = fixture::uncertain_comment();
    let find = h.one();
    while h.domain.pop_fact().is_some() {}
    h.outcomes.clear();
    for number in [1, 99] {
        let mut out = skein_lib::Queue::uncapped();
        crate::step(&mut h.domain, &h.env, Event::Withdraw { entry: number }, &mut out);
        assert!(out.is_empty(), "{number}");
        assert_eq!(h.domain.pop_fact(), None, "{number}");
    }
    let mut comment = kept::comment(9);
    comment.author = 1;
    comment.key = Some(crate::effect_key(b"kit", b":task:milestone", LIMITS.op_bytes).expect("key fits"));
    h.answer(find, Ok(Answer::Item { item: writing::summary(), comments: Box::new([comment]), more: false }));
    assert_eq!(h.outcomes.as_slice(), &[(1, Outcome::Made { made: Made::Commented(9), found: true })]);
    assert_eq!(h.writes, 1);
}
#[test]
fn echo_landing_rearms_the_resting_resource() {
    for (name, review) in [("comment", false), ("review", true)] {
        let (mut h, resource) = fixture::resting_echo(review);
        assert_eq!(h.domain.next_deadline(), Some(at(fixture::RESTING_AT)), "{name}");
        fixture::land_echo(&mut h, review);
        assert_eq!(
            h.record(&resource).echoes.as_ref(),
            &[if review { Echo::Review(9) } else { Echo::Comment(9) }],
            "{name}"
        );
        assert_eq!(h.domain.next_deadline(), Some(at(fixture::HINT_AT)), "{name}");
        h.fire(fixture::HINT_AT);
        let object = h.one();
        assert_eq!(object.op, Op::Read(Read::Item { number: 9, after: 0, since: kept::summary().updated }), "{name}");
        let mut own = kept::comment(9);
        own.author = 1;
        h.answer(
            object,
            Answer::Item {
                item: kept::summary(),
                comments: if review { Box::new([]) } else { Box::new([own]) },
                more: false,
            },
        );
        assert!(h.news.is_empty(), "{name}");
    }
}
#[test]
fn issue_find_requires_a_believed_key_without_provenance() {
    for (name, key, author, found) in [
        ("kit", &b"\x01\x00\x03kita"[..], 1, true),
        ("writer", &b"\x01\x00\x03kita"[..], 2, false),
        ("wrong", &b"\x01\x00\x03kitb"[..], 1, false),
        ("foreign", &b"\x01\x00\x03fooa"[..], 1, false),
        ("unframed", &b"a"[..], 1, false),
    ] {
        let expected = if name == "foreign" || name == "unframed" { key } else { &b"\x01\x00\x03kita"[..] };
        let mut h = writing::Harness::new();
        h.make(
            1,
            Write::CreateIssue { key: Box::from(expected), title: Box::new([]), body: Box::new([]) },
            Condition::None,
        );
        h.clock();
        let write = h.one();
        h.answer(write, Err(Error::Timeout));
        let mut item = writing::summary();
        item.key = Some(Box::from(key));
        item.author = author;
        let find = h.one();
        h.answer(
            find,
            Ok(Answer::Items { cut: false, items: Box::new([item.clone()]), more: false, now: at(fixture::START_AT) }),
        );
        if found {
            assert_eq!(
                h.outcomes.as_slice().last(),
                Some(&(1, Outcome::Made { made: Made::Created(9), found: true })),
                "{name}"
            );
        } else {
            assert_eq!(h.outcomes.as_slice(), &[(1, Outcome::Uncertain)], "{name}");
            h.fire(fixture::ATTEMPT_DEADLINE);
            let find = h.one();
            h.answer(
                find,
                Ok(Answer::Items {
                    cut: false,
                    items: Box::new([item]),
                    more: false,
                    now: at(fixture::ATTEMPT_DEADLINE),
                }),
            );
            assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Held)), "{name}");
        }
        assert_eq!(h.writes, 1, "{name}");
    }
}
#[test]
fn comment_and_review_finds_require_believed_original_keys() {
    for (name, key) in
        [("kit", &b"\x01\x00\x03kita"[..]), ("foreign", &b"\x01\x00\x03fooa"[..]), ("unframed", &b"a"[..])]
    {
        for review in [false, true] {
            let mut h = fixture::recovering_key(review, key);
            let answer = fixture::keyed_answer(review, key);
            let find = h.one();
            h.answer(find, Ok(answer.clone()));
            if name == "kit" {
                let made = if review { Made::Reviewed(9) } else { Made::Commented(9) };
                assert_eq!(
                    h.outcomes.as_slice().last(),
                    Some(&(1, Outcome::Made { made, found: true })),
                    "{name} review={review}"
                );
            } else {
                assert_eq!(h.outcomes.as_slice(), &[(1, Outcome::Uncertain)], "{name} review={review}");
                h.fire(fixture::ATTEMPT_DEADLINE);
                let find = h.one();
                h.answer(find, Ok(answer));
                assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Held)), "{name} review={review}");
            }
            assert_eq!(h.writes, 1, "{name} review={review}");
        }
    }
}
#[test]
fn oversized_and_lost_write_answers_are_uncertain() {
    let branches = Answer::Branches(Box::new([Box::from(&b"kit"[..])]));
    crate::bounds::answer(&branches, &Limits { answer_bytes: 19, ..LIMITS }).expect("fits");
    assert!(crate::bounds::answer(&branches, &Limits { answer_bytes: 18, ..LIMITS }).is_err());
    for (name, answer, error) in
        [("oversized", Ok(branches), Error::InvalidAnswer), ("lost", Err(Error::Timeout), Error::Timeout)]
    {
        let mut h = writing::Harness::new();
        h.env.limits.answer_bytes = 18;
        h.make(1, writing::comment(), Condition::None);
        h.clock();
        let write = h.one();
        h.answer(write, answer);
        fixture::failed_fact(&mut h, error);
        assert_eq!(h.store.get(&1).expect("uncertain entry saved").failures, 1, "{name}");
        assert_eq!(h.outcomes.as_slice(), &[(1, Outcome::Uncertain)], "{name}");
        assert_eq!(h.one().op, Op::Read(Read::Item { number: 9, after: 0, since: skein_lib::Time::ZERO }), "{name}");
        assert_eq!(h.writes, 1, "{name}");
    }
}
#[test]
fn exists_after_a_write_finds_under_its_attempt() {
    let mut h = writing::Harness::new();
    h.make(
        1,
        Write::OpenPull {
            head: Box::from(&b"topic"[..]),
            base: Box::from(&b"main"[..]),
            title: Box::new([]),
            body: Box::new([]),
        },
        Condition::None,
    );
    h.clock();
    let write = h.one();
    let saved = h.store.get(&1).expect("durable attempt").clone();
    assert_eq!(saved.attempt.expect("attempt").deadline, at(fixture::ATTEMPT_DEADLINE));
    h.answer(write, Err(Error::Exists));
    let find = h.one();
    assert_eq!(find.op, Op::Read(Read::PullFor { head: Box::from(&b"topic"[..]), base: Box::from(&b"main"[..]) }));
    assert_eq!(h.store.get(&1).expect("find retains entry"), &saved);
    h.answer(find, Ok(Answer::Pull(kept::current())));
    assert_eq!(h.outcomes.as_slice(), &[(1, Outcome::Made { made: Made::Created(9), found: true })]);
    assert_eq!(h.writes, 1);
}

#[test]
fn uncertain_comment_becomes_held_at_its_deadline() {
    let (h, _) = fixture::held_comment();
    assert_eq!(h.outcomes.as_slice(), &[(1, Outcome::Uncertain), (1, Outcome::Held)]);
}
#[test]
fn valid_saved_records_restore_and_admit_resource_keeping() {
    skein_rows::check(
        [
            ("fresh_paired_restoration_accepts_keep", fixture::live()),
            ("restore_accepts_exact_page_and_stamp_bounds: comment page", {
                let mut row = fixture::live();
                row.comment_page = LIMITS.inbox;
                row
            }),
            ("restore_accepts_exact_page_and_stamp_bounds: review page", {
                let mut row = fixture::live();
                row.review_page = LIMITS.inbox;
                row
            }),
            ("restore_accepts_exact_page_and_stamp_bounds: comment stamps", {
                let mut row = fixture::live();
                row.comments = fixture::deliveries(LIMITS.inbox, false);
                row
            }),
            ("restore_accepts_exact_page_and_stamp_bounds: review stamps", {
                let mut row = fixture::live();
                row.reviews = fixture::deliveries(LIMITS.inbox, false);
                row
            }),
            ("restore_accepts_exact_cache_and_echo_row_bounds: item rows", {
                let mut row = fixture::live();
                let mut value = kept::summary();
                value.labels = fixture::repeated(Box::<[u8]>::default(), LIMITS.rows);
                row.cached.item = Some(value);
                row
            }),
            ("restore_accepts_exact_cache_and_echo_row_bounds: pull rows", {
                let mut row = fixture::live();
                let mut value = kept::current();
                value.reviewers = fixture::repeated(7, LIMITS.rows);
                row.cached.pull = Some(value);
                row
            }),
            ("restore_accepts_exact_cache_and_echo_row_bounds: echo rows", {
                let mut row = fixture::live();
                row.echoes = fixture::repeated(Echo::Comment(1), LIMITS.rows);
                row
            }),
            ("restore_accepts_exact_cached_byte_bounds: item bytes", {
                let mut row = fixture::live();
                let mut value = kept::summary();
                value.title = fixture::repeated(b'a', LIMITS.answer_bytes);
                row.cached.item = Some(value);
                row
            }),
            ("restore_accepts_exact_cached_byte_bounds: pull bytes", {
                let mut row = fixture::live();
                let mut value = kept::current();
                value.head = fixture::repeated(b'a', fixture::PULL_HEAD_BOUND);
                row.cached.pull = Some(value);
                row
            }),
        ],
        |row| fixture::accepted_live("saved record", row),
    );
}

fn lost_turn(write: Write) -> writing::Harness {
    let mut h = writing::Harness::new();
    h.make(1, write, Condition::None);
    h.clock();
    let sent = h.one();
    h.answer(sent, Err(Error::Timeout));
    h
}
fn find_dispatch(h: &mut writing::Harness, runs: Box<[api::Run]>, more: bool) {
    let branch = h.one();
    assert_eq!(branch.op, Op::Read(Read::Branch { branch: Box::from(*b"work") }));
    h.answer(branch, Ok(Answer::Commit([1; 32])));
    let listing = h.one();
    assert_eq!(listing.op, Op::Read(Read::Runs { workflow: Box::from(*b"ci"), commit: [1; 32], page: 1 }));
    h.answer(listing, Ok(Answer::Runs { runs, more }));
}
fn dispatch_write() -> Write {
    Write::Dispatch { workflow: Box::from(*b"ci"), branch: Box::from(*b"work"), inputs: Box::new([]) }
}
#[test]
fn a_lost_dispatch_is_found_by_its_run() {
    let mut h = lost_turn(dispatch_write());
    find_dispatch(&mut h, Box::new([super::turn::run()]), false);
    assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Made { made: Made::Set, found: true })));
    assert_eq!(h.writes, 1);
}
#[test]
fn another_actor_an_old_run_or_another_event_cannot_prove_dispatch() {
    let run = super::turn::run();
    let mut other_actor = run.clone();
    other_actor.actor = 2;
    let mut old = run.clone();
    old.created = at(fixture::BEFORE_ATTEMPT);
    let mut pushed = run;
    pushed.event = api::Event::Other;
    for run in [other_actor, old, pushed] {
        let mut h = lost_turn(dispatch_write());
        let deadline = h.store.get(&1).expect("saved attempt").attempt.expect("sent attempt").deadline;
        find_dispatch(&mut h, Box::new([run.clone()]), false);
        assert!(h.send().is_none());
        h.fire(fixture::ATTEMPT_DEADLINE);
        assert_eq!(h.store.get(&1).expect("saved entry").attempt.expect("saved attempt").deadline, deadline);
        find_dispatch(&mut h, Box::new([run]), false);
        assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Held)));
        assert_eq!(h.writes, 1);
    }
}
#[test]
fn a_lost_rerun_is_found_by_a_later_attempt() {
    let expected = api::JobAttempt { head: [1; 32], run: 7, job: 10, attempt: 3 };
    let mut h = lost_turn(Write::Rerun { attempt: expected });
    let sent = h.one();
    assert_eq!(sent.op, Op::Read(Read::Run { run: 7 }));
    let mut run = super::turn::run();
    run.jobs = Box::new([api::RunJob {
        steps: 0,
        id: 10,
        name: Box::from(*b"job"),
        attempt: 4,
        state: api::RunState::Running,
        conclusion: None,
        step: None,
    }]);
    h.answer(sent, Ok(Answer::Run(run)));
    assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Made { made: Made::Set, found: true })));
    assert_eq!(h.writes, 1);
}
#[test]
fn a_rerun_at_the_saved_attempt_is_held_at_the_deadline() {
    let expected = api::JobAttempt { head: [1; 32], run: 7, job: 10, attempt: 3 };
    let mut h = lost_turn(Write::Rerun { attempt: expected });
    let mut run = super::turn::run();
    run.jobs = Box::new([api::RunJob {
        steps: 0,
        id: 10,
        name: Box::new([]),
        attempt: 3,
        state: api::RunState::Completed,
        conclusion: Some(api::Conclusion::Failure),
        step: None,
    }]);
    for deadline in [false, true] {
        if deadline {
            h.fire(fixture::ATTEMPT_DEADLINE);
        }
        let sent = h.one();
        h.answer(sent, Ok(Answer::Run(run.clone())));
    }
    assert_eq!(h.outcomes.as_slice().last(), Some(&(1, Outcome::Held)));
    assert_eq!(h.writes, 1);
}

fn saved_pair(row: crate::LiveRecord) -> Box<[Stored]> {
    Box::new([Stored::Live(row), Stored::Repository(fixture::repository())])
}
