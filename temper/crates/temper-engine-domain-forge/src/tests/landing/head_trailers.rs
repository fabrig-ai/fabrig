//! The round reads the head's trailers from its containment range (domain/landing-prepare.md, 1.2).
use super::*;
use skein_forge::api::{Answer, Bound, CommitMessage, Error, Excess, Op, Read};

const TASK: u64 = 61;
const OTHER: [u8; 32] = [8; 32];
const TRAILERS: &[u8] = b"Look-request: usage-model-week\nLook-landed: usage-model-week";

fn start_range(d: &mut Domain, clock: &Env<Limits>) {
    let mut out = Queue::uncapped();
    step(
        d,
        clock,
        Event::StepLanding { task: TASK, entry: 91, heard: preparing_heard(Some(policy::Preparation::Absent)) },
        &mut out,
    );
    read_preparing_member(d);
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"temper/1/l61".as_slice().into() }));
    fixtures::answer(d, call, Err(Error::Missing));
}

fn range_page(d: &mut Domain, page: u32) -> Token {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page }));
    call
}

fn page(d: &mut Domain, number: u32, commits: Box<[CommitMessage]>, more: bool) -> Box<[Request]> {
    let call = range_page(d, number);
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
    let out = page(&mut d, 1, Box::new([earlier(), head()]), false);
    assert_prepare(&d, &out, TRAILERS);
}

#[test]
fn the_round_reads_the_next_page_of_the_same_range_for_the_head() {
    let mut d = preparing_landing();
    start_range(&mut d, &env());
    let out = page(&mut d, 1, Box::new([earlier()]), true);
    for request in out {
        assert!(!matches!(request, Request::LandingDecision { .. }), "the round still needs page 2");
    }
    let out = page(&mut d, 2, Box::new([head()]), false);
    assert_prepare(&d, &out, TRAILERS);
    assert_eq!(d.landing(TASK).unwrap().landing.members[0].landed, None);
}

#[test]
fn a_refused_second_page_waits_read_and_is_read_again() {
    let mut d = preparing_landing();
    let clock = env();
    start_range(&mut d, &clock);
    page(&mut d, 1, Box::new([earlier()]), true);
    let call = range_page(&mut d, 2);
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
    page(&mut d, 1, Box::new([earlier()]), true);
    let out = page(&mut d, 2, Box::new([head()]), false);
    assert_prepare(&d, &out, TRAILERS);
}

#[test]
fn a_page_too_large_leaves_the_head_trailers_empty() {
    let mut d = preparing_landing();
    start_range(&mut d, &env());
    page(&mut d, 1, Box::new([earlier()]), true);
    let call = range_page(&mut d, 2);
    let out = fixtures::answer(&mut d, call, Err(Error::TooLarge(Excess::at_least(Bound::AnswerBytes, 40, 41))));
    assert_prepare(&d, &out, b"");
}

#[test]
fn a_range_ending_without_the_head_keeps_no_trailers_and_is_not_contained() {
    let mut d = preparing_landing();
    start_range(&mut d, &env());
    page(&mut d, 1, Box::new([earlier()]), true);
    let out = page(&mut d, 2, Box::new([]), false);
    assert_prepare(&d, &out, b"");
    assert_eq!(d.landing(TASK).unwrap().landing.members[0].landed, None, "only page 1 judges containment");
}

#[test]
fn an_empty_range_on_the_first_page_tells_the_member_it_landed() {
    let mut d = preparing_landing();
    start_range(&mut d, &env());
    let out = page(&mut d, 1, Box::new([]), false);
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
