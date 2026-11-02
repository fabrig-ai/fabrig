//! After-landing transitions (temper/docs/design/domain/after-landing.md, 6).
use super::*;
use crate::Verification;

fn merged(verification: Verification) -> Facts {
    Facts { pull: Pull::Merged { commit: MERGE }, verification, verification_bound: Duration::from_secs(10), ..facts() }
}

#[test]
fn an_unfollowed_merge_finishes_at_landed_and_finishes_again() {
    let f = merged(Verification::Unfollowed);
    let next = step(&change(State::Landing { head: HEAD }), &f, &heard(), at(20), &LIMITS);
    assert_eq!(next.change.state, State::Landed { merge: MERGE, since: at(20) });
    assert_eq!(next.decision, Decision::Finish { merge: MERGE });
    let again = step(&next.change, &f, &heard(), at(25), &LIMITS);
    assert_eq!(again, next);
}

#[test]
fn verifying_waits_from_the_first_observation_and_finishes_at_the_bound() {
    let f = merged(Verification::Verifying);
    let next = step(&change(State::Landing { head: HEAD }), &f, &heard(), at(20), &LIMITS);
    assert_eq!(next.change.state, State::Landed { merge: MERGE, since: at(20) });
    assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
    let before = step(&next.change, &f, &heard(), at(29), &LIMITS);
    assert_eq!(before, next);
    for now in [at(30), at(31)] {
        let ended = step(&before.change, &f, &heard(), now, &LIMITS);
        assert_eq!(ended.change.state, State::Uncleared { merge: MERGE });
        assert_eq!(ended.decision, Decision::Finish { merge: MERGE });
    }
}

#[test]
fn red_at_the_merge_waits_without_a_deadline_and_stays_red_on_verifying() {
    let mut f = merged(Verification::Red);
    let next = step(&change(State::Landing { head: HEAD }), &f, &heard(), at(20), &LIMITS);
    assert_eq!(next.change.state, State::Red { merge: MERGE, since: at(20) });
    assert_eq!(next.decision, Decision::Wait { until: None });
    for verification in [Verification::Verifying, Verification::Red, Verification::Unfollowed] {
        f.verification = verification;
        assert_eq!(step(&next.change, &f, &heard(), at(100), &LIMITS), next);
    }
}

#[test]
fn a_red_verdict_after_the_bound_still_waits_for_repair() {
    let f = merged(Verification::Red);
    let next = step(&change(State::Landed { merge: MERGE, since: at(20) }), &f, &heard(), at(40), &LIMITS);
    assert_eq!(next.change.state, State::Red { merge: MERGE, since: at(20) });
    assert_eq!(next.decision, Decision::Wait { until: None });
}

#[test]
fn cleared_finishes_from_the_merge_landed_or_red_even_past_the_bound() {
    let f = merged(Verification::Cleared);
    for state in [
        State::Landing { head: HEAD },
        State::Landed { merge: MERGE, since: at(20) },
        State::Red { merge: MERGE, since: at(20) },
    ] {
        let next = step(&change(state), &f, &heard(), at(100), &LIMITS);
        assert_eq!(next.change.state, State::Cleared { merge: MERGE });
        assert_eq!(next.decision, Decision::Finish { merge: MERGE });
    }
}

#[test]
fn cancelling_a_verifying_or_red_change_finishes_uncleared() {
    let mut h = heard();
    h.cancelled = true;
    for state in [State::Landed { merge: MERGE, since: at(20) }, State::Red { merge: MERGE, since: at(20) }] {
        for verification in
            [Verification::Unfollowed, Verification::Verifying, Verification::Red, Verification::Cleared]
        {
            let next = step(&change(state.clone()), &merged(verification), &h, at(21), &LIMITS);
            assert_eq!(next.change.state, State::Uncleared { merge: MERGE });
            assert_eq!(next.decision, Decision::Finish { merge: MERGE });
        }
    }
}

#[test]
fn cleared_and_uncleared_stay_finished_despite_later_facts_or_cancellation() {
    let mut h = heard();
    h.cancelled = true;
    for state in [State::Cleared { merge: MERGE }, State::Uncleared { merge: MERGE }] {
        let next = step(&change(state.clone()), &merged(Verification::Red), &h, at(100), &LIMITS);
        assert_eq!(next.change.state, state);
        assert_eq!(next.decision, Decision::Finish { merge: MERGE });
    }
}
