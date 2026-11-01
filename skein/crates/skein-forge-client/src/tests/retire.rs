//! Owner-directed forge retirement (forge-client.md, 7.2).
use super::fixtures::outbox::{comment, pull};
use super::fixtures::{Harness, entry};
use super::{LIMITS, REPO, Sent, at, configured};
use crate::{Attempt, Condition, Entry, Event, Outcome, Position, RecoveryClock, Request, Stored, Writer};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Error, Op, Repository, Write};
use skein_lib::{Duration, List, Queue, Token, Wall};

const OTHER: Repository = Repository { forge: 3, ..REPO };
const OTHER_REPOSITORY: Repository = Repository { repository: 6, ..REPO };
const FAILED: Outcome = Outcome::Failed(Error::NotConnected);

fn ready() -> Harness {
    let mut h = Harness::new(crate::Limits { rate: 100, reserve: 0, ..LIMITS });
    h.domain = configured(&h.env.limits);
    h.step(Event::Restored { clock: RecoveryClock::Monotonic });
    h
}

fn numbered(number: u64, repository: Repository) -> Entry {
    Entry { number, repository, ..entry(comment()) }
}

fn attempted(mut entry: Entry) -> Entry {
    entry.start = Some(Position { at: at(100), comment: 0, review: 0, review_page: 1 });
    entry.attempt = Some(Attempt {
        sent: at(100),
        deadline: at(111),
        wall: Wall::from_nanos(at(100).as_nanos()),
        expires: Wall::from_nanos(at(111).as_nanos()),
    });
    entry
}

fn outcome(number: u64) -> Request {
    Request::Outcome { entry: number, task: 7, outcome: FAILED }
}

fn take(out: &mut Queue<Request>) -> Box<[Request]> {
    let mut requests = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        requests.push(request).expect("measured output fits");
    }
    requests.into_boxed()
}

fn resume(h: &mut Harness) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    crate::resume(&mut h.domain, &h.env, &mut out);
    h.domain.reclaim();
    take(&mut out)
}

fn one(h: &mut Harness) -> (Sent, Repository) {
    let requests = resume(h);
    let Some(Request::Call { call, repository, op }) = requests.last() else {
        panic!("one call is ready: {requests:?}");
    };
    (Sent { call: *call, op: op.clone() }, *repository)
}

fn clock(h: &mut Harness) {
    let (sent, _) = one(h);
    h.answer(&sent, 1, Ok(Answer::Items { items: Box::new([]), more: false, now: at(100) }));
}

fn fire_at(h: &mut Harness, seconds: u64) -> Box<[Request]> {
    h.env.now = at(seconds);
    let mut out = Queue::uncapped();
    crate::fire(&mut h.domain, &h.env, &mut out);
    h.domain.reclaim();
    take(&mut out)
}

#[test]
fn retiring_a_forge_settles_its_unsent_and_uncertain_entries_at_once() {
    let mut h = ready();
    h.step(Event::Make { entry: numbered(1, REPO) });
    clock(&mut h);
    let (write, _) = one(&mut h);
    h.answer(&write, 1, Err(Error::Timeout));
    let (find, _) = one(&mut h);
    h.answer(&find, 1, Err(Error::Unavailable));
    h.step(Event::Make { entry: numbered(2, REPO) });
    h.step(Event::Make { entry: numbered(3, OTHER_REPOSITORY) });
    // Leave an unsent call queued beside the uncertain entry's backoff.
    crate::outbox::pump(&mut h.domain, &h.env, &mut Queue::uncapped());
    let mut queued = attempted(numbered(4, REPO));
    queued.effect.write = Write::Merge { number: 10, head: [1; 32], how: api::MergeHow::Style };
    queued.effect.condition = Condition::Merge { base: Box::from(b"main".as_slice()) };
    queued.queued = true;
    h.step(Event::Make { entry: queued });

    assert_eq!(h.step(Event::Retire { forge: REPO.forge }).as_ref(), &[outcome(1), outcome(2), outcome(3), outcome(4)]);
    assert_eq!(h.domain.next_deadline(), None);
    assert!(resume(&mut h).is_empty());
    assert!(h.step(Event::Retire { forge: REPO.forge }).is_empty());
}

#[test]
fn a_retired_entry_with_a_call_out_settles_at_its_answer_whatever_it_is() {
    for result in [
        Ok(Answer::Commented(42)),
        Ok(Answer::Done), // An answer invalid for this write is still terminal.
        Err(Error::Unavailable),
        Err(Error::Timeout),
        Err(Error::InvalidAnswer),
        Err(Error::RateLimited { after: Duration::from_secs(1) }),
        Err(Error::Forbidden),
        Err(Error::Missing),
        Err(Error::MissingJob),
        Err(Error::TooLarge),
        Err(Error::Empty),
        Err(Error::Full),
        Err(Error::Exists),
        Err(Error::NothingToMerge),
        Err(Error::Closed),
        Err(Error::Stale),
        Err(Error::Conflict),
        Err(Error::Protected),
        Err(Error::Refused),
        Err(Error::NotConnected),
        Err(Error::NotInstalled),
        Err(Error::Busy),
    ] {
        let mut h = ready();
        h.step(Event::Make { entry: numbered(1, REPO) });
        clock(&mut h);
        let (write, _) = one(&mut h);
        assert!(h.step(Event::Retire { forge: REPO.forge }).is_empty());
        assert!(h.step(Event::Retire { forge: REPO.forge }).is_empty());
        assert_eq!(h.answer(&write, 1, result.clone()).as_ref(), &[outcome(1)], "{result:?}");
        assert!(fire_at(&mut h, 200).is_empty());
        assert!(resume(&mut h).is_empty(), "{result:?}");
        assert!(!h.domain.is_ready());
    }
}

#[test]
fn a_retired_forge_has_no_writer_and_refuses_a_make() {
    for restored in [false, true] {
        let mut h = Harness::new(LIMITS);
        h.domain = configured(&h.env.limits);
        if restored {
            h.step(Event::Restored { clock: RecoveryClock::Monotonic });
        }
        assert!(h.step(Event::Retire { forge: REPO.forge }).is_empty(), "no writer Erase");
        if !restored {
            h.step(Event::Restored { clock: RecoveryClock::Monotonic });
        }
        assert_eq!(
            h.step(Event::Make { entry: numbered(1, REPO) }).as_ref(),
            &[Request::Outcome { entry: 1, task: 7, outcome: Outcome::Failed(Error::Forbidden) }]
        );
        assert!(resume(&mut h).is_empty());
    }
}

#[test]
fn retiring_one_forge_leaves_another_forges_entries_and_calls() {
    let mut h = ready();
    crate::writer(&mut h.domain, &h.env, Writer { forge: OTHER.forge, author: 2 }, &mut Queue::uncapped());
    h.step(Event::Make { entry: numbered(1, REPO) });
    let (retiring, _) = one(&mut h);
    h.step(Event::Make { entry: numbered(2, OTHER) });
    clock(&mut h);
    let (other, repository) = one(&mut h);
    h.answer(&other, 1, Err(Error::Unavailable));
    let other_alarm = h.domain.next_deadline();
    h.step(Event::Make { entry: numbered(3, OTHER) });
    let before_budget = h.domain.calls.remaining();
    assert!(h.step(Event::Retire { forge: REPO.forge }).is_empty());
    assert_eq!(h.domain.calls.remaining(), before_budget);
    assert_eq!(h.domain.next_deadline(), other_alarm);
    assert_eq!(repository, OTHER);
    assert_eq!(h.answer(&retiring, 1, Err(Error::Timeout)).as_ref(), &[outcome(1)]);
    fire_at(&mut h, 102);
    let (retry, repository) = one(&mut h);
    assert_eq!(repository, OTHER);
    assert_eq!(retry.op, other.op);
    assert_eq!(
        h.answer(&retry, 1, Ok(Answer::Commented(42))).as_ref(),
        &[Request::Outcome {
            entry: 2,
            task: 7,
            outcome: Outcome::Made { made: crate::Made::Commented(42), found: false }
        }]
    );
    clock(&mut h);
    let (next, repository) = one(&mut h);
    assert_eq!(repository, OTHER);
    assert_eq!(next.op, Op::Write(comment()));
}

#[test]
fn nothing_names_a_retired_entry_afterwards() {
    let mut h = ready();
    h.step(Event::PauseOutbox);
    h.step(Event::Make { entry: attempted(numbered(1, REPO)) });
    assert_eq!(h.step(Event::Retire { forge: REPO.forge }).as_ref(), &[outcome(1)]);
    assert_eq!(h.step(Event::SettleOutbox).as_ref(), &[Request::OutboxDone]);
    assert_eq!(h.step(Event::Settle).as_ref(), &[Request::Settled { left: Box::new([]) }]);
    assert!(fire_at(&mut h, 200).is_empty());
    assert!(resume(&mut h).is_empty());

    // A real backoff alarm is cancelled before its former firing time.
    let mut h = ready();
    h.step(Event::Make { entry: attempted(numbered(1, REPO)) });
    let (find, _) = one(&mut h);
    h.answer(&find, 1, Err(Error::Unavailable));
    assert_eq!(h.step(Event::Retire { forge: REPO.forge }).as_ref(), &[outcome(1)]);
    assert!(fire_at(&mut h, 200).is_empty());
    assert!(resume(&mut h).is_empty());
    assert!(h.step(Event::Withdraw { entry: 1 }).is_empty());
}

#[test]
fn retiring_an_unknown_forge_says_nothing() {
    let mut h = ready();
    h.step(Event::Make { entry: numbered(1, REPO) });
    let (call, _) = one(&mut h);
    let before_budget = h.domain.calls.remaining();
    assert!(h.step(Event::Retire { forge: OTHER.forge }).is_empty());
    assert_eq!(h.domain.calls.remaining(), before_budget);
    assert!(h.answer(&call, 1, Ok(Answer::Items { items: Box::new([]), more: false, now: at(100) })).is_empty());
    let (write, repository) = one(&mut h);
    assert_eq!(repository, REPO);
    assert_eq!(write.op, Op::Write(comment()));
}

#[test]
fn retirement_does_not_wait_for_a_call_to_finish_outbox_settlement_or_stop() {
    let mut h = ready();
    h.step(Event::PauseOutbox);
    h.step(Event::Make { entry: attempted(numbered(1, REPO)) });
    h.step(Event::SettleOutbox);
    let (find, _) = one(&mut h);
    assert_eq!(h.step(Event::Retire { forge: REPO.forge }).as_ref(), &[Request::OutboxDone]);
    assert_eq!(h.step(Event::Settle).as_ref(), &[Request::Settled { left: Box::new([]) }]);
    assert_eq!(h.answer(&find, 1, Err(Error::Timeout)).as_ref(), &[outcome(1)]);
    assert!(fire_at(&mut h, 200).is_empty());
    assert!(resume(&mut h).is_empty());

    let mut h = ready();
    h.step(Event::Make { entry: numbered(1, REPO) });
    clock(&mut h);
    let (write, _) = one(&mut h);
    h.step(Event::Retire { forge: REPO.forge });
    assert_eq!(h.step(Event::Settle).as_ref(), &[Request::Settled { left: Box::new([]) }]);
    assert!(fire_at(&mut h, 200).is_empty());
    assert_eq!(h.answer(&write, 1, Ok(Answer::Commented(42))).as_ref(), &[outcome(1)]);
}

#[test]
fn retirement_waits_for_a_clock_call_and_for_a_proving_read() {
    let mut h = ready();
    h.step(Event::Make { entry: numbered(1, REPO) });
    let (capture, _) = one(&mut h);
    assert!(h.step(Event::Retire { forge: REPO.forge }).is_empty());
    assert_eq!(
        h.answer(&capture, 1, Ok(Answer::Items { items: Box::new([]), more: false, now: at(100) })).as_ref(),
        &[outcome(1)]
    );
    assert!(resume(&mut h).is_empty());

    let mut h = ready();
    let mut merge = numbered(1, REPO);
    merge.effect.write = Write::Merge { number: 9, head: [1; 32], how: api::MergeHow::Style };
    merge.effect.condition = Condition::Merge { base: Box::from(b"main".as_slice()) };
    h.step(Event::Make { entry: merge });
    clock(&mut h);
    let (check, _) = one(&mut h);
    h.answer(&check, 1, Ok(Answer::Pull(pull([1; 32]))));
    let (write, _) = one(&mut h);
    h.answer(&write, 1, Ok(Answer::Merged([4; 32])));
    let (proof, _) = one(&mut h);
    h.step(Event::Retire { forge: REPO.forge });
    assert_eq!(h.answer(&proof, 1, Ok(Answer::Pull(pull([1; 32])))).as_ref(), &[outcome(1)]);
    assert!(resume(&mut h).is_empty());
}

#[test]
fn retirement_leaves_live_records_until_the_owner_drops_their_watches() {
    let mut h = Harness::new(LIMITS);
    let live = super::fixtures::recovery::live();
    h.domain = configured(&h.env.limits);
    h.step(Event::Restore { record: Stored::Repository(super::fixtures::recovery::repository()) });
    h.step(Event::Restore { record: Stored::Live(live.clone()) });
    h.step(Event::Restored { clock: RecoveryClock::Monotonic });
    assert!(h.step(Event::Retire { forge: REPO.forge }).is_empty());
    assert!(h.domain.cached(&live.watch.resource).is_some());
    assert_eq!(
        h.step(Event::Keep { owner: Token::new(1), watches: Box::new([]) }).as_ref(),
        &[
            Request::Erase { key: crate::Key::Live(live.watch.resource) },
            Request::Erase { key: crate::Key::Repository(REPO) },
            Request::Kept { owner: Token::new(1), result: Ok(()) },
        ]
    );
}

#[test]
fn retirement_preserves_other_forges_outstanding_and_queued_calls() {
    let mut h = ready();
    crate::writer(&mut h.domain, &h.env, Writer { forge: OTHER.forge, author: 2 }, &mut Queue::uncapped());
    h.step(Event::Make { entry: numbered(1, REPO) });
    let (retiring, _) = one(&mut h);
    h.step(Event::Make { entry: numbered(2, OTHER) });
    let (other_clock, _) = one(&mut h);
    let other_repository = Repository { repository: OTHER_REPOSITORY.repository, ..OTHER };
    h.step(Event::Make { entry: numbered(3, other_repository) });
    crate::outbox::pump(&mut h.domain, &h.env, &mut Queue::uncapped());

    assert!(h.step(Event::Retire { forge: REPO.forge }).is_empty());
    assert!(h.answer(&other_clock, 1, Ok(Answer::Items { items: Box::new([]), more: false, now: at(100) })).is_empty());
    let (queued_clock, repository) = one(&mut h);
    assert_eq!(repository, other_repository);
    let (other_write, repository) = one(&mut h);
    assert_eq!(repository, OTHER);
    assert_eq!(other_write.op, Op::Write(comment()));
    assert_eq!(
        h.answer(&other_write, 1, Ok(Answer::Commented(42))).as_ref(),
        &[Request::Outcome {
            entry: 2,
            task: 7,
            outcome: Outcome::Made { made: crate::Made::Commented(42), found: false }
        }]
    );
    assert!(
        h.answer(&queued_clock, 1, Ok(Answer::Items { items: Box::new([]), more: false, now: at(100) })).is_empty()
    );
    let (queued_write, repository) = one(&mut h);
    assert_eq!(repository, other_repository);
    assert_eq!(queued_write.op, Op::Write(comment()));
    assert_eq!(h.answer(&retiring, 1, Err(Error::Unavailable)).as_ref(), &[outcome(1)]);
}

#[test]
fn retirement_cancels_a_queued_write_and_frees_its_lane() {
    let mut h = ready();
    h.step(Event::Make { entry: numbered(1, REPO) });
    clock(&mut h);
    crate::outbox::pump(&mut h.domain, &h.env, &mut Queue::uncapped());
    assert_eq!(h.step(Event::Retire { forge: REPO.forge }).as_ref(), &[outcome(1)]);
    assert!(resume(&mut h).is_empty());
    crate::writer(&mut h.domain, &h.env, Writer { forge: REPO.forge, author: 1 }, &mut Queue::uncapped());
    h.step(Event::Make { entry: numbered(2, REPO) });
    clock(&mut h);
    let (write, repository) = one(&mut h);
    assert_eq!(repository, REPO);
    assert_eq!(write.op, Op::Write(comment()));
}

#[test]
fn a_retired_lane_with_a_call_out_frees_at_its_terminal() {
    let mut h = ready();
    h.step(Event::Make { entry: numbered(1, REPO) });
    let (capture, _) = one(&mut h);
    h.step(Event::Retire { forge: REPO.forge });
    crate::writer(&mut h.domain, &h.env, Writer { forge: REPO.forge, author: 1 }, &mut Queue::uncapped());
    h.step(Event::Make { entry: numbered(2, REPO) });
    assert!(resume(&mut h).is_empty());
    assert_eq!(h.answer(&capture, 1, Err(Error::Unavailable)).as_ref(), &[outcome(1)]);
    clock(&mut h);
    let (write, repository) = one(&mut h);
    assert_eq!(repository, REPO);
    assert_eq!(write.op, Op::Write(comment()));
}
