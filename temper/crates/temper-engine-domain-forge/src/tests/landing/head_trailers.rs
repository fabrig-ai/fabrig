//! The round reads the head's trailers from its containment range (domain/landing-prepare.md, 1.2).
use super::*;
use skein_forge::api::{Answer, Bound, CommitMessage, Error, Excess, Op, Read};

const TASK: u64 = 61;
const OTHER: [u8; 32] = [8; 32];
const TRAILERS: &[u8] = b"Look-request: usage-model-week\nLook-landed: usage-model-week";

fn start_round(d: &mut Domain, clock: &Env<Limits>) {
    let mut out = Queue::uncapped();
    step(
        d,
        clock,
        Event::StepLanding { task: TASK, entry: 91, heard: preparing_heard(Some(policy::Preparation::Absent)) },
        &mut out,
    );
}

fn start_range(d: &mut Domain, clock: &Env<Limits>) {
    start_round(d, clock);
    read_preparing_member(d);
    absent_prefix(d, b"temper/1/l61");
}

fn absent_prefix(d: &mut Domain, branch: &[u8]) {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: branch.into() }));
    fixtures::answer(d, call, Err(Error::Missing));
}

fn range_page(d: &mut Domain) -> Token {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
    call
}

fn page(d: &mut Domain, commits: Box<[CommitMessage]>, more: bool) -> Box<[Request]> {
    let call = range_page(d);
    fixtures::answer(d, call, Ok(Answer::Commits { commits, more }))
}

fn head() -> CommitMessage {
    CommitMessage {
        id: fixtures::HEAD,
        parents: Box::new([OTHER]),
        message: b"Keep the look request\n\nLook-request: usage-model-week\nLook-landed: usage-model-week"
            .as_slice()
            .into(),
    }
}

fn earlier() -> CommitMessage {
    CommitMessage {
        id: OTHER,
        parents: Box::new([fixtures::BASE]),
        message: b"An earlier commit\n\nLook-request: not-the-head".as_slice().into(),
    }
}

fn assert_prepare(d: &Domain, out: &[Request], trailers: &[u8]) {
    let mut prepared = false;
    for request in out {
        if let Request::LandingDecision {
            task: TASK,
            decision: policy::Decision::Prepare { member: 0, head: fixtures::HEAD, main: fixtures::BASE },
            entry: None,
            head_trailers,
            ..
        } = request
        {
            if trailers.is_empty() {
                assert!(head_trailers.is_empty());
            } else {
                assert_eq!(head_trailers.as_ref(), &[LandingHeadTrailers { member: 0, block: trailers.into() }]);
            }
            prepared = true;
        }
    }
    assert!(prepared, "expected Prepare with the head's trailers: {out:?}");
    assert!(d.landing.pending.is_empty(), "no further range read");
}

#[test]
fn the_head_on_the_first_page_needs_no_more_reads() {
    let mut d = preparing_landing();
    start_range(&mut d, &env());
    let out = page(&mut d, Box::new([earlier(), head()]), false);
    assert_prepare(&d, &out, TRAILERS);
}

#[test]
fn the_round_reads_the_next_member_instead_of_the_next_containment_page() {
    let mut d = preparing_landing();
    let mut waiter = d.changes.get(&fixtures::CHANGE).expect("ready change").clone();
    waiter.task = TRAIN_WAITER;
    waiter.change.task = TRAIN_WAITER;
    waiter.branch = b"temper/1/c62".as_slice().into();
    waiter.pull = Some(3);
    outputs(&mut d, Event::Change { row: waiter });
    assert!(d.changes.contains_key(&TRAIN_WAITER));
    let mut row = d.landing(TASK).expect("round").clone();
    let first = row.landing.members[0].clone();
    let second = policy::Member {
        change: TRAIN_WAITER,
        branch: b"temper/1/l61-2".as_slice().into(),
        pull: 3,
        unit: 1,
        ..first.clone()
    };
    row.landing.members = Box::new([first, second]);
    restore_landing(&mut d, row);

    start_round(&mut d, &env());
    read_preparing_member(&mut d);
    // The second member gathers on the same base, whose tip was already read.
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(&mut d, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    let mut pull = fixtures::open_pull();
    pull.number = 3;
    fixtures::answer_next(&mut d, Ok(Answer::Pull(pull)));
    fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
    absent_prefix(&mut d, b"temper/1/l61");
    page(&mut d, Box::new([earlier()]), true);
    absent_prefix(&mut d, b"temper/1/l61-2");
    let out = page(&mut d, Box::new([earlier()]), false);
    assert_prepare(&d, &out, b"");
    assert_eq!(d.landing(TASK).unwrap().landing.members[0].landed, None);
}

#[test]
fn a_refused_first_page_waits_read_and_is_read_again() {
    let mut d = preparing_landing();
    let clock = env();
    start_range(&mut d, &clock);
    let call = range_page(&mut d);
    let out = fixtures::answer(&mut d, call, Err(Error::Refused));
    let mut deadline = None;
    for request in out {
        if let Request::LandingDecision {
            task: TASK,
            decision: policy::Decision::Wait { until: Some(until), on: policy::Awaited::Read },
            head_trailers,
            ..
        } = request
        {
            assert!(head_trailers.is_empty());
            deadline = Some(until);
        }
    }
    let until = deadline.expect("a refused page waits read at a deadline");
    assert!(until > clock.wall);
    d.wait_landing(&clock, TASK, Some(until));
    let due = outbox_room::later(&clock, Duration::from_nanos(until.as_nanos().saturating_sub(clock.wall.as_nanos())));
    assert!(fire_at(&mut d, &due, until).contains(&Request::WakeProcedure { task: TASK }));

    start_range(&mut d, &due);
    let out = page(&mut d, Box::new([head()]), false);
    assert_prepare(&d, &out, TRAILERS);
}

#[test]
fn a_page_too_large_leaves_the_head_trailers_empty() {
    let mut d = preparing_landing();
    start_range(&mut d, &env());
    let call = range_page(&mut d);
    let out = fixtures::answer(&mut d, call, Err(Error::TooLarge(Excess::at_least(Bound::AnswerBytes, 40, 41))));
    assert_prepare(&d, &out, b"");
}

#[test]
fn a_range_ending_without_the_head_keeps_no_trailers_and_is_not_contained() {
    let mut d = preparing_landing();
    start_range(&mut d, &env());
    let out = page(&mut d, Box::new([earlier()]), false);
    assert_prepare(&d, &out, b"");
    assert_eq!(d.landing(TASK).unwrap().landing.members[0].landed, None, "only page 1 judges containment");
}

#[test]
fn an_empty_range_on_the_first_page_tells_the_member_it_landed() {
    let mut d = preparing_landing();
    start_range(&mut d, &env());
    let out = page(&mut d, Box::new([]), false);
    let mut landed = false;
    for request in &out {
        if let Request::LandingDecision {
            task: TASK,
            decision: policy::Decision::Landed { member: 0, prefix: fixtures::BASE },
            head_trailers,
            ..
        } = request
        {
            assert!(head_trailers.is_empty());
            landed = true;
        }
    }
    assert!(landed, "expected the member to be told it already landed: {out:?}");
    assert_eq!(d.landing(TASK).unwrap().landing.members[0].landed, Some(fixtures::BASE));
}
