//! Branch observations, not a pull's delayed head, decide drift.
use super::*;

#[test]
fn a_lagging_pull_waits_with_its_stall_counting_then_takes_the_pushed_head() {
    for way in [crate::Way::Queue, crate::Way::Turn] {
        let mut c = change(State::Opening { requested: true });
        c.way = way;
        let mut f = Facts { pull: Pull::Open { head: NEW, base: BASE }, ..facts() };
        let waiting = decide(&c, &f, &heard());
        assert_eq!(waiting.decision, Decision::Wait { until: Some(at(30)) });
        assert_eq!(waiting.change, c);
        let stalled = step(&waiting.change, &f, &heard(), at(30), &LIMITS);
        assert_eq!(stalled.decision, Decision::Hold(Hold::Stalled));
        assert_eq!(stalled.change.last_head, Some(HEAD));
        f.pull = Pull::Open { head: HEAD, base: BASE };
        let caught_up = decide(&waiting.change, &f, &heard());
        let expected = match way {
            crate::Way::Queue => State::Checking { head: HEAD },
            crate::Way::Turn => State::Gating { head: HEAD, asked: Box::new([]) },
        };
        assert_eq!(caught_up.change.state, expected);
        assert_eq!(caught_up.decision, Decision::None);
    }
}

#[test]
fn opening_drift_keeps_the_expected_head_even_when_the_pull_agrees_with_the_branch() {
    let c = change(State::Opening { requested: true });
    let f = Facts { branch: Some(NEW), pull: Pull::Open { head: NEW, base: BASE }, ..facts() };
    let moved = decide(&c, &f, &heard());
    assert_eq!(moved.decision, Decision::Hold(Hold::BranchMoved));
    assert_eq!(moved.change.last_head, Some(HEAD));
    assert_eq!(moved.change.state, State::Held { was: Box::new(c.state), why: Hold::BranchMoved });
    let mut fresh = change(State::Opening { requested: true });
    fresh.last_head = None;
    assert_eq!(decide(&fresh, &f, &heard()).change.last_head, Some(NEW));
}

#[test]
fn a_release_finding_its_pull_closed_reopens_instead_of_opening() {
    // A general release and a moved branch both keep ownership of their closed pull.
    for (why, was) in
        [(Hold::Stalled, State::Checking { head: HEAD }), (Hold::BranchMoved, State::Opening { requested: true })]
    {
        let c = change(State::Held { was: Box::new(was), why });
        let f = Facts { pull: Pull::Closed, ..facts() };
        let lifted = decide(&c, &f, &Heard { released: true, ..heard() });
        assert_eq!(lifted.change.state, State::Reopening { requested: false, retarget: false });
        let reopening = decide(&lifted.change, &f, &heard());
        assert_eq!(reopening.decision, Decision::Effect(Effect::Reopen));
    }
}

fn comparing_states() -> [State; 8] {
    [
        State::Checking { head: HEAD },
        State::Gating { head: HEAD, asked: Box::new([]) },
        State::CancellingGates { head: HEAD },
        State::Queued { head: HEAD, ready_since: at(0) },
        State::First { head: HEAD, ready_since: at(0) },
        State::Ready { head: HEAD, ready_since: at(0) },
        State::Judging { head: HEAD, ready_since: at(0), target: target(), asked: Box::new([]) },
        State::Opening { requested: true },
    ]
}

fn target() -> crate::Judgement {
    crate::Judgement {
        landing: 9,
        round: 1,
        pull: 10,
        branch: b"ready/9".as_slice().into(),
        base: b"main".as_slice().into(),
        prefix: MERGE,
    }
}

#[test]
fn a_step_reading_its_branch_missing_holds_as_deleted_not_moved() {
    for state in comparing_states() {
        let c = change(state);
        let missing = decide(&c, &Facts { branch: None, ..facts() }, &heard());
        assert_eq!(missing.decision, Decision::Hold(Hold::BranchMissing), "{:?}", c.state);
        let moved = decide(&c, &Facts { branch: Some(NEW), ..facts() }, &heard());
        assert_eq!(moved.decision, Decision::Hold(Hold::BranchMoved), "{:?}", c.state);
    }
}

#[test]
fn a_stale_prepare_request_is_not_a_branch_move() {
    let c = change(State::Ready { head: HEAD, ready_since: at(0) });
    let f = Facts { ask: Some(crate::Ask::Prepare { head: NEW, main: BASE }), ..facts() };
    let next = decide(&c, &f, &heard());
    assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
    assert_eq!(next.change, c);
}

#[test]
fn a_branch_at_the_steps_head_is_never_moved() {
    let other_states = [
        State::Recreating { head: HEAD },
        State::Updating { from: HEAD, base: BASE, ready_since: at(0) },
        State::Preparing { from: HEAD, main: BASE, ready_since: at(0) },
        State::Landing { head: HEAD },
        State::WaitingProducer { base: BASE },
        State::Resolving { requested: true, base: BASE },
        State::Landed { merge: HEAD, since: at(0) },
        State::Red { merge: HEAD, since: at(0) },
        State::Cleared { merge: HEAD },
        State::Uncleared { merge: HEAD },
    ];
    for state in comparing_states().into_iter().chain(other_states) {
        let c = change(state);
        let f = Facts { pull: Pull::Open { head: NEW, base: BASE }, ..facts() };
        assert_ne!(decide(&c, &f, &heard()).decision, Decision::Hold(Hold::BranchMoved), "{:?}", c.state);
    }
}
