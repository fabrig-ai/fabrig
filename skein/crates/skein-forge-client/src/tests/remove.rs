//! Immediate forge removal and late terminals (forge-client.md, 5–8;
//! temper/docs/design/draft/playground-reset.md, 4).
use super::fixtures::keep::{current, pull, summary, watch};
use super::fixtures::{Harness, at, entry};
use super::{LIMITS, REPO};
use crate::{
    Config, Domain, Event, Key, Limits, Made, Outcome, RecoveryClock, Request, Resource, Stored, Writer, remove_forge,
};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Error, Op, Read, Repository, Write};
use skein_lib::{List, Queue, Token};

const OTHER: Repository = Repository { forge: 3, ..REPO };
const WRITER: Writer = Writer { forge: REPO.forge, author: 1 };
const OTHER_WRITER: Writer = Writer { forge: OTHER.forge, author: 2 };
// Two live rows, their two repository scans, and the keep terminal.
const TWO_WATCH_REQUESTS: usize = 5;
// The writer erasure and one terminal for each read in the story.
const THREE_READ_REMOVAL_REQUESTS: usize = 4;
const FIVE_READ_REMOVAL_REQUESTS: usize = 6;

fn client() -> Harness {
    let mut h = Harness::new(Limits { rate: 100, reserve: 0, ..LIMITS });
    h.domain = Domain::configured(
        &h.env.limits,
        0,
        Config { namespace: Box::from(&b"kit"[..]), writers: Box::new([WRITER, OTHER_WRITER]) },
    )
    .expect("two configured forges");
    assert!(h.step(Event::Restored { clock: RecoveryClock::Monotonic }).is_empty());
    h
}

fn take(out: &mut Queue<Request>) -> Box<[Request]> {
    let mut requests = List::with_capacity(out.len());
    while let Some(request) = out.pop() {
        requests.push(request).expect("measured requests fit");
    }
    requests.into_boxed()
}

fn remove(h: &mut Harness, forge: u16) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    remove_forge(&mut h.domain, &h.env, forge, &mut out);
    take(&mut out)
}

fn sent(h: &mut Harness, repository: Repository) -> (Token, Op) {
    let mut out = Queue::uncapped();
    crate::resume(&mut h.domain, &h.env, &mut out);
    let mut sent = None;
    while let Some(request) = out.pop() {
        match request {
            Request::Call { call, repository: actual, op } => {
                assert_eq!(actual, repository);
                assert!(sent.replace((call, op)).is_none(), "one call per resume");
            }
            Request::Progress { entry } => assert_eq!(entry.repository, repository),
            Request::Read { .. }
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Outcome { .. }
            | Request::Kept { .. }
            | Request::Changed { .. }
            | Request::Drift { .. }
            | Request::ReadAfreshDone
            | Request::Settled { .. }
            | Request::OutboxDone => panic!("unexpected send output: {request:?}"),
        }
    }
    sent.expect("call ready")
}

fn answer(h: &mut Harness, call: Token, result: Result<Answer, Error>) -> Box<[Request]> {
    h.step(Event::Answered { call, cost: 1, free: 0, rate: None, result })
}

fn clock(h: &mut Harness, repository: Repository) {
    let (call, op) = sent(h, repository);
    assert_eq!(op, Op::Read(Read::Items { since: at(u64::MAX), page: 1, kind: None }));
    drop(answer(h, call, Ok(Answer::Items { items: Box::new([]), more: false, now: at(100) })));
}

fn read(h: &mut Harness, owner: u64, repository: Repository) {
    assert!(
        h.step(Event::Read { owner: Token::new(owner), repository, read: Read::Pull { number: owner } }).is_empty()
    );
}

fn flight(h: &mut Harness, owner: u64, repository: Repository) -> Token {
    read(h, owner, repository);
    sent(h, repository).0
}

fn keep_two(h: &mut Harness) -> (Resource, Resource) {
    let resource = pull();
    let other = Resource { repository: OTHER, ..resource.clone() };
    let kept = h.step(Event::Keep {
        owner: Token::new(1),
        watches: Box::new([watch(resource.clone(), false), watch(other.clone(), false)]),
    });
    assert_eq!(kept.len(), TWO_WATCH_REQUESTS);
    (resource, other)
}

#[test]
fn removing_a_forge_erases_its_writer_and_a_new_one_saves_again() {
    let mut h = client();
    assert_eq!(remove(&mut h, REPO.forge).as_ref(), &[Request::Erase { key: Key::Writer(REPO.forge) }]);
    assert!(!h.domain.writers.contains_key(&REPO.forge));
    assert_eq!(h.domain.writers.get(&OTHER.forge), Some(&OTHER_WRITER.author));
    let mut out = Queue::uncapped();
    let replacement = Writer { author: 4, ..WRITER };
    assert!(crate::writer(&mut h.domain, &h.env, replacement, &mut out));
    assert_eq!(take(&mut out).as_ref(), &[Request::Save { record: Stored::Writer(replacement) }]);
    assert!(crate::writer(&mut h.domain, &h.env, OTHER_WRITER, &mut out));
    assert!(out.is_empty());
}

#[test]
fn removing_a_forge_drops_its_entries_and_keeps_the_others() {
    let mut h = client();
    for (number, repository) in [(1, REPO), (2, REPO), (3, OTHER), (4, OTHER)] {
        let mut entry = entry(Write::Close { number: 9 });
        entry.number = number;
        entry.repository = repository;
        assert!(h.step(Event::Make { entry }).is_empty());
    }
    clock(&mut h, REPO);
    let (forgotten, op) = sent(&mut h, REPO);
    assert_eq!(op, Op::Write(Write::Close { number: 9 }));
    clock(&mut h, OTHER);
    let (kept, op) = sent(&mut h, OTHER);
    assert_eq!(op, Op::Write(Write::Close { number: 9 }));
    let budget = h.domain.calls.remaining();
    assert_eq!(remove(&mut h, REPO.forge).as_ref(), &[Request::Erase { key: Key::Writer(REPO.forge) }]);
    assert_eq!(h.domain.calls.remaining(), budget, "sent calls stay charged");
    assert!(answer(&mut h, forgotten, Ok(Answer::Done)).is_empty());
    for number in [1, 2] {
        assert!(h.step(Event::Withdraw { entry: number }).is_empty(), "removed entries are unknown");
    }
    assert_eq!(
        answer(&mut h, kept, Ok(Answer::Done)).as_ref(),
        &[Request::Outcome { entry: 3, task: 7, outcome: Outcome::Made { made: Made::Set, found: false } }]
    );
    clock(&mut h, OTHER);
    let (kept, op) = sent(&mut h, OTHER);
    assert_eq!(op, Op::Write(Write::Close { number: 9 }));
    assert_eq!(
        answer(&mut h, kept, Ok(Answer::Done)).as_ref(),
        &[Request::Outcome { entry: 4, task: 7, outcome: Outcome::Made { made: Made::Set, found: false } }]
    );
    assert!(!h.domain.is_ready());
    assert_eq!(h.domain.next_deadline(), None);
}

#[test]
fn a_read_on_a_removed_forge_gets_its_one_terminal() {
    let mut h = client();
    let retry = flight(&mut h, 1, REPO);
    assert!(answer(&mut h, retry, Err(Error::Timeout)).is_empty());
    let outstanding = flight(&mut h, 2, REPO);
    read(&mut h, 3, REPO);
    read(&mut h, 4, OTHER);
    let budget = h.domain.calls.remaining();
    let removed = remove(&mut h, REPO.forge);
    assert_eq!(removed.len(), THREE_READ_REMOVAL_REQUESTS);
    assert_eq!(removed.first(), Some(&Request::Erase { key: Key::Writer(REPO.forge) }));
    for owner in [1, 2, 3] {
        assert!(removed.contains(&Request::Read { owner: Token::new(owner), result: Err(Error::NotConnected) }));
    }
    assert_eq!(h.domain.calls.remaining(), budget);
    // Repeated removal before reclaim cannot retire a call twice.
    assert!(remove(&mut h, REPO.forge).is_empty());
    h.domain.reclaim();
    assert!(answer(&mut h, outstanding, Ok(Answer::Pull(current()))).is_empty());
    let (kept, _) = sent(&mut h, OTHER);
    assert_eq!(
        answer(&mut h, kept, Err(Error::Forbidden)).as_ref(),
        &[Request::Read { owner: Token::new(4), result: Err(Error::Forbidden) }]
    );
    h.env.now = at(200);
    h.fire();
    assert_eq!(h.domain.next_deadline(), None, "the removed read's retry alarm is gone");
    assert!(!h.domain.is_ready());
    assert!(h.domain.has_room());
}

#[test]
fn a_late_answer_for_a_removed_forges_flight_is_ignored() {
    let mut h = client();
    let forgotten = flight(&mut h, 1, REPO);
    drop(remove(&mut h, REPO.forge));
    h.domain.reclaim();
    let kept = flight(&mut h, 2, OTHER);
    assert_ne!(forgotten, kept);
    let budget = h.domain.calls.remaining();
    assert!(
        h.step(Event::Answered {
            call: forgotten,
            cost: h.env.limits.rate,
            free: 0,
            rate: Some(h.rate(0, 60)),
            result: Err(Error::RateLimited { after: h.env.limits.window }),
        })
        .is_empty()
    );
    assert_eq!(h.domain.calls.remaining(), budget);
    assert_eq!(h.domain.next_deadline(), None);
    assert_eq!(
        answer(&mut h, kept, Err(Error::Forbidden)).as_ref(),
        &[Request::Read { owner: Token::new(2), result: Err(Error::Forbidden) }]
    );
    assert!(answer(&mut h, forgotten, Ok(Answer::Done)).is_empty());
}

#[test]
#[should_panic(expected = "terminal names an issued call token")]
fn a_never_issued_token_still_panics() {
    let mut h = client();
    let issued = flight(&mut h, 1, REPO);
    drop(remove(&mut h, REPO.forge));
    let never = Token::new(issued.raw().checked_add(1).expect("test sequence fits"));
    drop(answer(&mut h, never, Ok(Answer::Done)));
}

#[test]
#[should_panic(expected = "terminal names an issued call token")]
fn token_zero_was_never_issued() {
    let mut h = client();
    drop(answer(&mut h, Token::new(0), Ok(Answer::Done)));
}

#[test]
fn removing_a_forge_erases_its_watch_rows_and_alarms() {
    let mut h = client();
    let (resource, other) = keep_two(&mut h);
    for repository in [REPO, OTHER] {
        clock(&mut h, repository);
    }
    for repository in [REPO, OTHER] {
        let (call, op) = sent(&mut h, repository);
        assert_eq!(op, Op::Read(Read::Item { number: 9, after: u64::MAX }));
        drop(answer(&mut h, call, Ok(Answer::Item { item: summary(), comments: Box::new([]), more: false })));
        let (call, op) = sent(&mut h, repository);
        assert_eq!(op, Op::Read(Read::Pull { number: 9 }));
        drop(answer(&mut h, call, Ok(Answer::Pull(current()))));
    }
    assert!(h.domain.next_deadline().is_some());
    assert_eq!(
        remove(&mut h, REPO.forge).as_ref(),
        &[
            Request::Erase { key: Key::Writer(REPO.forge) },
            Request::Erase { key: Key::Live(resource.clone()) },
            Request::Erase { key: Key::Repository(REPO) },
        ]
    );
    assert!(h.domain.cached(&resource).is_none());
    assert_eq!(h.domain.cached(&other).expect("other watch remains").pull, Some(current()));
    assert!(
        h.step(Event::Hint { hint: api::Hint { repository: REPO, change: api::Change::Item(9), key: None } })
            .is_empty()
    );
    h.env.now = at(200);
    while h.domain.is_due(h.env.now) {
        h.fire();
    }
    let (call, _) = sent(&mut h, OTHER);
    drop(answer(&mut h, call, Ok(Answer::Items { items: Box::new([]), more: false, now: at(200) })));
    let (call, _) = sent(&mut h, OTHER);
    drop(answer(&mut h, call, Err(Error::Missing)));
    assert!(!h.domain.is_ready(), "only the other forge's due work ran");
}

#[test]
fn removing_an_unknown_forge_changes_nothing() {
    let mut h = client();
    let call = flight(&mut h, 1, REPO);
    let budget = h.domain.calls.remaining();
    let deadline = h.domain.next_deadline();
    assert!(remove(&mut h, 99).is_empty());
    assert_eq!(h.domain.calls.remaining(), budget);
    assert_eq!(h.domain.next_deadline(), deadline);
    assert_eq!(h.domain.writers.get(&REPO.forge), Some(&WRITER.author));
    assert_eq!(h.domain.writers.get(&OTHER.forge), Some(&OTHER_WRITER.author));
    assert_eq!(
        answer(&mut h, call, Err(Error::Forbidden)).as_ref(),
        &[Request::Read { owner: Token::new(1), result: Err(Error::Forbidden) }]
    );
}

#[test]
fn removal_frees_every_priority_queue_and_preserves_the_shared_rate_reset() {
    let mut h = client();
    let call = flight(&mut h, 1, REPO);
    assert!(
        h.step(Event::Answered {
            call,
            cost: 1,
            free: 0,
            rate: Some(h.rate(0, 60)),
            result: Err(Error::RateLimited { after: h.env.limits.window }),
        })
        .is_empty()
    );
    let deadline = h.domain.next_deadline();
    for (owner, priority) in [
        (2, crate::Priority::Fresh),
        (3, crate::Priority::Write),
        (4, crate::Priority::Keep),
        (5, crate::Priority::Slow),
    ] {
        crate::calls::queue(
            &mut h.domain.calls,
            crate::calls::Owner::Read(Token::new(owner)),
            REPO,
            Op::Read(Read::Settings),
            priority,
        );
    }
    read(&mut h, 6, OTHER);
    let budget = h.domain.calls.remaining();
    let removed = remove(&mut h, REPO.forge);
    assert_eq!(removed.len(), FIVE_READ_REMOVAL_REQUESTS);
    for owner in 1..=5 {
        assert!(removed.contains(&Request::Read { owner: Token::new(owner), result: Err(Error::NotConnected) }));
    }
    h.domain.reclaim();
    assert_eq!(h.domain.calls.remaining(), budget);
    assert_eq!(h.domain.next_deadline(), deadline);
    assert!(!h.domain.is_ready(), "removal preserves the client-wide rate hold");
    h.env.now = at(160);
    h.fire();
    let (call, op) = sent(&mut h, OTHER);
    assert_eq!(op, Op::Read(Read::Pull { number: 6 }));
    drop(answer(&mut h, call, Err(Error::Forbidden)));
    assert!(!h.domain.is_ready());
}

#[test]
fn removal_forgets_busy_watch_and_scan_flights() {
    let mut h = client();
    let (resource, other) = keep_two(&mut h);
    let (scan, _) = sent(&mut h, REPO);
    let (kept, _) = sent(&mut h, OTHER);
    let (live, _) = sent(&mut h, REPO);
    assert_eq!(
        remove(&mut h, REPO.forge).as_ref(),
        &[
            Request::Erase { key: Key::Writer(REPO.forge) },
            Request::Erase { key: Key::Live(resource.clone()) },
            Request::Erase { key: Key::Repository(REPO) },
        ]
    );
    let after = h.env.limits.window;
    assert!(answer(&mut h, scan, Err(Error::RateLimited { after })).is_empty());
    assert!(answer(&mut h, live, Err(Error::Missing)).is_empty());
    assert!(h.domain.cached(&resource).is_none());
    assert!(h.domain.cached(&other).is_some());
    assert_eq!(
        answer(&mut h, kept, Ok(Answer::Items { items: Box::new([]), more: false, now: at(100) })).as_ref(),
        &[Request::Save {
            record: Stored::Repository(crate::RepositoryRecord {
                repository: OTHER,
                mark: Some(at(100)),
                clock: at(100)
            }),
        }]
    );
    let (call, _) = sent(&mut h, OTHER);
    drop(answer(&mut h, call, Err(Error::Missing)));
    h.env.now = at(200);
    while h.domain.is_due(h.env.now) {
        h.fire();
    }
    drop(sent(&mut h, OTHER));
}

#[test]
fn removal_completes_the_fresh_pass_when_its_last_watch_leaves() {
    let mut h = client();
    let resource = pull();
    drop(h.step(Event::Keep { owner: Token::new(1), watches: Box::new([watch(resource.clone(), false)]) }));
    assert!(h.step(Event::ReadAfresh).is_empty());
    assert_eq!(
        remove(&mut h, REPO.forge).as_ref(),
        &[
            Request::Erase { key: Key::Writer(REPO.forge) },
            Request::Erase { key: Key::Live(resource) },
            Request::Erase { key: Key::Repository(REPO) },
            Request::ReadAfreshDone,
        ]
    );
    assert!(remove(&mut h, REPO.forge).is_empty());
    assert!(!h.domain.is_ready());
}

#[test]
fn only_removing_a_known_forge_advances_settling() {
    let mut h = client();
    assert!(h.step(Event::Make { entry: entry(Write::Close { number: 9 }) }).is_empty());
    clock(&mut h, REPO);
    let (call, _) = sent(&mut h, REPO);
    assert!(h.step(Event::Settle).is_empty());
    h.env.now = at(200);
    assert!(remove(&mut h, 99).is_empty(), "an unknown forge changes nothing even past the stop deadline");
    assert_eq!(
        remove(&mut h, REPO.forge).as_ref(),
        &[Request::Erase { key: Key::Writer(REPO.forge) }, Request::Settled { left: Box::new([]) },]
    );
    assert!(answer(&mut h, call, Ok(Answer::Done)).is_empty());
    assert_eq!(h.domain.next_deadline(), None);
}
