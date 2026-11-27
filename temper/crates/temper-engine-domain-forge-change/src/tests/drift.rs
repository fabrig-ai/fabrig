//! Branch observations, not a pull's delayed head, decide drift.
use super::*;

#[test]
fn a_lagging_pull_waits_with_its_stall_counting_then_takes_the_pushed_head() {
    for way in [crate::Way::Queue, crate::Way::Turn] {
        let mut c = change(State::Opening { requested: true });
        c.way = way;
        let mut f = Facts { pull: Pull::Open { head: NEW, base: BASE }, behind: Some(at(2)), ..facts() };
        let waiting = step(&c, &f, &heard(), at(2), &LIMITS);
        assert_eq!(waiting.decision, Decision::Wait { until: Some(at(32)) });
        assert_eq!(waiting.change, c);
        let stalled = step(&waiting.change, &f, &heard(), at(32), &LIMITS);
        assert_eq!(stalled.decision, Decision::Hold(Hold::Stalled));
        assert_eq!(stalled.change.last_head, Some(HEAD));
        f.pull = Pull::Open { head: HEAD, base: BASE };
        f.behind = None;
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
    let f = Facts {
        branch: Some(NEW),
        pull: Pull::Open { head: NEW, base: BASE },
        drift: Some(Hold::BranchMoved),
        ..facts()
    };
    let moved = decide(&c, &f, &heard());
    assert_eq!(moved.decision, Decision::Hold(Hold::BranchMoved));
    assert_eq!(moved.change.last_head, Some(HEAD));
    assert_eq!(moved.change.state, State::Held { was: Box::new(c.state), why: Hold::BranchMoved });
    let mut fresh = change(State::Opening { requested: true });
    fresh.last_head = None;
    assert_eq!(decide(&fresh, &Facts { drift: None, ..f }, &heard()).change.last_head, Some(NEW));
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

#[test]
fn a_step_reading_its_branch_missing_holds_as_deleted_not_moved() {
    let c = change(State::Checking { head: HEAD });
    let missing = decide(&c, &Facts { branch: None, drift: Some(Hold::BranchMissing), ..facts() }, &heard());
    assert_eq!(missing.decision, Decision::Hold(Hold::BranchMissing));
    let moved = decide(&c, &Facts { branch: Some(NEW), drift: Some(Hold::BranchMoved), ..facts() }, &heard());
    assert_eq!(moved.decision, Decision::Hold(Hold::BranchMoved));
}

#[test]
fn a_stale_prepare_is_dropped_without_holding_the_change() {
    let mut c = change(State::Ready { head: HEAD, ready_since: at(0) });
    c.way = crate::Way::Turn;
    let f = Facts { ask: Some(handed(crate::Ask::Prepare { head: NEW, main: BASE })), ..facts() };
    let next = step(&c, &f, &heard(), at(60), &LIMITS);
    assert_eq!(next.decision, Decision::Wait { until: None });
    assert_eq!(next.took, Some(crate::Took::Dropped(17)));
    assert_eq!(next.change, c);
}

#[test]
fn a_branch_at_the_steps_head_is_never_moved() {
    for state in [
        State::Opening { requested: true },
        State::Checking { head: HEAD },
        State::Updating { from: HEAD, base: BASE, ready_since: at(0) },
        State::Preparing { from: HEAD, main: BASE, ready_since: at(0) },
    ] {
        let c = change(state);
        let f = Facts { pull: Pull::Open { head: NEW, base: BASE }, ..facts() };
        assert_ne!(decide(&c, &f, &heard()).decision, Decision::Hold(Hold::BranchMoved), "{:?}", c.state);
    }
}
