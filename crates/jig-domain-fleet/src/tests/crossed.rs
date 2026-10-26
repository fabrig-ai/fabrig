//! Crossed restart claims stop before their run can write again.
use super::*;

// The fixture's engine grace is ten seconds (LIMITS.grace).
const GRACE: u64 = 10;
const BEFORE_GRACE: u64 = GRACE - 1;
const DISCONNECT: u64 = 4;
const WORKSTREAM: u64 = 1;

fn cross(h: &mut Harness, run: Token, attempt: Token) -> Box<[Request]> {
    h.step(Event::Cross { reply_to: to(attempt), run, attempt, worked: false })
}

fn crossed(worked: bool) -> Harness {
    let mut h = Harness::new(Limits { turns: 2, ..LIMITS });
    h.step(Event::Cross { reply_to: to(A1), run: R1, attempt: A1, worked });
    h
}

fn listed_crossed() -> Harness {
    let mut h = crossed(false);
    h.hello(C1, 2, &[], &[hosted(R1, A1, Phase::Active)]);
    h
}

#[test]
fn a_crossed_claim_listed_at_hello_is_cancelled_and_lost_once_its_run_answers() {
    for answer in [Answer::Ended, Answer::Parked, FAILED, Answer::Busy, Answer::Invalid] {
        let mut h = crossed(false);
        assert_eq!(&*h.hello(C2, 2, &[], &[hosted(R1, A1, Phase::Active)]), &[cancel(C2, R1, A1)]);
        assert_eq!(&*h.answer(C2, R1, A1, answer, 1), &[ack(C2, R1, A1), drop(1), lost(R1, A1)]);
        assert_eq!(&*h.answer(C2, R1, A1, answer, 2), &[ack(C2, R1, A1), drop(2)]);
        assert!(h.at(GRACE).is_empty());
    }
}

#[test]
fn a_crossed_unworked_claim_no_worker_lists_is_not_started_at_its_grace() {
    let mut h = crossed(false);
    assert!(h.at(BEFORE_GRACE).is_empty());
    assert_eq!(&*h.at(GRACE), &[Request::NotStarted { to: to(A1), run: R1, attempt: A1 }]);
    assert!(h.at(GRACE + 1).is_empty());
}

#[test]
fn a_crossed_stray_listed_before_loading_is_cancelled_on_its_worker() {
    let mut h = Harness::new(LIMITS);
    h.hello(C2, 2, &[], &[hosted(R1, A1, Phase::Active)]);
    assert_eq!(&*cross(&mut h, R1, A1), &[cancel(C2, R1, A1)]);
    h.loaded();
    assert!(h.at(GRACE).is_empty(), "a listed run must answer or lose contact");
    assert_eq!(&*h.answer(C2, R1, A1, FAILED, 1), &[ack(C2, R1, A1), drop(1), lost(R1, A1)]);
}

#[test]
fn a_crossed_claims_kept_answer_is_forgotten_and_the_claim_lost() {
    let mut h = Harness::new(LIMITS);
    h.hello(C2, 2, &[], &[hosted(R1, A1, Phase::Answered)]);
    h.answer(C2, R1, A1, Answer::Ended, 1);
    assert_eq!(&*cross(&mut h, R1, A1), &[drop(1), ack(C2, R1, A1), lost(R1, A1)]);
    assert_eq!(h.domain.attempts(), 0);
}

#[test]
fn a_crossed_claims_turn_is_acknowledged_and_dropped() {
    let mut h = listed_crossed();
    assert_eq!(&*turn(&mut h, C1, 1, 1), &[turn_ack(C1, 1), drop(1)]);
    assert_eq!(h.domain.attempts(), 1, "a turn does not release the run");
}

#[test]
fn a_crossed_claim_holds_its_runs_next_attempt_back_until_lost() {
    let mut h = crossed(false);
    h.hello(C1, 2, &[], &[]);
    assert_eq!(&*h.start(R1, A2, WORKSTREAM), &[withdrawn(R1, A1, Withdrawal::Replaced)]);
    assert!(h.settle().is_empty());
    assert!(h.at(BEFORE_GRACE).is_empty());
    // The newer start withdrew the old call; its remaining fence expires
    // without a second terminal event, just as a cancelled claim's does.
    assert!(h.at(GRACE).is_empty());
    assert_eq!(&*h.settle(), &[assign(C1, R1, A2), placed(R1, A2)]);
}

#[test]
fn a_crossed_claim_whose_worker_is_lost_waits_the_grace_again() {
    let mut h = listed_crossed();
    h.at(DISCONNECT);
    h.step(Event::Lost { channel: C1 });
    assert!(h.at(DISCONNECT + BEFORE_GRACE).is_empty());
    assert_eq!(&*h.at(DISCONNECT + GRACE), &[lost(R1, A1)]);
}

#[test]
fn a_crossed_claims_parent_cancel_does_not_end_the_claim() {
    let mut h = listed_crossed();
    assert!(h.step(Event::Cancel { run: R1, attempt: A1 }).is_empty());
    assert_eq!(&*h.answer(C1, R1, A1, FAILED, 1), &[ack(C1, R1, A1), drop(1), lost(R1, A1)]);
}

#[test]
fn a_crossed_claims_call_is_dropped_without_relaying() {
    let mut h = listed_crossed();
    assert_eq!(
        &*h.step(Event::Relay { channel: C1, run: R1, attempt: A1, call: host_call(A2, payload(1)) }),
        &[Request::DropCall { call: host_call(A2, payload(1)) }]
    );
}

#[test]
fn crossing_a_stray_discards_its_waiting_turns() {
    let mut h = Harness::new(Limits { turns: 2, ..LIMITS });
    h.hello(C1, 2, &[], &[hosted(R1, A1, Phase::Active)]);
    turn(&mut h, C1, 1, 1);
    cross(&mut h, R1, A1);
    assert_eq!(&*h.settle(), &[turn_ack(C1, 1), drop(1)]);
}

#[test]
fn a_crossed_claim_stays_listed_across_worker_reconnects() {
    let mut h = crossed(false);
    assert_eq!(&*h.hello(C1, 2, &[], &[hosted(R1, A1, Phase::Active)]), &[cancel(C1, R1, A1)]);
    h.step(Event::Lost { channel: C1 });
    assert_eq!(&*h.hello(C2, 2, &[], &[hosted(R1, A1, Phase::Active)]), &[cancel(C2, R1, A1)]);
    assert_eq!(&*h.answer(C2, R1, A1, FAILED, 1), &[ack(C2, R1, A1), drop(1), lost(R1, A1)]);
}

#[test]
fn a_crossed_listing_and_an_excess_listing_fit_the_declared_output_room() {
    // One tracked slot, one extra listing allowed by hello's twice-slots
    // bound, and no grant/source traffic to mask the output reservation.
    let mut h = Harness::new(Limits {
        configured_workers: 1,
        workers: 1,
        slots: 1,
        attempts: 1,
        grants: 0,
        program_sources: 0,
        ..LIMITS
    });
    cross(&mut h, R1, A1);
    assert_eq!(
        &*h.hello(C1, 1, &[], &[hosted(R1, A1, Phase::Active), hosted(R2, A2, Phase::Active)]),
        &[cancel(C1, R1, A1), cancel(C1, R2, A2)]
    );
}

#[test]
fn a_crossed_worked_claim_no_worker_lists_is_lost_at_its_grace() {
    let mut h = crossed(true);
    assert!(h.at(BEFORE_GRACE).is_empty());
    assert_eq!(&*h.at(GRACE), &[lost(R1, A1)]);
    assert!(h.at(GRACE + 1).is_empty());
}

#[test]
fn a_crossed_stray_is_lost_at_its_grace_even_without_committed_work() {
    let mut h = Harness::new(LIMITS);
    h.hello(C1, 2, &[], &[hosted(R1, A1, Phase::Active)]);
    h.step(Event::Lost { channel: C1 });
    cross(&mut h, R1, A1);
    assert!(h.at(BEFORE_GRACE).is_empty());
    assert_eq!(&*h.at(GRACE), &[lost(R1, A1)]);
}
